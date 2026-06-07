use crate::hw;

/// Timer registers for the cpu
/// `div` -> divider register, always counting
/// `tima` -> timer counter, used to trigger interrupts
/// `tma` -> timer modulo, when `tima`` overflows, it resets to this value
/// `tac -> timer control, sets the speed and turns the timer on/off
/// `interrupt_request` -> signals if there is an interrupt request
#[derive(Debug)]
pub struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,
    interrupt_request: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: hw::U16_ZERO,
            tima: hw::U8_ZERO,
            tma: hw::U8_ZERO,
            tac: hw::U8_ZERO,
            interrupt_request: false,
        }
    }

    /// Helper to read the interrupt request flag
    pub fn get_interrupt(&self) -> bool {
        self.interrupt_request
    }

    /// Helper to set the interrupt request flag
    pub fn set_interrupt(&mut self, flag: bool) {
        self.interrupt_request = flag;
    }

    /// Simulates a tick in the [`hw::cpu::Cpu`]
    pub fn tick(&mut self, cycles: u32) {
        // Increment the div counter
        let old_div = self.div;
        self.div = self.div.wrapping_add(cycles as u16);

        // Check if tima should increment (tac bit 2 is "Timer Enable")
        if (self.tac & 0x04) != 0 {
            // tac bits 0-1 define frequency
            let bit_to_check = match self.tac & 0x03 {
                0x01 => hw::FREQUENCY_BIT_262144,
                0x02 => hw::FREQUENCY_BIT_65536,
                0x03 => hw::FREQUENCY_BIT_16384,
                _ => hw::FREQUENCY_BIT_4096,
            };

            // tima increments when the specific bit in div changes from 1 to 0
            let old_bit = (old_div >> bit_to_check) & 0x01;
            let new_bit = (self.div >> bit_to_check) & 0x01;

            if old_bit == 1 && new_bit == 0 {
                let (new_tima, overflow) = self.tima.overflowing_add(1);
                if overflow {
                    // Reset to Modulo and request interrupt
                    self.tima = self.tma;
                    self.interrupt_request = true;
                } else {
                    self.tima = new_tima;
                }
            }
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            hw::DIV_ADDRESS => (self.div >> 8) as u8,
            hw::TIMA_ADDRESS => self.tima,
            hw::TMA_ADDRESS => self.tma,
            hw::TAC_ADDRESS => self.tac,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            hw::DIV_ADDRESS => self.div = 0,
            hw::TIMA_ADDRESS => self.tima = val,
            hw::TMA_ADDRESS => self.tma = val,
            hw::TAC_ADDRESS => self.tac = val,
            _ => {}
        }
    }
}
