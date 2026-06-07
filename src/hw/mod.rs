use crate::utils::error::{GBError, GBResult};

pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod joypad;
pub mod opcodes;
pub mod ppu;
pub mod timer;

/*
    GENERICS
*/

pub const USIZE_ZERO: usize = 0;
pub const U8_ZERO: u8 = 0u8;
pub const U16_ZERO: u16 = 0u16;
pub const U32_ZERO: u32 = 0u32;

pub const CYCLE_4: u32 = 4;
pub const CYCLE_8: u32 = 8;
pub const CYCLE_12: u32 = 12;
pub const CYCLE_16: u32 = 16;
pub const CYCLE_20: u32 = 20;
pub const CYCLE_24: u32 = 24;

pub const KIB_1: usize = 1024 * 8; // 1KB
pub const KIB_2: usize = 1024 * 2; // 2KB
pub const KIB_8: usize = 1024 * 8; // 8KB
pub const KIB_32: usize = 1024 * 32; // 32KB
pub const KIB_64: usize = 1024 * 64; // 64KB
pub const KIB_128: usize = 1024 * 128; // 128KB

/*
    MEMORY LAYOUT VALUES
*/

// ROM values
pub const ROM_SIZE: usize = 16 * 1024;
pub const ROM_START: u16 = 0x0000;
pub const ROM_END: u16 = 0x7FFF;

// Video RAM values
pub const VRAM_SIZE: usize = 8 * 1024;
pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;

// External RAM values (Cartridge RAM)
pub const EXTERNAL_RAM_START: u16 = 0xA000;
pub const EXTERNAL_RAM_END: u16 = 0xBFFF;

// Work RAM values
pub const WRAM_SIZE: usize = 8 * 1024;
pub const WRAM_START: u16 = 0xC000;
pub const WRAM_END: u16 = 0xDFFF;

// Echo RAM values
pub const ERAM_START: u16 = 0xE000;
pub const ERAM_END: u16 = 0xFDFF;

// OAM values
pub const OAM_SIZE: usize = 160;
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;

// Priv values
pub const PRIV_START: u16 = 0xFEA0;
pub const PRIV_END: u16 = 0xFEFF;

// I/O registers values
pub const IO_JOYPAD: u16 = 0xFF00;
pub const IO_SERIAL_START: u16 = 0xFF01;
pub const IO_SERIAL_END: u16 = 0xFF02;
pub const IO_TIMER_START: u16 = 0xFF04;
pub const IO_TIMER_END: u16 = 0xFF07;
pub const IF_ADDRESS: u16 = 0xFF0F;
pub const IO_AUDIO_START: u16 = 0xFF10;
pub const IO_AUDIO_END: u16 = 0xFF26;
pub const IO_WAVE_START: u16 = 0xFF30;
pub const IO_WAVE_END: u16 = 0xFF3F;
pub const IO_LCD_START: u16 = 0xFF40;
pub const IO_LCD_END: u16 = 0xFF4B;
pub const IO_OAM_DMA: u16 = 0xFF46;
pub const IO_KEY_0: u16 = 0xFF4C;
pub const IO_KEY_1: u16 = 0xFF4D;
pub const IO_VRAM_BANK_SEL: u16 = 0xFF4F;
pub const IO_ROM_MAPPING: u16 = 0xFF50;
pub const IO_VRAM_DMA_START: u16 = 0xFF51;
pub const IO_VRAM_DMA_END: u16 = 0xFF55;
pub const IO_IR_PORT: u16 = 0xFF56;
pub const IO_PALETTES_START: u16 = 0xFF68;
pub const IO_PALETTES_END: u16 = 0xFF6B;
pub const IO_OBJ_PRIORITY: u16 = 0xFF6C;
pub const IO_WRAM_BANK_SEL: u16 = 0xFF70;

// High RAM values
pub const HRAM_SIZE: usize = 127;
pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;

// Interrupt Enable value
pub const IE_ADDRESS: u16 = 0xFFFF;

// ROM banks values
pub const BANK_0_START: u16 = 0x0000;
pub const BANK_0_END: u16 = 0x3FFF;
pub const BANK_N_START: u16 = 0x4000;
pub const BANK_N_END: u16 = 0x7FFF;

/*
    TIMER VALUES
*/

pub const DIV_ADDRESS: u16 = 0xFF04;
pub const TIMA_ADDRESS: u16 = 0xFF05;
pub const TMA_ADDRESS: u16 = 0xFF06;
pub const TAC_ADDRESS: u16 = 0xFF07;

// Frequency values
pub const FREQUENCY_BIT_4096: u8 = 9;
pub const FREQUENCY_BIT_262144: u8 = 3;
pub const FREQUENCY_BIT_65536: u8 = 5;
pub const FREQUENCY_BIT_16384: u8 = 7;

/*
    SCREEN VALUES
*/

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub const LCDC_ADDRESS: u16 = 0xFF40;
pub const STAT_ADDRESS: u16 = 0xFF41;
pub const SCY_ADDRESS: u16 = 0xFF42;
pub const SCX_ADDRESS: u16 = 0xFF43;
pub const LY_ADDRESS: u16 = 0xFF44;
pub const LYC_ADDRESS: u16 = 0xFF45;
pub const BGP_ADDRESS: u16 = 0xFF47;
pub const OBP0_ADDRESS: u16 = 0xFF48;
pub const OBP1_ADDRESS: u16 = 0xFF49;
pub const WY_ADDRESS: u16 = 0xFF4A;
pub const WX_ADDRESS: u16 = 0xFF4B;

pub const WHITE: u32 = 0xFFFFFFFF;
pub const LGREY: u32 = 0xFFAAAAAA;
pub const DGREY: u32 = 0xFF555555;
pub const BLACK: u32 = 0xFF000000;

// The Game Boy runs at ~59.7 frames per second
// 1 second / 59.7 = ~16,750 microseconds per frame
pub const FRAMERATE: u64 = 16750;
// 154 lines × 456 cycles per line
pub const APPROX_CYCLES_PER_FRAME: u32 = 154 * 456;

/*
    JOYPAD VALUES
*/

/// Buttons for the [`crate::hw::joypad::Joypad`]
pub enum Buttons {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}

impl TryFrom<minifb::Key> for Buttons {
    type Error = GBError;

    fn try_from(value: minifb::Key) -> GBResult<Self> {
        match value {
            minifb::Key::Up | minifb::Key::W => Ok(Buttons::Up),
            minifb::Key::Down | minifb::Key::S => Ok(Buttons::Down),
            minifb::Key::Left | minifb::Key::A => Ok(Buttons::Left),
            minifb::Key::Right | minifb::Key::D => Ok(Buttons::Right),
            minifb::Key::Z | minifb::Key::M => Ok(Buttons::A),
            minifb::Key::X | minifb::Key::L => Ok(Buttons::B),
            minifb::Key::Enter => Ok(Buttons::Start),
            minifb::Key::Backspace => Ok(Buttons::Select),
            _ => Err(GBError::MissingButtonMapping(value)),
        }
    }
}
