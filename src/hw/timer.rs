/// DIV ($FF04): Divider Register. Always counting.
/// TIMA ($FF05): Timer Counter. The one that triggers interrupts.
/// TMA ($FF06): Timer Modulo. When TIMA overflows, it resets to this value.
/// TAC ($FF07): Timer Control. Sets the speed and turns the timer on/off.
pub struct Timer {
    div_internal: u16, // Internal 16-bit counter
    tima: u8,
    tma: u8,
    tac: u8,
    pub interrupt_request: bool,
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

    pub fn tick(&mut self, cycles: u32) {
        // 1. Always increment the internal DIV counter
        let old_div_internal = self.div_internal;
        self.div_internal = self.div_internal.wrapping_add(cycles as u16);

        // 2. Check if TIMA should increment (TAC bit 2 is "Timer Enable")
        if (self.tac & 0x04) != 0 {
            // TAC bits 0-1 define frequency:
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

            // Logic: TIMA increments when the specific bit in internal DIV
            // changes from 1 to 0 (falling edge)
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

    // Helpers for Bus to read/write registers
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.div_internal >> 8) as u8, // Top 8 bits is the DIV register
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF04 => self.div_internal = 0, // Any write to DIV resets it to 0
            0xFF05 => self.tima = val,
            0xFF06 => self.tma = val,
            0xFF07 => self.tac = val,
            _ => {}
        }
    }
}
