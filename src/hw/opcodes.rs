/// Defines all 8 bit registers
#[derive(Debug, Clone, Copy)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLIndirect,
}

/// Defines all 16 bit registers
#[derive(Debug, Clone, Copy)]
pub enum Reg16 {
    BC,
    DE,
    HL,
    SP,
    AF,
}

/// Represents the conditions used by branching instructions (JP, JR, CALL, RET)
/// These conditions only check the Zero (Z) and Carry (C) flags
#[derive(Debug, Clone, Copy)]
pub enum Condition {
    /// Always jump
    None,
    /// Jump if the Zero flag is 0 (Result was not zero)
    NZ,
    /// Jump if the Zero flag is 1 (Result was zero)
    Z,
    /// Jump if the Carry flag is 0 (No overflow/borrow occurred)
    NC,
    /// Jump if the Carry flag is 1 (Overflow/borrow occurred)
    C,
}

/// Defines all instructions available in the device
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum Instruction {
    // --- Basic instructions ---

    // Only advances the program counter by 1
    NOP,
    // Execution of a STOP instruction stops both the system clock and oscillator circuit
    // STOP mode is entered and the LCD controller also stops
    // However, the status of the internal RAM register ports remains unchanged
    STOP,
    // After a HALT instruction is executed, the system clock is stopped and HALT mode is entered
    // Although the system clock is stopped in this status, the oscillator circuit and LCD controller continue to operate
    HALT,
    // Reset the interrupt master enable (IME) flag and prohibit maskable interrupts
    DI,
    // Set the interrupt master enable (IME) flag and enable maskable interrupts
    // This instruction can be used in an interrupt routine to enable higher-order interrupts
    EI,

    // --- 8-bit load ---

    // Load the contents of register 1 into register 2
    LD_8(Reg8, Reg8),
    // Load the 8-bit immediate operand into register
    LD_8_IMM(Reg8, u8),
    // Load into register A the contents of the internal RAM, port register, or mode register at the address in the range 0xFF00-0xFFFF
    // specified by the 8-bit immediate operand
    LDH_A_IMM(u8),
    // Store the contents of register A in the internal RAM, port register, or mode register at the address in the range 0xFF00-0xFFFF
    // specified by the 8-bit immediate operand
    LDH_IMM_A(u8),
    // Load into register A the contents of the internal RAM, port register, or mode register at the address in the range 0xFF00-0xFFFF
    // specified by register C
    LDH_A_C,
    // Store the contents of register A in the internal RAM, port register, or mode register at the address in the range 0xFF00-0xFFFF
    // specified by register C
    LDH_C_A,
    // Load into register A the contents of the internal RAM or register specified by the 16-bit immediate operand
    LD_A_IMM_16(u16),
    // Store the contents of register A in the internal RAM or register specified by the 16-bit immediate operand
    LD_IMM_16_A(u16),
    // Load the 8-bit contents of memory specified by register pair BC into register A
    LD_A_BC,
    // Load the 8-bit contents of memory specified by register pair DE into register A
    LD_A_DE,
    // Store the contents of register A in the memory location specified by register pair BC
    LD_BC_A,
    // Store the contents of register A in the memory location specified by register pair DE
    LD_DE_A,
    // Load the contents of memory specified by register pair HL into register A, and simultaneously increment the contents of HL
    LD_A_HLI,
    // Load the contents of memory specified by register pair HL into register A, and simultaneously decrement the contents of HL
    LD_A_HLD,
    // Store the contents of register A into the memory location specified by register pair HL, and simultaneously increment the contents of HL
    LD_HLI_A,
    // Store the contents of register A into the memory location specified by register pair HL, and simultaneously decrement the contents of HL
    LD_HLD_A,

    // --- 16-bit Load ---

    // Load the 2 bytes of immediate data into register pair
    LD_16_IMM(Reg16, u16),
    // Store the lower byte of stack pointer at the address specified by the 16-bit immediate operand
    // and store the upper byte of SP at address u16 + 1
    LD_IMM_16_SP(u16),
    // Load the contents of register pair HL into the stack pointer SP
    LD_SP_HL,
    // Add the 8-bit signed operand to the stack pointer SP, and store the result in register pair HL
    LD_HL_SP_e8(i8),
    // Push the contents of register pair onto the memory stack
    PUSH(Reg16),
    // Pop the contents from the memory stack into register pair into register pair
    POP(Reg16),

    // --- 8-bit Arithmetic ---

    // Add the contents of register to the contents of register A, and store the results in register A
    ADD(Reg8),
    // Add the contents of the 8-bit immediate operand to the contents of register A, and store the results in register A
    ADD_IMM(u8),
    // Add the contents of register and the CY flag to the contents of register A, and store the results in register A
    ADC(Reg8),
    // Add the contents of the 8-bit immediate operand and the CY flag to the contents of register A, and store the results in register A
    ADC_IMM(u8),
    // Subtract the contents of register from the contents of register A, and store the results in register A
    SUB(Reg8),
    // Subtract the contents of the 8-bit immediate operand from the contents of register A, and store the results in register A
    SUB_IMM(u8),
    // Subtract the contents of register and the CY flag from the contents of register A, and store the results in register A
    SBC(Reg8),
    // Subtract the contents of the 8-bit immediate operand and the carry flag CY from the contents of register A, and store the results in register A
    SBC_IMM(u8),
    // Take the logical AND for each bit of the contents of register and the contents of register A, and store the results in register A
    AND(Reg8),
    // Take the logical AND for each bit of the contents of 8-bit immediate operand and the contents of register A, and store the results in register A
    AND_IMM(u8),
    // Take the logical exclusive-OR for each bit of the contents of register and the contents of register A, and store the results in register A
    XOR(Reg8),
    // Take the logical exclusive-OR for each bit of the contents of the 8-bit immediate operand and the contents of register A
    // and store the results in register A
    XOR_IMM(u8),
    // Take the logical OR for each bit of the contents of register and the contents of register A, and store the results in register A
    OR(Reg8),
    // Take the logical OR for each bit of the contents of the 8-bit immediate operand and the contents of register A, and store the results in register A
    OR_IMM(u8),
    // Compare the contents of register and the contents of register A by calculating A - B, and set the Z flag if they are equal
    CP(Reg8),
    // Compare the contents of register A and the contents of the 8-bit immediate operand by calculating A - u8, and set the Z flag if they are equal
    CP_IMM(u8),
    // Increment the contents of register by 1
    INC_8(Reg8),
    // Decrement the contents of register by 1
    DEC_8(Reg8),

    // --- 16-bit Arithmetic ---

    // Add the contents of register pair to the contents of register pair HL, and store the results in register pair HL
    ADD_16(Reg16),
    // Add the contents of the 8-bit signed immediate operand and the stack pointer SP and store the results in SP
    ADD_16_SP_e8(i8),
    // Increment the contents of register pair by 1
    INC_16(Reg16),
    // Decrement the contents of register pair by 1
    DEC_16(Reg16),

    // --- Control Flow ---

    // Load the 16-bit immediate operand into the program counter PC if the Condition flag is 0
    // If the Condition flag is 0, then the subsequent instruction starts at address u16
    // If not, the contents of PC are incremented, and the next instruction following the current JP instruction is executed
    JP(Condition, u16),
    // Load the contents of register pair HL into the program counter PC
    // The next instruction is fetched from the location specified by the new value of PC
    JP_HL,
    // If the Condition flag is 1, jump i8 steps from the current address stored in the program counter PC
    // If not, the instruction following the current JP instruction is executed
    JR(Condition, i8),
    // If the Condition flag is 0, the program counter PC value corresponding to the memory location of the instruction
    // following the CALL instruction is pushed to the 2 bytes following the memory byte specified by the stack pointer SP
    // The 16-bit immediate operand u16 is then loaded into PC
    CALL(Condition, u16),
    // If the Condition flag is 0, control is returned to the source program by popping from the memory stack the program counter PC value
    // that was pushed to the stack when the subroutine was called
    RET(Condition),
    // Used when an interrupt-service routine finishes
    // The address for the return from the interrupt is loaded in the program counter PC
    // The master interrupt enable flag is returned to its pre-interrupt status
    RETI,
    // Push the current value of the program counter PC onto the memory stack, and load into PC the 6th byte of page 0 memory addresses, u8
    // The next instruction is fetched from the address specified by the new content of PC
    RST(u8),

    // --- Accumulator/Flags ---

    // Rotate the contents of register A to the left
    // That is, the contents of bit 0 are copied to bit 1, and the previous contents of bit 1 are copied to bit 2
    // The same operation is repeated in sequence for the rest of the register
    // The contents of bit 7 are placed in both the CY flag and bit 0 of register A
    RLCA,
    // Rotate the contents of register A to the left, through the carry (CY) flag
    // That is, the contents of bit 0 are copied to bit 1, and the previous contents of bit 1 are copied to bit 2
    // The same operation is repeated in sequence for the rest of the register
    // The previous contents of the carry flag are copied to bit 0
    RLA,
    // Rotate the contents of register A to the right
    // That is, the contents of bit 7 are copied to bit 6, and the previous contents of bit 6 are copied to bit 5
    // The same operation is repeated in sequence for the rest of the register
    // The contents of bit 0 are placed in both the CY flag and bit 7 of register A
    RRCA,
    // Rotate the contents of register A to the right, through the carry (CY) flag
    // That is, the contents of bit 7 are copied to bit 6, and the previous contents of bit 6 are copied to bit 5
    // The same operation is repeated in sequence for the rest of the register
    // The previous contents of the carry flag are copied to bit 7
    RRA,
    // Adjust the accumulator (register A) too a binary-coded decimal (BCD) number after BCD addition and subtraction operations
    DAA,
    // Take the one's complement of the contents of register A
    CPL,
    // Set the carry flag CY
    SCF,
    // Flip the carry flag CY
    CCF,

    // Opcodes prefixed by 0xCB
    PREFIX_CB(u8),
}

impl Instruction {
    pub fn decode(opcode: u8, reader: &mut impl FnMut() -> u8) -> Self {
        match opcode {
            // --- 0x00 - 0x0F ---
            0x00 => Instruction::NOP,
            0x01 => Instruction::LD_16_IMM(Reg16::BC, Self::read_u16(reader)),
            0x02 => Instruction::LD_BC_A,
            0x03 => Instruction::INC_16(Reg16::BC),
            0x04 => Instruction::INC_8(Reg8::B),
            0x05 => Instruction::DEC_8(Reg8::B),
            0x06 => Instruction::LD_8_IMM(Reg8::B, reader()),
            0x07 => Instruction::RLCA,
            0x08 => Instruction::LD_IMM_16_SP(Self::read_u16(reader)),
            0x09 => Instruction::ADD_16(Reg16::BC),
            0x0A => Instruction::LD_A_BC,
            0x0B => Instruction::DEC_16(Reg16::BC),
            0x0C => Instruction::INC_8(Reg8::C),
            0x0D => Instruction::DEC_8(Reg8::C),
            0x0E => Instruction::LD_8_IMM(Reg8::C, reader()),
            0x0F => Instruction::RRCA,

            // --- 0x10 - 0x1F ---
            0x10 => {
                reader();
                Instruction::STOP
            }
            0x11 => Instruction::LD_16_IMM(Reg16::DE, Self::read_u16(reader)),
            0x12 => Instruction::LD_DE_A,
            0x13 => Instruction::INC_16(Reg16::DE),
            0x14 => Instruction::INC_8(Reg8::D),
            0x15 => Instruction::DEC_8(Reg8::D),
            0x16 => Instruction::LD_8_IMM(Reg8::D, reader()),
            0x17 => Instruction::RLA,
            0x18 => Instruction::JR(Condition::None, reader() as i8),
            0x19 => Instruction::ADD_16(Reg16::DE),
            0x1A => Instruction::LD_A_DE,
            0x1B => Instruction::DEC_16(Reg16::DE),
            0x1C => Instruction::INC_8(Reg8::E),
            0x1D => Instruction::DEC_8(Reg8::E),
            0x1E => Instruction::LD_8_IMM(Reg8::E, reader()),
            0x1F => Instruction::RRA,

            // --- 0x20 - 0x2F ---
            0x20 => Instruction::JR(Condition::NZ, reader() as i8),
            0x21 => Instruction::LD_16_IMM(Reg16::HL, Self::read_u16(reader)),
            0x22 => Instruction::LD_HLI_A,
            0x23 => Instruction::INC_16(Reg16::HL),
            0x24 => Instruction::INC_8(Reg8::H),
            0x25 => Instruction::DEC_8(Reg8::H),
            0x26 => Instruction::LD_8_IMM(Reg8::H, reader()),
            0x27 => Instruction::DAA,
            0x28 => Instruction::JR(Condition::Z, reader() as i8),
            0x29 => Instruction::ADD_16(Reg16::HL),
            0x2A => Instruction::LD_A_HLI,
            0x2B => Instruction::DEC_16(Reg16::HL),
            0x2C => Instruction::INC_8(Reg8::L),
            0x2D => Instruction::DEC_8(Reg8::L),
            0x2E => Instruction::LD_8_IMM(Reg8::L, reader()),
            0x2F => Instruction::CPL,

            // --- 0x30 - 0x3F ---
            0x30 => Instruction::JR(Condition::NC, reader() as i8),
            0x31 => Instruction::LD_16_IMM(Reg16::SP, Self::read_u16(reader)),
            0x32 => Instruction::LD_HLD_A,
            0x33 => Instruction::INC_16(Reg16::SP),
            0x34 => Instruction::INC_8(Reg8::HLIndirect),
            0x35 => Instruction::DEC_8(Reg8::HLIndirect),
            0x36 => Instruction::LD_8_IMM(Reg8::HLIndirect, reader()),
            0x37 => Instruction::SCF,
            0x38 => Instruction::JR(Condition::C, reader() as i8),
            0x39 => Instruction::ADD_16(Reg16::SP),
            0x3A => Instruction::LD_A_HLD,
            0x3B => Instruction::DEC_16(Reg16::SP),
            0x3C => Instruction::INC_8(Reg8::A),
            0x3D => Instruction::DEC_8(Reg8::A),
            0x3E => Instruction::LD_8_IMM(Reg8::A, reader()),
            0x3F => Instruction::CCF,

            // --- 0x40 - 0x7F ---
            0x40..=0x75 | 0x77..=0x7F => {
                Instruction::LD_8(Self::map_reg8(opcode >> 3), Self::map_reg8(opcode))
            }
            0x76 => Instruction::HALT,

            // --- 0x80 - 0xBF ---
            0x80..=0x87 => Instruction::ADD(Self::map_reg8(opcode)),
            0x88..=0x8F => Instruction::ADC(Self::map_reg8(opcode)),
            0x90..=0x97 => Instruction::SUB(Self::map_reg8(opcode)),
            0x98..=0x9F => Instruction::SBC(Self::map_reg8(opcode)),
            0xA0..=0xA7 => Instruction::AND(Self::map_reg8(opcode)),
            0xA8..=0xAF => Instruction::XOR(Self::map_reg8(opcode)),
            0xB0..=0xB7 => Instruction::OR(Self::map_reg8(opcode)),
            0xB8..=0xBF => Instruction::CP(Self::map_reg8(opcode)),

            // --- 0xC0 - 0xCF ---
            0xC0 => Instruction::RET(Condition::NZ),
            0xC1 => Instruction::POP(Reg16::BC),
            0xC2 => Instruction::JP(Condition::NZ, Self::read_u16(reader)),
            0xC3 => Instruction::JP(Condition::None, Self::read_u16(reader)),
            0xC4 => Instruction::CALL(Condition::NZ, Self::read_u16(reader)),
            0xC5 => Instruction::PUSH(Reg16::BC),
            0xC6 => Instruction::ADD_IMM(reader()),
            0xC7 => Instruction::RST(0x00),
            0xC8 => Instruction::RET(Condition::Z),
            0xC9 => Instruction::RET(Condition::None),
            0xCA => Instruction::JP(Condition::Z, Self::read_u16(reader)),
            0xCB => Instruction::PREFIX_CB(reader()),
            0xCC => Instruction::CALL(Condition::Z, Self::read_u16(reader)),
            0xCD => Instruction::CALL(Condition::None, Self::read_u16(reader)),
            0xCE => Instruction::ADC_IMM(reader()),
            0xCF => Instruction::RST(0x08),

            // --- 0xD0 - 0xDF ---
            0xD0 => Instruction::RET(Condition::NC),
            0xD1 => Instruction::POP(Reg16::DE),
            0xD2 => Instruction::JP(Condition::NC, Self::read_u16(reader)),
            0xD4 => Instruction::CALL(Condition::NC, Self::read_u16(reader)),
            0xD5 => Instruction::PUSH(Reg16::DE),
            0xD6 => Instruction::SUB_IMM(reader()),
            0xD7 => Instruction::RST(0x10),
            0xD8 => Instruction::RET(Condition::C),
            0xD9 => Instruction::RETI,
            0xDA => Instruction::JP(Condition::C, Self::read_u16(reader)),
            0xDC => Instruction::CALL(Condition::C, Self::read_u16(reader)),
            0xDE => Instruction::SBC_IMM(reader()),
            0xDF => Instruction::RST(0x18),

            // --- 0xE0 - 0xEF ---
            0xE0 => Instruction::LDH_IMM_A(reader()),
            0xE1 => Instruction::POP(Reg16::HL),
            0xE2 => Instruction::LDH_C_A,
            0xE5 => Instruction::PUSH(Reg16::HL),
            0xE6 => Instruction::AND_IMM(reader()),
            0xE7 => Instruction::RST(0x20),
            0xE8 => Instruction::ADD_16_SP_e8(reader() as i8),
            0xE9 => Instruction::JP_HL,
            0xEA => Instruction::LD_IMM_16_A(Self::read_u16(reader)),
            0xEE => Instruction::XOR_IMM(reader()),
            0xEF => Instruction::RST(0x28),

            // --- 0xF0 - 0xFF ---
            0xF0 => Instruction::LDH_A_IMM(reader()),
            0xF1 => Instruction::POP(Reg16::AF),
            0xF2 => Instruction::LDH_A_C,
            0xF3 => Instruction::DI,
            0xF5 => Instruction::PUSH(Reg16::AF),
            0xF6 => Instruction::OR_IMM(reader()),
            0xF7 => Instruction::RST(0x30),
            0xF8 => Instruction::LD_HL_SP_e8(reader() as i8),
            0xF9 => Instruction::LD_SP_HL,
            0xFA => Instruction::LD_A_IMM_16(Self::read_u16(reader)),
            0xFB => Instruction::EI,
            0xFE => Instruction::CP_IMM(reader()),
            0xFF => Instruction::RST(0x38),

            // Unused/Illegal Opcodes on original hardware
            _ => Instruction::NOP,
        }
    }

    // Maps a u8 to a 1B register
    fn map_reg8(bits: u8) -> Reg8 {
        match bits & 0b111 {
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

    // Reads a u16 from the device's memory
    fn read_u16(reader: &mut impl FnMut() -> u8) -> u16 {
        let low = reader() as u16;
        let high = reader() as u16;
        (high << 8) | low
    }
}
