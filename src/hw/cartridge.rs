use crate::hw;

/// Emulates an MBC3 cartridge slot
/// `rom` -> ROM memory of the cartridge that is swapped in the banks
/// `ram` -> RAM memory of the cartridge for saving data
/// `rom_bank` -> defines which bank of ROM memory we are working on
/// `ram_bank` -> defines which bank of RAM memory we are working on
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
    pub fn new() -> Self {
        Self {
            rom: vec![hw::U8_ZERO; hw::ROM_SIZE],
            ram: vec![hw::U8_ZERO; hw::VRAM_SIZE],
            rom_bank: 1,
            ram_bank: hw::USIZE_ZERO,
            ram_enabled: false,
        }
    }

    /// Helper to lead the input ROM in memory
    pub fn load(&mut self, buffer: Vec<u8>) {
        // Read byte at address 0x0149 to get RAM size
        if buffer.len() > 0x0149 {
            let ram_size_code = buffer[0x0149];
            // Resize RAM based on found value since the ROM tells how many banks it needs
            let ram_size = match ram_size_code {
                0x01 => hw::KIB_2,
                0x02 => hw::KIB_8,
                0x03 => hw::KIB_32,
                0x04 => hw::KIB_128,
                0x05 => hw::KIB_64,
                _ => hw::KIB_1,
            };
            self.ram = vec![0; ram_size];
        }

        self.rom = buffer;
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            hw::BANK_0_START..=hw::BANK_0_END => self.rom[addr as usize],
            hw::BANK_N_START..=hw::BANK_N_END => {
                let offset = self.rom_bank * hw::BANK_N_START as usize;
                self.rom[offset + (addr - hw::BANK_N_START) as usize]
            }
            hw::EXTERNAL_RAM_START..=hw::EXTERNAL_RAM_END if self.ram_enabled => {
                let offset = self.ram_bank * 0x2000;
                self.ram[offset + (addr - hw::EXTERNAL_RAM_START) as usize]
            }
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            hw::BANK_0_START..=0x1FFF => {
                self.ram_enabled = (val & 0x0F) == 0x0A;
            }
            0x2000..=hw::BANK_0_END => {
                let num_banks = self.rom.len() / hw::BANK_N_START as usize;
                let mut bank = (val & 0x7F) as usize;
                if bank == 0 {
                    bank = 1;
                }
                // Modulo prevents index oob
                self.rom_bank = bank % num_banks;
            }
            hw::BANK_N_START..=0x5FFF if val <= 0x03 => {
                self.ram_bank = val as usize;
            }
            hw::EXTERNAL_RAM_START..=hw::EXTERNAL_RAM_END if self.ram_enabled => {
                let offset = self.ram_bank * 0x2000;
                let physical_addr = offset + (addr - hw::EXTERNAL_RAM_START) as usize;
                if physical_addr < self.ram.len() {
                    self.ram[physical_addr] = val;
                }
            }
            _ => {}
        }
    }
}
