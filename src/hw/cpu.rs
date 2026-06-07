use log::warn;
use std::{fmt, path::PathBuf};

use crate::{
    hw::{
        self, bus,
        opcodes::{Condition, Instruction, Reg16, Reg8},
    },
    utils::error::GBResult,
};

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
/// `memory` -> bus memory
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
    bus: bus::Bus,
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
    pub fn new(rom_path: PathBuf) -> Self {
        let mut bus = bus::Bus::new();
        bus.load_rom(rom_path).expect("Failed to load ROM");

        Self {
            pc: 0x100,
            // TODO this is the setup after boot rom, add real in the future
            sp: 0xFFFE, // Standard stack pointer start
            a: 0x01,    // Standard boot value
            f: 0xB0,    // Standard boot value
            b: 0x00,    // Standard boot value
            c: 0x13,    // Standard boot value
            d: 0x00,    // Standard boot value
            e: 0xD8,    // Standard boot value
            h: 0x01,    // Standard boot value
            l: 0x4D,    // Standard boot value
            ime: false,
            halted: false,
            bus,
        }
    }

    /// Performs a CPU step
    pub fn step(&mut self) -> u32 {
        // Handle Interrupts
        let interrupt_cycles = self.handle_interrupts();

        // Handle HALT state
        if self.halted {
            // While halted, the CPU "idles" for 4 cycles at a time
            // until an interrupt is triggered
            self.bus.tick(4);
            return 4 + interrupt_cycles;
        }

        // Fetch opcode
        let opcode = self.bus.read(self.pc);
        self.pc += 1;

        // Decode the instruction read from memory
        // This gives an enclosure s.t. if the decode function needs to read more that it can do so
        let instr = Instruction::decode(opcode, &mut || {
            let val = self.bus.read(self.pc);
            self.pc += 1;
            val
        });

        // debug!("Performing instruction: {:?} - CPU state: {}", instr, self);

        // Execute the decoded instruction
        let cycles = self.execute(instr);
        self.bus.tick(cycles);

        cycles + interrupt_cycles
    }

    /// Handle interrupts
    fn handle_interrupts(&mut self) -> u32 {
        // Get interrupt enable and interrupt flag
        let ie_reg = self.bus.read(hw::IE_ADDRESS);
        let if_reg = self.bus.read(hw::IF_ADDRESS);

        if if_reg == hw::U8_ZERO {
            return 0;
        }

        // Any pending interrupt wakes the CPU, even if IME is disabled
        if self.halted {
            self.halted = false;
        }

        // If Interrupt Master Enable is off, don't jump to handlers
        if !self.ime {
            return 0;
        }

        // Check bits 0-4 for VBlank, LCD Stat, Timer, Serial, Joypad
        for i in 0..5 {
            // Check if both interrupt is enabled (IE) and pending (IF)
            if ((ie_reg & (1 << i)) != 0) && ((if_reg & (1 << i)) != 0) {
                // Disable interrupts while handling one
                self.ime = false;

                // Clear the flag in the IF register
                let if_reg = self.bus.read(hw::IF_ADDRESS);
                self.bus.write(hw::IF_ADDRESS, if_reg & !(1 << i));

                let pc_bytes = self.pc.to_be_bytes();
                self.push_stack(pc_bytes[0], pc_bytes[1]);
                self.pc = 0x0040 + (i as u16 * 8);
                return 20;
            }
        }

        0
    }

    /// Simulate the decoded instruction performing the same operation
    /// Returns the number of cycles used
    fn execute(&mut self, instr: Instruction) -> u32 {
        match instr {
            Instruction::NOP => hw::CYCLE_4,

            Instruction::STOP => {
                warn!("STOP executed, entering deep sleep");
                // TODO for now work like HALT
                self.halted = true;
                hw::CYCLE_4
            }

            Instruction::HALT => {
                self.halted = true;
                hw::CYCLE_4
            }

            Instruction::DI => {
                self.ime = false;
                hw::CYCLE_4
            }

            Instruction::EI => {
                self.ime = true;
                hw::CYCLE_4
            }

            Instruction::LD_8(dst, src) => {
                let val = self.get_reg8(src);
                self.set_reg8(dst, val);
                if dst == Reg8::HLIndirect || src == Reg8::HLIndirect {
                    hw::CYCLE_8
                } else {
                    hw::CYCLE_4
                }
            }

            Instruction::LD_8_IMM(dst, val) => {
                self.set_reg8(dst, val);
                if dst == Reg8::HLIndirect {
                    hw::CYCLE_12
                } else {
                    hw::CYCLE_8
                }
            }

            // hw::IO_JOYPAD is the start of the I/O registers
            Instruction::LDH_A_IMM(val) => {
                let address = hw::IO_JOYPAD | (val as u16);
                self.a = self.bus.read(address);
                hw::CYCLE_12
            }

            Instruction::LDH_IMM_A(val) => {
                let address = hw::IO_JOYPAD | (val as u16);
                self.bus.write(address, self.a);
                hw::CYCLE_12
            }

            Instruction::LDH_A_C => {
                let address = hw::IO_JOYPAD | (self.c as u16);
                self.a = self.bus.read(address);
                hw::CYCLE_8
            }

            Instruction::LDH_C_A => {
                let address = hw::IO_JOYPAD | (self.c as u16);
                self.bus.write(address, self.a);
                hw::CYCLE_8
            }

            Instruction::LD_A_IMM_16(val) => {
                self.a = self.bus.read(val);
                hw::CYCLE_16
            }

            Instruction::LD_IMM_16_A(val) => {
                self.bus.write(val, self.a);
                hw::CYCLE_16
            }

            Instruction::LD_A_BC => {
                let address = self.get_reg16(Reg16::BC);
                self.a = self.bus.read(address);
                hw::CYCLE_8
            }

            Instruction::LD_A_DE => {
                let address = self.get_reg16(Reg16::DE);
                self.a = self.bus.read(address);
                hw::CYCLE_8
            }

            Instruction::LD_BC_A => {
                let address = self.get_reg16(Reg16::BC);
                self.bus.write(address, self.a);
                hw::CYCLE_8
            }

            Instruction::LD_DE_A => {
                let address = self.get_reg16(Reg16::DE);
                self.bus.write(address, self.a);
                hw::CYCLE_8
            }

            Instruction::LD_A_HLI => {
                let hl = self.get_reg16(Reg16::HL);
                self.a = self.bus.read(hl);
                self.set_reg16(Reg16::HL, hl.wrapping_add(1));
                hw::CYCLE_8
            }

            Instruction::LD_A_HLD => {
                let hl = self.get_reg16(Reg16::HL);
                self.a = self.bus.read(hl);
                self.set_reg16(Reg16::HL, hl.wrapping_sub(1));
                hw::CYCLE_8
            }

            Instruction::LD_HLI_A => {
                let hl = self.get_reg16(Reg16::HL);
                self.set_reg16(Reg16::HL, hl.wrapping_add(1));
                self.bus.write(hl, self.a);
                hw::CYCLE_8
            }

            Instruction::LD_HLD_A => {
                let hl = self.get_reg16(Reg16::HL);
                self.set_reg16(Reg16::HL, hl.wrapping_sub(1));
                self.bus.write(hl, self.a);
                hw::CYCLE_8
            }

            Instruction::LD_16_IMM(dst, val) => {
                self.set_reg16(dst, val);
                hw::CYCLE_12
            }

            Instruction::LD_IMM_16_SP(val) => {
                let sp_val = self.sp;

                // Extract low byte (bits 0-7) and high byte (bits 8-15)
                let low_byte = (sp_val & 0xFF) as u8;
                let high_byte = (sp_val >> 8) as u8;

                self.bus.write(val, low_byte);
                self.bus.write(val.wrapping_add(1), high_byte);
                hw::CYCLE_20
            }

            Instruction::LD_SP_HL => {
                self.sp = self.get_reg16(Reg16::HL);
                hw::CYCLE_8
            }

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
                hw::CYCLE_12
            }

            Instruction::PUSH(src) => {
                let val = self.get_reg16(src);
                self.push_stack((val >> 8) as u8, (val & 0xFF) as u8);
                hw::CYCLE_16
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
                hw::CYCLE_12
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
                if src == Reg8::HLIndirect {
                    hw::CYCLE_8
                } else {
                    hw::CYCLE_4
                }
            }

            Instruction::ADD_IMM(val) => {
                let result = self.a.wrapping_add(val);

                let z = result == 0;
                let n = false;
                let h = (self.a & 0x0F) + (val & 0x0F) > 0x0F;
                let c = (self.a as u16) + (val as u16) > 0xFF;

                self.set_flags(z, n, h, c);
                self.a = result;
                hw::CYCLE_8
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
                if src == Reg8::HLIndirect {
                    hw::CYCLE_8
                } else {
                    hw::CYCLE_4
                }
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
                hw::CYCLE_8
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
                if src == Reg8::HLIndirect {
                    hw::CYCLE_8
                } else {
                    hw::CYCLE_4
                }
            }

            Instruction::SUB_IMM(val) => {
                let result = self.a.wrapping_sub(val);

                let z = result == 0;
                let n = true;
                let h = (self.a & 0x0F) < (val & 0x0F);
                let c = self.a < val;

                self.set_flags(z, n, h, c);
                self.a = result;
                hw::CYCLE_8
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
                if src == Reg8::HLIndirect {
                    hw::CYCLE_8
                } else {
                    hw::CYCLE_4
                }
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
                hw::CYCLE_8
            }

            Instruction::AND(src) => {
                let val = self.get_reg8(src);
                self.a &= val;

                let z = self.a == 0;
                self.set_flags(z, false, true, false);
                if src == Reg8::HLIndirect {
                    hw::CYCLE_8
                } else {
                    hw::CYCLE_4
                }
            }

            Instruction::AND_IMM(val) => {
                self.a &= val;

                let z = self.a == 0;
                self.set_flags(z, false, true, false);
                hw::CYCLE_8
            }

            Instruction::XOR(src) => {
                let val = self.get_reg8(src);
                self.a ^= val;

                let z = self.a == 0;
                self.set_flags(z, false, false, false);
                if src == Reg8::HLIndirect {
                    hw::CYCLE_8
                } else {
                    hw::CYCLE_4
                }
            }

            Instruction::XOR_IMM(val) => {
                self.a ^= val;

                let z = self.a == 0;
                self.set_flags(z, false, false, false);
                hw::CYCLE_8
            }

            Instruction::OR(src) => {
                let val = self.get_reg8(src);
                self.a |= val;

                let z = self.a == 0;
                self.set_flags(z, false, false, false);
                if src == Reg8::HLIndirect {
                    hw::CYCLE_8
                } else {
                    hw::CYCLE_4
                }
            }

            Instruction::OR_IMM(val) => {
                self.a |= val;

                let z = self.a == 0;
                self.set_flags(z, false, false, false);
                hw::CYCLE_8
            }

            Instruction::CP(src) => {
                let val = self.get_reg8(src);
                let result = self.a.wrapping_sub(val);

                let z = result == 0;
                let n = true;
                let h = (self.a & 0x0F) < (val & 0x0F);
                let c = self.a < val;

                self.set_flags(z, n, h, c);
                if src == Reg8::HLIndirect {
                    hw::CYCLE_8
                } else {
                    hw::CYCLE_4
                }
            }

            Instruction::CP_IMM(val) => {
                let result = self.a.wrapping_sub(val);

                let z = result == 0;
                let n = true;
                let h = (self.a & 0x0F) < (val & 0x0F);
                let c = self.a < val;

                self.set_flags(z, n, h, c);
                hw::CYCLE_8
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
                if reg == Reg8::HLIndirect {
                    hw::CYCLE_12
                } else {
                    hw::CYCLE_4
                }
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
                if reg == Reg8::HLIndirect {
                    hw::CYCLE_12
                } else {
                    hw::CYCLE_4
                }
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
                hw::CYCLE_8
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
                hw::CYCLE_16
            }

            Instruction::INC_16(reg) => {
                let val = self.get_reg16(reg);
                self.set_reg16(reg, val.wrapping_add(1));
                hw::CYCLE_8
            }

            Instruction::DEC_16(reg) => {
                let val = self.get_reg16(reg);
                self.set_reg16(reg, val.wrapping_sub(1));
                hw::CYCLE_8
            }

            Instruction::JP(cond, val) => {
                if self.check_condition(cond) {
                    self.pc = val;
                    hw::CYCLE_16
                } else {
                    hw::CYCLE_12
                }
            }

            Instruction::JP_HL => {
                self.pc = self.get_reg16(Reg16::HL);
                hw::CYCLE_4
            }

            Instruction::JR(cond, offset) => {
                if self.check_condition(cond) {
                    self.pc = (self.pc as i16 + offset as i16) as u16;
                    hw::CYCLE_12
                } else {
                    hw::CYCLE_8
                }
            }

            Instruction::CALL(cond, val) => {
                if self.check_condition(cond) {
                    let pc_bytes = self.pc.to_be_bytes();
                    self.push_stack(pc_bytes[0], pc_bytes[1]);
                    self.pc = val;
                    hw::CYCLE_24
                } else {
                    hw::CYCLE_12
                }
            }

            Instruction::RET(cond) => {
                if self.check_condition(cond) {
                    let (low, high) = self.pop_stack();
                    self.pc = ((high as u16) << 8) | (low as u16);
                    // Special case: Conditional RET takes 20 cycles if taken, but RET (None) is always 16
                    if cond == Condition::None {
                        hw::CYCLE_16
                    } else {
                        hw::CYCLE_20
                    }
                } else {
                    hw::CYCLE_8
                }
            }

            Instruction::RETI => {
                let (low, high) = self.pop_stack();
                self.pc = ((high as u16) << 8) | (low as u16);
                self.ime = true;
                hw::CYCLE_16
            }

            Instruction::RST(val) => {
                let pc_bytes = self.pc.to_be_bytes();
                self.push_stack(pc_bytes[0], pc_bytes[1]);
                self.pc = val as u16;
                hw::CYCLE_16
            }

            Instruction::RLCA => {
                let bit7 = (self.a >> 7) & 1;
                self.a = (self.a << 1) | bit7;
                self.set_flags(false, false, false, bit7 != 0);
                hw::CYCLE_4
            }

            Instruction::RLA => {
                let old_carry = if (self.f & 0x10) != 0 { 1 } else { 0 };
                let bit7 = (self.a >> 7) & 1;
                self.a = (self.a << 1) | old_carry;
                self.set_flags(false, false, false, bit7 != 0);
                hw::CYCLE_4
            }

            Instruction::RRCA => {
                let bit0 = self.a & 0x01;
                self.a = (self.a >> 1) | (bit0 << 7);
                self.set_flags(false, false, false, bit0 != 0);
                hw::CYCLE_4
            }

            Instruction::RRA => {
                let old_carry = if (self.f & 0x10) != 0 { 1 } else { 0 };
                let bit0 = self.a & 0x01;
                self.a = (self.a >> 1) | (old_carry << 7);
                self.set_flags(false, false, false, bit0 != 0);
                hw::CYCLE_4
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
                hw::CYCLE_4
            }

            Instruction::CPL => {
                self.a = !self.a;
                let z = (self.f & 0x80) != 0;
                let c = (self.f & 0x10) != 0;
                self.set_flags(z, true, true, c);
                hw::CYCLE_4
            }

            Instruction::SCF => {
                let z = (self.f & 0x80) != 0;
                self.set_flags(z, false, false, true);
                hw::CYCLE_4
            }

            Instruction::CCF => {
                let z = (self.f & 0x80) != 0;
                let old_c = (self.f & 0x10) != 0;
                self.set_flags(z, false, false, !old_c);
                hw::CYCLE_4
            }

            Instruction::PREFIX_CB(cb_opcode) => {
                let category = cb_opcode >> 6;
                let bit = (cb_opcode >> 3) & 0x07;
                let reg_code = cb_opcode & 0x07;
                let reg = self.decode_cb_reg(reg_code);

                match category {
                    0 => self.execute_cb_rotate_shift(bit, reg),
                    1 => {
                        let val = self.get_reg8(reg);
                        let z = (val & (1 << bit)) == 0;
                        let c = (self.f & 0x10) != 0;
                        self.set_flags(z, false, true, c);
                    }
                    2 => {
                        let val = self.get_reg8(reg);
                        self.set_reg8(reg, val & !(1 << bit));
                    }
                    3 => {
                        let val = self.get_reg8(reg);
                        self.set_reg8(reg, val | (1 << bit));
                    }
                    _ => unreachable!(),
                }
                // CB instructions take 8 cycles if register, 16 if [HL]
                if reg == Reg8::HLIndirect {
                    hw::CYCLE_16
                } else {
                    hw::CYCLE_8
                }
            }
        }
    }

    /// Read value stored in the specified [`Reg8`] register
    fn get_reg8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::H => self.h,
            Reg8::L => self.l,
            Reg8::HLIndirect => self.bus.read(self.get_reg16(Reg16::HL)),
        }
    }

    /// Set value in the specified [`Reg8`] register
    fn set_reg8(&mut self, reg: Reg8, val: u8) {
        match reg {
            Reg8::A => self.a = val,
            Reg8::B => self.b = val,
            Reg8::C => self.c = val,
            Reg8::D => self.d = val,
            Reg8::E => self.e = val,
            Reg8::H => self.h = val,
            Reg8::L => self.l = val,
            Reg8::HLIndirect => self.bus.write(self.get_reg16(Reg16::HL), val),
        }
    }

    /// Read value stored in the specified [`Reg16`] register
    fn get_reg16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::AF => ((self.a as u16) << 8) | (self.f as u16),
            Reg16::BC => ((self.b as u16) << 8) | (self.c as u16),
            Reg16::DE => ((self.d as u16) << 8) | (self.e as u16),
            Reg16::HL => ((self.h as u16) << 8) | (self.l as u16),
            Reg16::SP => self.sp,
        }
    }

    /// Set value in the specified [`Reg16`] register
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

    /// Decode a cb masked instruction
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

    /// Execute rotate and shifts for cb masked instructions
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

        // All CB rotations: N=0, H=0, C=new carry
        let z = result == 0;
        self.set_flags(z, false, false, carry);
        self.set_reg8(reg, result);
    }

    /// Push a value in the stack
    fn push_stack(&mut self, high: u8, low: u8) {
        // Decrement SP, then write the high byte
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write(self.sp, high);

        // Decrement SP again, then write the low byte
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write(self.sp, low);
    }

    /// Pop a value from the stack
    fn pop_stack(&mut self) -> (u8, u8) {
        // Increment SP, then read the low byte
        let low = self.bus.read(self.sp);
        self.sp = self.sp.wrapping_add(1);

        // increment SP again, then read the high byte
        let high = self.bus.read(self.sp);
        self.sp = self.sp.wrapping_add(1);

        (low, high)
    }

    /// Set flags in the F register
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

    /// Helper to set the specified button in [`hw::joypad::Joypad`]
    pub fn set_button(&mut self, key: minifb::Key) -> GBResult<()> {
        self.bus.set_button(hw::Buttons::try_from(key)?);
        Ok(())
    }

    /// Helper to unset buttons in [`hw::joypad::Joypad`]
    pub fn unset_buttons(&mut self) {
        self.bus.unset_buttons()
    }

    /// Helper to read the frame buffer
    pub fn get_frame(&self) -> [u32; hw::SCREEN_WIDTH * hw::SCREEN_HEIGHT] {
        self.bus.get_frame()
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
