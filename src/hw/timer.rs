const DIV_ADDRESS: u16 = 0xFF04;
const TIMA_ADDRESS: u16 = 0xFF05;
const TMA_ADDRESS: u16 = 0xFF06;
const TAC_ADDRESS: u16 = 0xFF07;

/// Timer registers for the cpu
/// `div_internal` -> divider register, always counting
/// `tima` -> timer counter, used to trigger interrupts
/// `tma` -> timer modulo, when `tima`` overflows, it resets to this value
/// `tac -> timer control, sets the speed and turns the timer on/off
/// `interrupt_request` -> signals if there is an interrupt request
#[derive(Debug)]
pub struct Timer {
    div_internal: u16,
    tima: u8,
    tma: u8,
    tac: u8,
    interrupt_request: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div_internal: 0,
            tima: 0,
            tma: 0,
            tac: 0,
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

    /// Simulates a tick in the [`crate::hw::cpu::Cpu`]
    pub fn tick(&mut self, cycles: u32) {
        // Increment the `div_internal` counter
        let old_div_internal = self.div_internal;
        self.div_internal = self.div_internal.wrapping_add(cycles as u16);

        // Check if `tima` should increment (`tac` bit 2 is "Timer Enable")
        if (self.tac & 0x04) != 0 {
            // `tac` bits 0-1 define frequency:
            // 00: 4096 Hz   (Internal bit 9)
            // 01: 262144 Hz (Internal bit 3)
            // 10: 65536 Hz  (Internal bit 5)
            // 11: 16384 Hz  (Internal bit 7)
            let bit_to_check = match self.tac & 0x03 {
                0x01 => 3,
                0x02 => 5,
                0x03 => 7,
                _ => 9,
            };

            // `tima` increments when the specific bit in `div_internal` changes from 1 to 0
            let old_bit = (old_div_internal >> bit_to_check) & 0x01;
            let new_bit = (self.div_internal >> bit_to_check) & 0x01;

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

    /// Helper for [`crate::hw::bus::Bus`] to read [`Timer`] registers
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            DIV_ADDRESS => (self.div_internal >> 8) as u8,
            TIMA_ADDRESS => self.tima,
            TMA_ADDRESS => self.tma,
            TAC_ADDRESS => self.tac,
            _ => 0xFF,
        }
    }

    /// Helper for [`crate::hw::bus::Bus`] to write [`Timer`] registers
    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            DIV_ADDRESS => self.div_internal = 0,
            TIMA_ADDRESS => self.tima = val,
            TMA_ADDRESS => self.tma = val,
            TAC_ADDRESS => self.tac = val,
            _ => {}
        }
    }
}
