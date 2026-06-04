use log::debug;
use std::fs;

use crate::hw::timer::Timer;
use crate::utils::error::GBResult;

const START: u16 = 0x0000;

const BOOT_ROM_SIZE: usize = 256;

const ROM_END: u16 = 0x7FFF;

const VRAM_SIZE: usize = 8192;
const VRAM_START: u16 = 0x8000;
const VRAM_END: u16 = 0x9FFF;

const WRAM_SIZE: usize = 8192;
const WRAM_START: u16 = 0xC000;
const WRAM_END: u16 = 0xDFFF;

const HRAM_SIZE: usize = 127;
const HRAM_START: u16 = 0xFF80;
const HRAM_END: u16 = 0xFFFE;

const TIMER_START: u16 = 0xFF04;
const TIMER_END: u16 = 0xFF07;

pub const IF_ADDRESS: u16 = 0xFF0F;
pub const IE_ADDRESS: u16 = 0xFFFF;

/// Simulates the memory map of Gameboy
/// `boot_rom` -> boot rom
/// `rom` -> cartridge rom
/// `vram` -> video ram
/// `wram` -> work ram
/// `hram` -> high ram
/// `timer` -> holds the timers for the [`crate::hw::cpu::Cpu`]
/// `if_reg` -> interrupt flag
/// `ie_reg` -> interrupt enable
#[derive(Debug)]
#[allow(dead_code)]
pub struct Bus {
    boot_rom: [u8; BOOT_ROM_SIZE],
    rom: Vec<u8>,
    vram: [u8; VRAM_SIZE],
    wram: [u8; WRAM_SIZE],
    hram: [u8; HRAM_SIZE],
    timer: Timer,
    if_reg: u8,
    ie_reg: u8,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            boot_rom: [0u8; BOOT_ROM_SIZE],
            rom: Vec::new(),
            vram: [0u8; VRAM_SIZE],
            wram: [0u8; WRAM_SIZE],
            hram: [0u8; HRAM_SIZE],
            timer: Timer::new(),
            if_reg: 0,
            ie_reg: 0,
        }
    }

    pub fn load_rom(&mut self, rom_path: &str) -> GBResult<()> {
        let buffer = fs::read(rom_path)?;
        let rom_size = buffer.len();

        self.rom = buffer;

        debug!("ROM successfully loaded into Bus, read {} bytes", rom_size);
        Ok(())
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
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            START..=ROM_END if (addr as usize) < self.rom.len() => self.rom[addr as usize],
            VRAM_START..=VRAM_END => self.vram[(addr - VRAM_START) as usize],
            WRAM_START..=WRAM_END => self.wram[(addr - WRAM_START) as usize],
            HRAM_START..=HRAM_END => self.hram[(addr - HRAM_START) as usize],
            TIMER_START..=TIMER_END => self.timer.read(addr),
            IF_ADDRESS => self.if_reg,
            IE_ADDRESS => self.ie_reg,
            0xFF44 => 0x90, // TODO hardcode VBlank so loops work
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            VRAM_START..=VRAM_END => self.vram[(addr - VRAM_START) as usize] = val,
            WRAM_START..=WRAM_END => self.wram[(addr - WRAM_START) as usize] = val,
            HRAM_START..=HRAM_END => self.hram[(addr - HRAM_START) as usize] = val,
            TIMER_START..=TIMER_END => self.timer.write(addr, val),
            IF_ADDRESS => self.if_reg = val,
            IE_ADDRESS => self.ie_reg = val,
            _ => {}
        }
    }
}
