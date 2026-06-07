use log::debug;
use std::fs;

use crate::hw::{self, cartridge::Cartridge, joypad::Joypad, ppu::Ppu, timer::Timer};
use crate::utils::error::GBResult;

/// Simulates the memory map of Gameboy
/// `rom` -> 16 KiB cartridge rom from bank 00 to NN (0000-7FFF) and 16 KiB external ram (A000-BFFF)
///         managed by the [`Cartridge`]
/// `vram` -> 8 KiB video ram (8000-9FFF)
///         managed by the [`Ppu`]
/// `wram` -> 8 KiB work ram (C000-DFFF)
/// `eram` -> echo ram, mirror of wram (prohibited use, E000-FDFF)
/// `oam` -> Object attribute memory (FE00-FE9F)
///         managed by the [`Ppu`]
/// `priv` -> Not usable memory space (FEA0-FEFF)
/// `io` -> I/O registers (FF00-FF7F)
///         managed by the [`Ppu`], [`Joypad`], and [`Timer`]
/// `hram` -> High ram (FF80-FFFE)
/// `if` -> Interrupt flag
/// `ie` -> Interrupt enable register (FFFF-FFFF)
#[derive(Debug)]
#[allow(dead_code)]
pub struct Bus {
    rom: Cartridge,
    wram: [u8; hw::WRAM_SIZE],
    hram: [u8; hw::HRAM_SIZE],
    timer: Timer,
    ppu: Ppu,
    joypad: Joypad,
    if_reg: u8,
    ie_reg: u8,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            rom: Cartridge::new(),
            wram: [hw::U8_ZERO; hw::WRAM_SIZE],
            hram: [hw::U8_ZERO; hw::HRAM_SIZE],
            timer: Timer::new(),
            ppu: Ppu::new(),
            joypad: Joypad::new(),
            if_reg: hw::U8_ZERO,
            ie_reg: hw::U8_ZERO,
        }
    }

    /// Helper to load input ROM in memory
    pub fn load_rom(&mut self, rom_path: &str) -> GBResult<()> {
        let buffer = fs::read(rom_path)?;
        let rom_size = buffer.len();

        self.rom.load(buffer);

        debug!("ROM successfully loaded into Bus, read {} bytes", rom_size);
        Ok(())
    }

    /// Helper to read the frame buffer
    pub fn get_frame(&self) -> [u32; hw::SCREEN_WIDTH * hw::SCREEN_HEIGHT] {
        self.ppu.get_frame()
    }

    /// Helper to set a button in the [`Joypad`]
    pub fn set_button(&mut self, button: hw::Buttons) {
        self.joypad.set_button(button)
    }

    /// Helper to unset all buttons in the [`Joypad`]
    pub fn unset_buttons(&mut self) {
        self.joypad.unset_buttons()
    }

    /// Helper to tick the [`Timer`]
    pub fn tick(&mut self, cycles: u32) {
        // Advance the timer
        self.timer.tick(cycles);

        // If timer has an interrupt, set bit 2 of the IF register
        if self.timer.get_interrupt() {
            self.if_reg |= 0x04; // Timer interrupt flag
            self.timer.set_interrupt(false);
        }

        self.ppu.tick(cycles);
        // If PPU has an interrupt, set bit 0 (V-Blank) of the IF register
        if self.ppu.get_interrupt() {
            self.if_reg |= 0x01; // V-Blank interrupt flag
            self.ppu.set_interrupt(false);
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            hw::ROM_START..=hw::ROM_END => self.rom.read(addr),
            hw::VRAM_START..=hw::VRAM_END => self.ppu.read(addr),
            hw::EXTERNAL_RAM_START..=hw::EXTERNAL_RAM_END => self.rom.read(addr),
            hw::WRAM_START..=hw::WRAM_END => self.wram[(addr - hw::WRAM_START) as usize],
            hw::ERAM_START..=hw::ERAM_END => panic!("Prohibited access to echo ram"),
            hw::OAM_START..=hw::OAM_END => self.ppu.read(addr),
            hw::PRIV_START..=hw::PRIV_END => panic!("Prohibited access to private memory"),
            hw::IO_JOYPAD => self.joypad.read(),
            hw::IO_SERIAL_START..=hw::IO_SERIAL_END => 0x00,
            hw::IO_TIMER_START..=hw::IO_TIMER_END => self.timer.read(addr),
            hw::IF_ADDRESS => self.if_reg,
            hw::IO_AUDIO_START..=hw::IO_AUDIO_END => 0x00,
            hw::IO_WAVE_START..=hw::IO_WAVE_END => 0x00,
            hw::IO_OAM_DMA => 0x00,
            hw::IO_LCD_START..=hw::IO_LCD_END => self.ppu.read(addr),
            hw::IO_KEY_0 => 0x00,
            hw::IO_KEY_1 => 0x00,
            hw::IO_VRAM_BANK_SEL => 0x00,
            hw::IO_ROM_MAPPING => 0x00,
            hw::IO_VRAM_DMA_START..=hw::IO_VRAM_DMA_END => 0x00,
            hw::IO_IR_PORT => 0x00,
            hw::IO_PALETTES_START..=hw::IO_PALETTES_END => 0x00,
            hw::IO_OBJ_PRIORITY => 0x00,
            hw::IO_WRAM_BANK_SEL => 0x00,
            hw::HRAM_START..=hw::HRAM_END => self.hram[(addr - hw::HRAM_START) as usize],
            hw::IE_ADDRESS => self.ie_reg,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            hw::ROM_START..=hw::ROM_END => self.rom.write(addr, val),
            hw::VRAM_START..=hw::VRAM_END => self.ppu.write(addr, val),
            hw::EXTERNAL_RAM_START..=hw::EXTERNAL_RAM_END => self.rom.write(addr, val),
            hw::WRAM_START..=hw::WRAM_END => self.wram[(addr - hw::WRAM_START) as usize] = val,
            hw::ERAM_START..=hw::ERAM_END => panic!("Prohibited access to echo ram"),
            hw::OAM_START..=hw::OAM_END => self.ppu.write(addr, val),
            hw::PRIV_START..=hw::PRIV_END => panic!("Prohibited access to private memory"),
            hw::IO_JOYPAD => self.joypad.write(val),
            hw::IO_SERIAL_START..=hw::IO_SERIAL_END => {}
            hw::IO_TIMER_START..=hw::IO_TIMER_END => self.timer.write(addr, val),
            hw::IF_ADDRESS => self.if_reg = val,
            hw::IO_AUDIO_START..=hw::IO_AUDIO_END => {}
            hw::IO_WAVE_START..=hw::IO_WAVE_END => {}
            hw::IO_OAM_DMA => {
                let src_addr = (val as u16) << 8;
                for i in 0..hw::OAM_SIZE as u16 {
                    let byte = self.read(src_addr + i);
                    self.ppu.write(hw::OAM_START + i, byte);
                }
            }
            hw::IO_LCD_START..=hw::IO_LCD_END => self.ppu.write(addr, val),
            hw::IO_KEY_0 => {}
            hw::IO_KEY_1 => {}
            hw::IO_VRAM_BANK_SEL => {}
            hw::IO_ROM_MAPPING => {}
            hw::IO_VRAM_DMA_START..=hw::IO_VRAM_DMA_END => {}
            hw::IO_IR_PORT => {}
            hw::IO_PALETTES_START..=hw::IO_PALETTES_END => {}
            hw::IO_OBJ_PRIORITY => {}
            hw::IO_WRAM_BANK_SEL => {}
            hw::HRAM_START..=hw::HRAM_END => self.hram[(addr - hw::HRAM_START) as usize] = val,
            hw::IE_ADDRESS => self.ie_reg = val,
            _ => {}
        }
    }
}
