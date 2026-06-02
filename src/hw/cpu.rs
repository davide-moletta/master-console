use log::{debug, error, warn};
use std::{fmt, fs};

use crate::hw::opcodes::{Condition, Instruction, Reg8, Reg16};
use crate::utils::error::{GBError, GBResult};

const IO_MEMORY_START: u16 = 0xFF00;

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
    halted: bool,
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
            halted: false,
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
        // If halted, check for interrupts to wake up
        if self.halted {
            // Check if IME flag is not zero
            let ie = self.read_mem(0xFFFF);
            let if_flag = self.read_mem(0xFF0F);

            if (ie & if_flag) != 0 {
                self.halted = false;
            } else {
                // Still halted
                return Ok(());
            }
        }

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
            Instruction::NOP => {}

            Instruction::STOP => {
                warn!("STOP executed, entering deep sleep");
                // TODO for now work like HALT
                self.halted = true;
            }

            Instruction::HALT => self.halted = true,

            Instruction::DI => self.ime = false,

            Instruction::EI => self.ime = true,

            Instruction::LD_8(dst, src) => {
                let val = self.get_reg8(src);
                self.set_reg8(dst, val);
            }

            Instruction::LD_8_IMM(dst, val) => {
                self.set_reg8(dst, val);
            }

            Instruction::LDH_A_IMM(val) => {
                let address = IO_MEMORY_START | (val as u16);
                self.a = self.read_mem(address);
            }

            Instruction::LDH_IMM_A(val) => {
                let address = IO_MEMORY_START | (val as u16);
                self.write_mem(address, self.a);
            }

            Instruction::LDH_A_C => {
                let address = IO_MEMORY_START | (self.c as u16);
                self.a = self.read_mem(address);
            }

            Instruction::LDH_C_A => {
                let address = IO_MEMORY_START | (self.c as u16);
                self.write_mem(address, self.a);
            }

            Instruction::LD_A_IMM_16(val) => self.a = self.read_mem(val),

            Instruction::LD_IMM_16_A(val) => self.write_mem(val, self.a),

            Instruction::LD_A_BC => {
                let address = self.get_reg16(Reg16::BC);
                self.a = self.read_mem(address);
            }

            Instruction::LD_A_DE => {
                let address = self.get_reg16(Reg16::DE);
                self.a = self.read_mem(address);
            }

            Instruction::LD_BC_A => {
                let address = self.get_reg16(Reg16::BC);
                self.write_mem(address, self.a);
            }

            Instruction::LD_DE_A => {
                let address = self.get_reg16(Reg16::DE);
                self.write_mem(address, self.a);
            }

            Instruction::LD_A_HLI => {
                let hl = self.get_reg16(Reg16::HL);
                self.a = self.read_mem(hl);
                self.set_reg16(Reg16::HL, hl.wrapping_add(1));
            }

            Instruction::LD_A_HLD => {
                let hl = self.get_reg16(Reg16::HL);
                self.a = self.read_mem(hl);
                self.set_reg16(Reg16::HL, hl.wrapping_sub(1));
            }

            Instruction::LD_HLI_A => {
                let hl = self.get_reg16(Reg16::HL);
                self.set_reg16(Reg16::HL, hl.wrapping_add(1));
                self.write_mem(hl, self.a);
            }

            Instruction::LD_HLD_A => {
                let hl = self.get_reg16(Reg16::HL);
                self.set_reg16(Reg16::HL, hl.wrapping_sub(1));
                self.write_mem(hl, self.a);
            }

            Instruction::LD_16_IMM(dst, val) => self.set_reg16(dst, val),

            Instruction::LD_IMM_16_SP(val) => {
                let sp_val = self.sp;

                // Extract low byte (bits 0-7) and high byte (bits 8-15)
                let low_byte = (sp_val & 0xFF) as u8;
                let high_byte = (sp_val >> 8) as u8;

                self.write_mem(val, low_byte);
                self.write_mem(val.wrapping_add(1), high_byte);
            }

            Instruction::LD_SP_HL => self.sp = self.get_reg16(Reg16::HL),

            Instruction::LD_HL_SP_e8(val) => {
                let sp = self.sp;

                // Treat val as a u16 for the carry bit logic,
                // but the actual addition is signed
                let offset = val as u16;

                // Calculate flags
                let h_check = (sp & 0x0F) + (offset & 0x0F) > 0x0F;
                let c_check = (sp & 0xFF) + (offset & 0xFF) > 0xFF;

                // Reset Z and N, set H and C
                self.f = 0;
                if h_check {
                    self.f |= 0x20;
                }
                if c_check {
                    self.f |= 0x10;
                }

                // Calculate the 16-bit result
                let result = sp.wrapping_add(val as i16 as u16);
                self.set_reg16(Reg16::HL, result);
            }

            Instruction::PUSH(src) => {
                let val = self.get_reg16(src);
                self.push_stack((val >> 8) as u8, (val & 0xFF) as u8);
            }

            Instruction::POP(dst) => {
                let (low, high) = self.pop_stack();
                let val = ((high as u16) << 8) | (low as u16);

                if dst == Reg16::AF {
                    // Bits 0-3 of F are always 0 on Game Boy
                    self.set_reg16(Reg16::AF, val & 0xFFF0);
                } else {
                    self.set_reg16(dst, val);
                }
            }

            Instruction::ADD(src) => {
                let val = self.get_reg8(src);
                let result = self.a.wrapping_add(val);

                let z = result == 0;
                let n = false;
                let h = (self.a & 0x0F) + (val & 0x0F) > 0x0F;
                let c = (self.a as u16) + (val as u16) > 0xFF;

                self.set_flags(z, n, h, c);
                self.a = result;
            }

            Instruction::ADD_IMM(val) => {
                let result = self.a.wrapping_add(val);

                let z = result == 0;
                let n = false;
                let h = (self.a & 0x0F) + (val & 0x0F) > 0x0F;
                let c = (self.a as u16) + (val as u16) > 0xFF;

                self.set_flags(z, n, h, c);
                self.a = result;
            }

            Instruction::ADC(src) => {
                let val = self.get_reg8(src);

                // Get current carry flag
                let carry = if (self.f & 0x10) != 0 { 1 } else { 0 };

                // Calculate result wrapped around u8
                let result = self.a.wrapping_add(val).wrapping_add(carry);

                let z = result == 0;
                let n = false;
                let h = (self.a & 0x0F) as u16 + (val & 0x0F) as u16 + (carry as u16) > 0x0F;
                let c = (self.a as u16) + (val as u16) + (carry as u16) > 0xFF;

                self.set_flags(z, n, h, c);
                self.a = result;
            }

            Instruction::ADC_IMM(val) => {
                // Get current carry flag
                let carry = if (self.f & 0x10) != 0 { 1 } else { 0 };

                // Calculate result wrapped around u8
                let result = self.a.wrapping_add(val).wrapping_add(carry);

                let z = result == 0;
                let n = false;
                let h = (self.a & 0x0F) as u16 + (val & 0x0F) as u16 + (carry as u16) > 0x0F;
                let c = (self.a as u16) + (val as u16) + (carry as u16) > 0xFF;

                self.set_flags(z, n, h, c);
                self.a = result;
            }

            Instruction::SUB(src) => {
                let val = self.get_reg8(src);
                let result = self.a.wrapping_sub(val);

                let z = result == 0;
                let n = true;
                let h = (self.a & 0x0F) < (val & 0x0F);
                let c = self.a < val;

                self.set_flags(z, n, h, c);
                self.a = result;
            }

            Instruction::SUB_IMM(val) => {
                let result = self.a.wrapping_sub(val);

                let z = result == 0;
                let n = true;
                let h = (self.a & 0x0F) < (val & 0x0F);
                let c = self.a < val;

                self.set_flags(z, n, h, c);
                self.a = result;
            }

            Instruction::SBC(src) => {
                let val = self.get_reg8(src);

                // Get current carry flag
                let carry = if (self.f & 0x10) != 0 { 1 } else { 0 };

                // Calculate result wrapped around u8
                let result = self.a.wrapping_sub(val).wrapping_sub(carry);

                let z = result == 0;
                let n = true;
                let h = (self.a as u16 & 0x0F) < (val as u16 & 0x0F) + (carry as u16);
                let c = (self.a as u16) < (val as u16) + (carry as u16);

                self.set_flags(z, n, h, c);
                self.a = result;
            }

            Instruction::SBC_IMM(val) => {
                // Get current carry flag
                let carry = if (self.f & 0x10) != 0 { 1 } else { 0 };

                // Calculate result wrapped around u8
                let result = self.a.wrapping_sub(val).wrapping_sub(carry);

                let z = result == 0;
                let n = true;
                let h = (self.a as u16 & 0x0F) < (val as u16 & 0x0F) + (carry as u16);
                let c = (self.a as u16) < (val as u16) + (carry as u16);

                self.set_flags(z, n, h, c);
                self.a = result;
            }

            Instruction::AND(src) => {
                let val = self.get_reg8(src);
                self.a &= val;

                let z = self.a == 0;
                self.set_flags(z, false, true, false);
            }

            Instruction::AND_IMM(val) => {
                self.a &= val;

                let z = self.a == 0;
                self.set_flags(z, false, true, false);
            }

            Instruction::XOR(src) => {
                let val = self.get_reg8(src);
                self.a ^= val;

                let z = self.a == 0;
                self.set_flags(z, false, false, false);
            }

            Instruction::XOR_IMM(val) => {
                self.a ^= val;

                let z = self.a == 0;
                self.set_flags(z, false, false, false);
            }

            Instruction::OR(src) => {
                let val = self.get_reg8(src);
                self.a |= val;

                let z = self.a == 0;
                self.set_flags(z, false, false, false);
            }

            Instruction::OR_IMM(val) => {
                self.a |= val;

                let z = self.a == 0;
                self.set_flags(z, false, false, false);
            }

            Instruction::CP(src) => {
                let val = self.get_reg8(src);
                let result = self.a.wrapping_sub(val);

                let z = result == 0;
                let n = true;
                let h = (self.a & 0x0F) < (val & 0x0F);
                let c = self.a < val;

                self.set_flags(z, n, h, c);
            }

            Instruction::CP_IMM(val) => {
                let result = self.a.wrapping_sub(val);

                let z = result == 0;
                let n = true;
                let h = (self.a & 0x0F) < (val & 0x0F);
                let c = self.a < val;

                self.set_flags(z, n, h, c);
            }

            Instruction::INC_8(reg) => {
                let val = self.get_reg8(reg);
                let result = val.wrapping_add(1);
                self.set_reg8(reg, result);

                let z = result == 0;
                let n = false;
                let h = (val & 0x0F) == 0x0F;
                let c = (self.f & 0x10) != 0;

                self.set_flags(z, n, h, c);
            }

            Instruction::DEC_8(reg) => {
                let val = self.get_reg8(reg);
                let result = val.wrapping_sub(1);
                self.set_reg8(reg, result);

                let z = result == 0;
                let n = true;
                let h = (val & 0x0F) == 0;
                let c = (self.f & 0x10) != 0;

                self.set_flags(z, n, h, c);
            }

            Instruction::ADD_16(src) => {
                let val = self.get_reg16(src);
                let hl = self.get_reg16(Reg16::HL);
                let result = hl.wrapping_add(val);

                let z = (self.f & 0x80) != 0;
                let n = false;
                let h = (hl & 0x0FFF) + (val & 0x0FFF) > 0x0FFF;
                let c = (hl as u32) + (val as u32) > 0xFFFF;

                self.set_flags(z, n, h, c);
                self.set_reg16(Reg16::HL, result);
            }

            Instruction::ADD_16_SP_e8(val) => {
                // Cast val to i16 to handle signs, then to u16 to add to SP
                let offset = val as i16 as u16;
                let result = self.sp.wrapping_add(offset);

                // Flags are calculated based on the unsigned low byte addition
                let unsigned_val = val as u8 as u16;

                // Half-Carry: carry from bit 3
                let h = (self.sp & 0x0F) + (unsigned_val & 0x0F) > 0x0F;

                // Carry: carry from bit 7
                let c = (self.sp & 0xFF) + (unsigned_val & 0xFF) > 0xFF;

                self.set_flags(false, false, h, c);
                self.sp = result;
            }

            Instruction::INC_16(reg) => {
                let val = self.get_reg16(reg);
                self.set_reg16(reg, val.wrapping_add(1));
            }

            Instruction::DEC_16(reg) => {
                let val = self.get_reg16(reg);
                self.set_reg16(reg, val.wrapping_sub(1));
            }

            Instruction::JP(cond, val) => {
                if self.check_condition(cond) {
                    self.pc = val;
                }
            }

            Instruction::JP_HL => self.pc = self.get_reg16(Reg16::HL),

            Instruction::JR(cond, offset) => {
                if self.check_condition(cond) {
                    // Cast offset to i16 to handle negative numbers, then back to u16
                    self.pc = (self.pc as i16 + offset as i16) as u16;
                }
            }

            Instruction::CALL(cond, val) => {
                if self.check_condition(cond) {
                    // Push PC onto the stack
                    let pc_bytes = self.pc.to_be_bytes();
                    self.push_stack(pc_bytes[0], pc_bytes[1]);

                    self.pc = val;
                }
            }

            Instruction::RET(cond) => {
                if self.check_condition(cond) {
                    // Pop return address from the stack
                    let low = self.read_mem(self.sp);
                    self.sp = self.sp.wrapping_add(1);
                    let high = self.read_mem(self.sp);
                    self.sp = self.sp.wrapping_add(1);

                    self.pc = ((high as u16) << 8) | (low as u16);
                }
            }

            Instruction::RETI => {
                // Pop return address from the stack
                let low = self.read_mem(self.sp);
                self.sp = self.sp.wrapping_add(1);
                let high = self.read_mem(self.sp);
                self.sp = self.sp.wrapping_add(1);

                self.pc = ((high as u16) << 8) | (low as u16);

                // Enable interrupts
                self.ime = true;
            }

            Instruction::RST(val) => {
                let pc_bytes = self.pc.to_be_bytes();
                self.push_stack(pc_bytes[0], pc_bytes[1]);
                self.pc = val as u16;
            }

            Instruction::RLCA => {
                let bit7 = (self.a >> 7) & 1;

                self.a = (self.a << 1) | bit7;

                self.set_flags(false, false, false, bit7 != 0);
            }

            Instruction::RLA => {
                let old_carry = if (self.f & 0x10) != 0 { 1 } else { 0 };
                let bit7 = (self.a >> 7) & 1;

                self.a = (self.a << 1) | old_carry;

                self.set_flags(false, false, false, bit7 != 0);
            }

            Instruction::RRCA => {
                let bit0 = self.a & 0x01;

                self.a = (self.a >> 1) | (bit0 << 7);

                self.set_flags(false, false, false, bit0 != 0);
            }

            Instruction::RRA => {
                let old_carry = if (self.f & 0x10) != 0 { 1 } else { 0 };

                let bit0 = self.a & 0x01;

                self.a = (self.a >> 1) | (old_carry << 7);

                self.set_flags(false, false, false, bit0 != 0);
            }

            Instruction::DAA => {
                let mut a = self.a;
                let mut adjust = 0;
                let mut carry = false;

                if (self.f & 0x20) != 0 || (!(self.f & 0x40) != 0 && (a & 0x0F) > 9) {
                    adjust |= 0x06;
                }

                if (self.f & 0x10) != 0 || (!(self.f & 0x40) != 0 && a > 0x99) {
                    adjust |= 0x60;
                    carry = true;
                }

                if (self.f & 0x40) != 0 {
                    a = a.wrapping_sub(adjust);
                } else {
                    a = a.wrapping_add(adjust);
                }

                let z = a == 0;
                let n = (self.f & 0x40) != 0;
                self.set_flags(z, n, false, carry);
                self.a = a;
            }

            Instruction::CPL => {
                // Bitwise NOT
                self.a = !self.a;

                let z = (self.f & 0x80) != 0;
                let c = (self.f & 0x10) != 0;
                self.set_flags(z, true, true, c);
            }

            Instruction::SCF => {
                let z = (self.f & 0x80) != 0;
                self.set_flags(z, false, false, true);
            }

            Instruction::CCF => {
                let z = (self.f & 0x80) != 0;
                let old_c = (self.f & 0x10) != 0;
                self.set_flags(z, false, false, !old_c);
            }

            Instruction::PREFIX_CB(cb_opcode) => {
                // Get category form bits 7-6
                let category = cb_opcode >> 6;
                // Get bit from bits 5-3
                let bit = (cb_opcode >> 3) & 0x07;
                // Get register from bits 2-0
                let reg_code = cb_opcode & 0x07;
                let reg = self.decode_cb_reg(reg_code);

                match category {
                    0 => self.execute_cb_rotate_shift(bit, reg),
                    1 => {
                        // BIT b, r (0x40 - 0x7F)
                        let val = self.get_reg8(reg);
                        let z = (val & (1 << bit)) == 0;
                        let c = (self.f & 0x10) != 0;
                        self.set_flags(z, false, true, c);
                    }
                    2 => {
                        // RES b, r (0x80 - 0xBF)
                        let val = self.get_reg8(reg);
                        self.set_reg8(reg, val & !(1 << bit));
                    }
                    3 => {
                        // SET b, r (0xC0 - 0xFF)
                        let val = self.get_reg8(reg);
                        self.set_reg8(reg, val | (1 << bit));
                    }
                    _ => unreachable!(),
                }
            }
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
            Reg16::AF => {
                self.a = (val >> 8) as u8;
                self.f = (val & 0xF0) as u8;
            }
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
        }
    }

    // Decode a cb masked instruction
    fn decode_cb_reg(&self, code: u8) -> Reg8 {
        match code {
            0 => Reg8::B,
            1 => Reg8::C,
            2 => Reg8::D,
            3 => Reg8::E,
            4 => Reg8::H,
            5 => Reg8::L,
            6 => Reg8::HLIndirect,
            7 => Reg8::A,
            _ => unreachable!(),
        }
    }

    // Execute rotate and shifts for cb masked instructions
    fn execute_cb_rotate_shift(&mut self, operation: u8, reg: Reg8) {
        let val = self.get_reg8(reg);
        let mut carry = (self.f & 0x10) != 0;

        let result = match operation {
            0 => {
                // RLC
                let bit7 = (val >> 7) & 1;
                carry = bit7 != 0;
                (val << 1) | bit7
            }
            1 => {
                // RRC
                let bit0 = val & 1;
                carry = bit0 != 0;
                (val >> 1) | (bit0 << 7)
            }
            2 => {
                // RL
                let bit7 = (val >> 7) & 1;
                let new_val = (val << 1) | (if carry { 1 } else { 0 });
                carry = bit7 != 0;
                new_val
            }
            3 => {
                // RR
                let bit0 = val & 1;
                let new_val = (val >> 1) | (if carry { 1 } else { 0 } << 7);
                carry = bit0 != 0;
                new_val
            }
            4 => {
                // SLA
                carry = (val >> 7) & 1 != 0;
                val << 1
            }
            5 => {
                // SRA
                carry = val & 1 != 0;
                let bit7 = val & 0x80;
                (val >> 1) | bit7
            }
            6 => {
                // SWAP
                carry = false;
                ((val & 0x0F) << 4) | ((val & 0xF0) >> 4)
            }
            7 => {
                // SRL
                carry = val & 1 != 0;
                val >> 1
            }
            _ => unreachable!(),
        };

        // Flags: SWAP has Z result, others also have Z result.
        // All CB rotations: N=0, H=0, C=new carry.
        let z = result == 0;
        self.set_flags(z, false, false, carry);
        self.set_reg8(reg, result);
    }

    // Push a value in the stack
    fn push_stack(&mut self, high: u8, low: u8) {
        // Decrement SP, then write the high byte
        self.sp = self.sp.wrapping_sub(1);
        self.memory[self.sp as usize] = high;

        // Decrement SP again, then write the low byte
        self.sp = self.sp.wrapping_sub(1);
        self.memory[self.sp as usize] = low;
    }

    // Pop a value from the stack
    fn pop_stack(&mut self) -> (u8, u8) {
        // Increment SP, then read the low byte
        let low = self.read_mem(self.sp);
        self.sp = self.sp.wrapping_add(1);

        // increment SP again, then read the high byte
        let high = self.read_mem(self.sp);
        self.sp = self.sp.wrapping_add(1);

        (low, high)
    }

    // Set flags in the F register
    fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        let mut new_f = 0u8;
        if z {
            new_f |= 0x80;
        }
        if n {
            new_f |= 0x40;
        }
        if h {
            new_f |= 0x20;
        }
        if c {
            new_f |= 0x10;
        }
        self.f = new_f;
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
