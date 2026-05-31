use log::{debug, error};
use std::fs;

use crate::utils::error::{GBError, GBResult};

/// Type that represents the cpu for the SM83 core
/// `pc` -> program counter
/// `sp` -> stack pointer
/// `a` -> `A` part of the `AF` register (accumulator and flags)
/// `f` -> `F` part of the `AF` register (accumulator and flags)
/// `b` -> `B` part of the `BC` register
/// `c` -> `C` part of the `BC` register
/// `d` -> `D` part of the `DE` register
/// `e` -> `E` part of the `DE` register
/// `h` -> `H` part of the `HL` register
/// `l` -> `L` part of the `HL` register
/// `ime` -> interrupts enablement
/// `memory` -> memory
pub struct Cpu {
    pc: u16,
    sp: u16,
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    ime: bool,
    pub memory: [u8; 65536],
}

impl Cpu {
    // Initialize the cpu, setting `pc` to 0x100
    pub fn new() -> Self {
        Self {
            pc: 0x100,
            sp: 0,
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            ime: false,
            memory: [0; 65536],
        }
    }

    // Reads ROM file and puts it into memory
    pub fn load_rom(&mut self, path: &str) -> GBResult<()> {
        let buffer = fs::read(path)?;
        let rom_size = buffer.len();

        if rom_size > self.memory.len() {
            error!(
                "ROM ({}) is larger than memory space ({})",
                rom_size,
                self.memory.len()
            );
            return Err(GBError::OversizedROM);
        }

        // Copy the bytes into memory
        self.memory[..rom_size].copy_from_slice(&buffer[..rom_size]);

        debug!("ROM successfully loaded, read {} bytes", rom_size);

        Ok(())
    }
}
