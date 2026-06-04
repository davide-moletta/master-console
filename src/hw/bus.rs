use log::debug;
use std::fs;

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

/// Simulates the memory map of Gameboy
/// `boot_rom` -> boot rom
/// `rom` -> cartridge rom
/// `vram` -> video ram
/// `wram` -> work ram
/// `hram` -> high ram
#[derive(Debug)]
#[allow(dead_code)]
pub struct Bus {
    boot_rom: [u8; BOOT_ROM_SIZE],
    rom: Vec<u8>,
    vram: [u8; VRAM_SIZE],
    wram: [u8; WRAM_SIZE],
    hram: [u8; HRAM_SIZE],
}

impl Bus {
    pub fn new() -> Self {
        Self {
            boot_rom: [0u8; BOOT_ROM_SIZE],
            rom: Vec::new(),
            vram: [0u8; VRAM_SIZE],
            wram: [0u8; WRAM_SIZE],
            hram: [0u8; HRAM_SIZE],
        }
    }

    pub fn load_rom(&mut self, rom_path: &str) -> GBResult<()> {
        let buffer = fs::read(rom_path)?;
        let rom_size = buffer.len();

        self.rom = buffer;

        debug!("ROM successfully loaded into Bus, read {} bytes", rom_size);
        Ok(())
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            START..=ROM_END if (addr as usize) < self.rom.len() => self.rom[addr as usize],
            VRAM_START..=VRAM_END => self.vram[(addr - VRAM_START) as usize],
            WRAM_START..=WRAM_END => self.wram[(addr - WRAM_START) as usize],
            HRAM_START..=HRAM_END => self.hram[(addr - HRAM_START) as usize],
            0xFF44 => 0x90, // TODO hardcode VBlank so loops work
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            VRAM_START..=VRAM_END => self.vram[(addr - VRAM_START) as usize] = val,
            WRAM_START..=WRAM_END => self.wram[(addr - WRAM_START) as usize] = val,
            HRAM_START..=HRAM_END => self.hram[(addr - HRAM_START) as usize] = val,
            _ => {}
        }
    }
}
