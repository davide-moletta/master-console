pub const RAM_SIZE: usize = 4 * 8192;

const BANK_0_START: u16 = 0x0000;
const BANK_0_END: u16 = 0x3FFF;

const BANK_N_START: u16 = 0x4000;
const BANK_N_END: u16 = 0x7FFF;

pub const RAM_START: u16 = 0xA000;
pub const RAM_END: u16 = 0xBFFF;

/// Emulates an MBC3 cartridge slot
/// `rom` -> ROM memory of the cartridge that is swapped in the banks
/// `ram` -> RAM memory of the cartridge for saving data
/// `rom_bank` -> defines which bank of memory we are working on
/// `ram_bank` -> defines which bank of memory we are working on
/// `ram_enabled` -> defines if the ram lock is active or not
#[derive(Debug)]
pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
}

impl Cartridge {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            rom: data,
            ram: vec![0; RAM_SIZE],
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            BANK_0_START..=BANK_0_END => self.rom[addr as usize],
            BANK_N_START..=BANK_N_END => {
                let offset = self.rom_bank * BANK_N_START as usize;
                self.rom[offset + (addr - BANK_N_START) as usize]
            }
            RAM_START..=RAM_END if self.ram_enabled => {
                let offset = self.ram_bank * 0x2000;
                self.ram[offset + (addr - RAM_START) as usize]
            }

            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            BANK_0_START..=0x1FFF => {
                self.ram_enabled = (val & 0x0F) == 0x0A;
            }
            0x2000..=BANK_0_END => {
                let num_banks = self.rom.len() / BANK_N_START as usize;
                let mut bank = (val & 0x7F) as usize;
                if bank == 0 {
                    bank = 1;
                }
                // Modulo prevents index oob
                self.rom_bank = bank % num_banks;
            }
            BANK_N_START..=0x5FFF if val <= 0x03 => {
                self.ram_bank = val as usize;
            }
            RAM_START..=RAM_END if self.ram_enabled => {
                let offset = self.ram_bank * 0x2000;
                let physical_addr = offset + (addr - RAM_START) as usize;
                if physical_addr < self.ram.len() {
                    self.ram[physical_addr] = val;
                }
            }
            _ => {}
        }
    }
}
