use log::{debug, error};
use std::{fmt, fs};

use crate::hw::opcodes::{Condition, Instruction, Reg8, Reg16};
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
#[derive(Debug)]
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

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let z = if (self.f & 0x80) != 0 { 'Z' } else { '-' };
        let n = if (self.f & 0x40) != 0 { 'N' } else { '-' };
        let h = if (self.f & 0x20) != 0 { 'H' } else { '-' };
        let c = if (self.f & 0x10) != 0 { 'C' } else { '-' };

        writeln!(f, "\n┌─── CPU REGISTERS ────────────────────────┐")?;
        writeln!(
            f,
            "│ PC: {:#06X}   SP: {:#06X}   IME: {:<5}     │",
            self.pc, self.sp, self.ime
        )?;
        writeln!(f, "├───┬────────────┬───┬───────┬─────────────┤")?;
        writeln!(
            f,
            "│ A │ {:02X}         │ F │ {:02X}    │ [{}{}{}{}]      │",
            self.a, self.f, z, n, h, c
        )?;
        writeln!(
            f,
            "│ B │ {:02X}         │ C │ {:02X}    │ BC: {:04X}    │",
            self.b,
            self.c,
            ((self.b as u16) << 8) | self.c as u16
        )?;
        writeln!(
            f,
            "│ D │ {:02X}         │ E │ {:02X}    │ DE: {:04X}    │",
            self.d,
            self.e,
            ((self.d as u16) << 8) | self.e as u16
        )?;
        writeln!(
            f,
            "│ H │ {:02X}         │ L │ {:02X}    │ HL: {:04X}    │",
            self.h,
            self.l,
            ((self.h as u16) << 8) | self.l as u16
        )?;
        write!(f, "└───┴────────────┴───┴───────┴─────────────┘")
    }
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

    // Performs a CPU step
    pub fn step(&mut self) -> GBResult<()> {
        // Read 1B from memory at the current PC address
        let opcode = self.read_mem(self.pc);
        self.pc += 1;

        // Decode the instruction read from memory
        // This gives an enclosure s.t. if the decode function needs to read more that it can do so
        let instr = Instruction::decode(opcode, &mut || {
            let val = self.read_mem(self.pc);
            self.pc += 1;
            val
        });

        debug!("Performing instruction: {:?} - CPU state: {}", instr, self);

        // Execute the decoded instruction
        self.execute(instr)
    }

    // Simulate the decoded instruction performing the same operation
    fn execute(&mut self, instr: Instruction) -> GBResult<()> {
        match instr {
            _ => todo!("Implement instruction: {:?}", instr),
        }
        Ok(())
    }

    // Read 1B from the specified address in memory
    fn read_mem(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    // Write 1B to the specified address in memory
    fn write_mem(&mut self, addr: u16, val: u8) {
        self.memory[addr as usize] = val;
    }

    // Read value stored in the specified 8-bit register
    fn get_reg8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::H => self.h,
            Reg8::L => self.l,
            Reg8::HLIndirect => self.read_mem(self.get_reg16(Reg16::HL)),
        }
    }

    // Set value in the specified 8-bit register
    fn set_reg8(&mut self, reg: Reg8, val: u8) {
        match reg {
            Reg8::A => self.a = val,
            Reg8::B => self.b = val,
            Reg8::C => self.c = val,
            Reg8::D => self.d = val,
            Reg8::E => self.e = val,
            Reg8::H => self.h = val,
            Reg8::L => self.l = val,
            Reg8::HLIndirect => self.write_mem(self.get_reg16(Reg16::HL), val),
        }
    }

    // Read value stored in the specified 16-bit register
    fn get_reg16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::AF => ((self.a as u16) << 8) | (self.f as u16),
            Reg16::BC => ((self.b as u16) << 8) | (self.c as u16),
            Reg16::DE => ((self.d as u16) << 8) | (self.e as u16),
            Reg16::HL => ((self.h as u16) << 8) | (self.l as u16),
            Reg16::SP => self.sp,
        }
    }

    // Set value in the specified 16-bit register
    fn set_reg16(&mut self, reg: Reg16, val: u16) {
        match reg {
            Reg16::BC => {
                self.b = (val >> 8) as u8;
                self.c = val as u8;
            }
            Reg16::DE => {
                self.d = (val >> 8) as u8;
                self.e = val as u8;
            }
            Reg16::HL => {
                self.h = (val >> 8) as u8;
                self.l = val as u8;
            }
            Reg16::SP => self.sp = val,
            _ => unreachable!(),
        }
    }

    // Check if the specified condition is set
    fn check_condition(&self, cond: Condition) -> bool {
        match cond {
            Condition::None => true,
            Condition::Z => (self.f & 0x80) != 0,
            Condition::NZ => (self.f & 0x80) == 0,
            Condition::C => (self.f & 0x10) != 0,
            Condition::NC => (self.f & 0x10) == 0,
        }
    }
}
