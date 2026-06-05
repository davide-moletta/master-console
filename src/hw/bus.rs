use log::debug;
use std::fs;

use crate::hw::cartridge::{self, Cartridge};
use crate::hw::joypad::{self, Joypad};
use crate::hw::ppu::{self, Ppu};
use crate::hw::timer::Timer;
use crate::utils::error::GBResult;

const START: u16 = 0x0000;

const BOOT_ROM_SIZE: usize = 256;

const ROM_END: u16 = 0x7FFF;

const WRAM_SIZE: usize = 8192;
const WRAM_START: u16 = 0xC000;
const WRAM_END: u16 = 0xDFFF;

const HRAM_SIZE: usize = 127;
const HRAM_START: u16 = 0xFF80;
const HRAM_END: u16 = 0xFFFE;

const JOYPAD_ADDRESS: u16 = 0xFF00;

const TIMER_START: u16 = 0xFF04;
const TIMER_END: u16 = 0xFF07;

const DMA_ADDRESS: u16 = 0xFF46;

pub const IF_ADDRESS: u16 = 0xFF0F;
pub const IE_ADDRESS: u16 = 0xFFFF;

/// Simulates the memory map of Gameboy
/// `boot_rom` -> boot rom
/// `rom` -> cartridge rom
/// `wram` -> work ram
/// `hram` -> high ram
/// `timer` -> holds the [`Timer`] for the [`crate::hw::cpu::Cpu`]
/// `ppu` -> holds the [`Ppu`] to render the screen
/// `joypad` -> holds the [`Joypad`] to interact with user
/// `if_reg` -> interrupt flag
/// `ie_reg` -> interrupt enable
#[derive(Debug)]
#[allow(dead_code)]
pub struct Bus {
    boot_rom: [u8; BOOT_ROM_SIZE],
    rom: Cartridge,
    wram: [u8; WRAM_SIZE],
    hram: [u8; HRAM_SIZE],
    timer: Timer,
    ppu: Ppu,
    joypad: Joypad,
    if_reg: u8,
    ie_reg: u8,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            boot_rom: [0u8; BOOT_ROM_SIZE],
            rom: Cartridge::new(vec![0; cartridge::RAM_SIZE]),
            wram: [0u8; WRAM_SIZE],
            hram: [0u8; HRAM_SIZE],
            timer: Timer::new(),
            ppu: Ppu::new(),
            joypad: Joypad::new(),
            if_reg: 0,
            ie_reg: 0,
        }
    }

    pub fn load_rom(&mut self, rom_path: &str) -> GBResult<()> {
        let buffer = fs::read(rom_path)?;
        let rom_size = buffer.len();

        self.rom = Cartridge::new(buffer);

        debug!("ROM successfully loaded into Bus, read {} bytes", rom_size);
        Ok(())
    }

    /// Helper to read the frame buffer
    pub fn get_frame(&self) -> [u32; ppu::SCREEN_WIDTH * ppu::SCREEN_HEIGHT] {
        self.ppu.get_frame()
    }

    /// Helper to interact with joypad
    pub fn set_button(&mut self, button: joypad::Buttons, val: bool) {
        self.joypad.set_button(button, val)
    }

    /// Helper to tick the [`Timer`]
    pub fn tick(&mut self, cycles: u32) {
        // Advance the timer
        self.timer.tick(cycles);

        // If the timer requested an interrupt, set the [`if_reg`] bit
        if self.timer.get_interrupt() {
            self.if_reg |= 0x04;
            self.timer.set_interrupt(false);
        }

        self.ppu.tick(cycles);
        if self.ppu.get_interrupt() {
            self.if_reg |= 0x01;
            self.ppu.set_interrupt(false);
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            START..=ROM_END => self.rom.read(addr),
            cartridge::RAM_START..=cartridge::RAM_END => self.rom.read(addr),
            ppu::VRAM_START..=ppu::VRAM_END => self.ppu.read(addr),
            ppu::OAM_START..=ppu::OAM_END => self.ppu.read(addr),
            ppu::LCDC_ADDRESS => self.ppu.read(addr),
            ppu::STAT_ADDRESS => self.ppu.read(addr),
            ppu::SCY_ADDRESS => self.ppu.read(addr),
            ppu::SCX_ADDRESS => self.ppu.read(addr),
            ppu::LY_ADDRESS => self.ppu.read(addr),
            ppu::LYC_ADDRESS => self.ppu.read(addr),
            ppu::BGP_ADDRESS => self.ppu.read(addr),
            ppu::OBP0_ADDRESS => self.ppu.read(addr),
            ppu::OBP1_ADDRESS => self.ppu.read(addr),
            WRAM_START..=WRAM_END => self.wram[(addr - WRAM_START) as usize],
            HRAM_START..=HRAM_END => self.hram[(addr - HRAM_START) as usize],
            JOYPAD_ADDRESS => self.joypad.read(),
            TIMER_START..=TIMER_END => self.timer.read(addr),
            IF_ADDRESS => self.if_reg,
            IE_ADDRESS => self.ie_reg,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            START..=ROM_END | cartridge::RAM_START..=cartridge::RAM_END => {
                self.rom.write(addr, val)
            }
            ppu::VRAM_START..=ppu::VRAM_END => self.ppu.write(addr, val),
            ppu::OAM_START..=ppu::OAM_END => self.ppu.write(addr, val),
            ppu::LCDC_ADDRESS => self.ppu.write(addr, val),
            ppu::STAT_ADDRESS => self.ppu.write(addr, val),
            ppu::SCY_ADDRESS => self.ppu.write(addr, val),
            ppu::SCX_ADDRESS => self.ppu.write(addr, val),
            ppu::LY_ADDRESS => { /* Do nothing, LY is read-only for the CPU */ }
            ppu::LYC_ADDRESS => self.ppu.write(addr, val),
            ppu::BGP_ADDRESS => self.ppu.write(addr, val),
            ppu::OBP0_ADDRESS => self.ppu.write(addr, val),
            ppu::OBP1_ADDRESS => self.ppu.write(addr, val),
            WRAM_START..=WRAM_END => self.wram[(addr - WRAM_START) as usize] = val,
            HRAM_START..=HRAM_END => self.hram[(addr - HRAM_START) as usize] = val,
            JOYPAD_ADDRESS => self.joypad.write(val),
            TIMER_START..=TIMER_END => self.timer.write(addr, val),
            DMA_ADDRESS => {
                let source_base = (val as u16) << 8;
                for i in 0..160 {
                    let data = self.read(source_base + i);
                    self.ppu.write(0xFE00 + i, data);
                }
            }
            IF_ADDRESS => self.if_reg = val,
            IE_ADDRESS => self.ie_reg = val,
            _ => {}
        }
    }
}
