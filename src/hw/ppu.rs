/// Defines the mode in which the [`Ppu`] is operating
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

const VRAM_SIZE: usize = 8192;
pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;

const OAM_SIZE: usize = 160;
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;

pub const LCDC_ADDRESS: u16 = 0xFF40;
pub const STAT_ADDRESS: u16 = 0xFF41;
pub const SCY_ADDRESS: u16 = 0xFF42;
pub const SCX_ADDRESS: u16 = 0xFF43;
pub const LY_ADDRESS: u16 = 0xFF44;
pub const LYC_ADDRESS: u16 = 0xFF45;
pub const BGP_ADDRESS: u16 = 0xFF47;

/// Emulates the PPU of the Gameboy
/// `vram` -> video ram
/// `oam` -> oam
/// `lcdc` -> LCD control 0xFF40
/// `stat` -> LCD status 0xFF41
/// `scy` -> Scroll Y 0xFF42
/// `scx` -> Scroll X 0xFF43
/// `ly` -> Current scanline 0xFF44
/// `lyc` -> LY compart 0xFF45
/// `bgp` -> BG palette 0xFF47
/// `mode` -> execution mode of the [`Ppu`]
/// `ticks` -> performed ticks
/// `frame_buffer` -> screen's pixels
/// `vblank_interrupt` -> signals if [Mode::VBlank] was requested
#[derive(Debug)]
pub struct Ppu {
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    bgp: u8,
    mode: Mode,
    ticks: u32,
    frame_buffer: [u32; SCREEN_WIDTH * SCREEN_HEIGHT],
    vblank_interrupt: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0,
            mode: Mode::OamScan,
            ticks: 0,
            frame_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            vblank_interrupt: false,
        }
    }

    /// Helper to read the vblank interrupt request flag
    pub fn get_interrupt(&self) -> bool {
        self.vblank_interrupt
    }

    /// Helper to set the vblank interrupt request flag
    pub fn set_interrupt(&mut self, flag: bool) {
        self.vblank_interrupt = flag;
    }

    /// Performs a tick for the [`Ppu`]
    pub fn tick(&mut self, cycles: u32) {
        self.ticks += cycles;

        match self.mode {
            Mode::HBlank => {
                if self.ticks >= 204 {
                    self.ticks -= 204;
                    self.ly += 1;

                    if self.ly == 144 {
                        self.mode = Mode::VBlank;
                        self.vblank_interrupt = true;
                    } else {
                        self.mode = Mode::OamScan;
                    }
                }
            }
            Mode::VBlank => {
                if self.ticks >= 456 {
                    self.ticks -= 456;
                    self.ly += 1;

                    if self.ly > 153 {
                        self.ly = 0;
                        self.mode = Mode::OamScan;
                    }
                }
            }
            Mode::OamScan => {
                if self.ticks >= 80 {
                    self.ticks -= 80;
                    self.mode = Mode::Drawing;
                }
            }
            Mode::Drawing => {
                if self.ticks >= 172 {
                    self.ticks -= 172;
                    self.mode = Mode::HBlank;
                    // TODO render screen
                }
            }
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            VRAM_START..=VRAM_END => self.vram[(addr - VRAM_START) as usize],
            OAM_START..=OAM_END => self.oam[(addr - OAM_START) as usize],
            LCDC_ADDRESS => self.lcdc,
            STAT_ADDRESS => self.stat,
            SCY_ADDRESS => self.scy,
            SCX_ADDRESS => self.scx,
            LY_ADDRESS => self.ly,
            LYC_ADDRESS => self.lyc,
            BGP_ADDRESS => self.bgp,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            VRAM_START..=VRAM_END => self.vram[(addr - VRAM_START) as usize] = val,
            OAM_START..=OAM_END => self.oam[(addr - OAM_START) as usize] = val,
            LCDC_ADDRESS => self.lcdc = val,
            STAT_ADDRESS => self.stat = val,
            SCY_ADDRESS => self.scy = val,
            SCX_ADDRESS => self.scx = val,
            LY_ADDRESS => self.ly = 0,
            LYC_ADDRESS => self.lyc = val,
            BGP_ADDRESS => self.bgp = val,
            _ => {}
        }
    }
}
