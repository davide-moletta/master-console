/// Defines the mode in which the [`Ppu`] is operating
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

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
pub const OBP0_ADDRESS: u16 = 0xFF48;
pub const OBP1_ADDRESS: u16 = 0xFF49;

const WHITE: u32 = 0xFFFFFFFF;
const LGREY: u32 = 0xFFAAAAAA;
const DGREY: u32 = 0xFF555555;
const BLACK: u32 = 0xFF000000;

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
/// `obp0` & `obp1` -> palette registers for sprites
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
    obp0: u8,
    obp1: u8,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            // TODO this is the setup after boot rom, add real in the future
            lcdc: 0x91, // LCD Enabled, BG Enabled
            stat: 0x85, // Mode 1 (starting in VBlank)
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC, // Standard palette (11 11 11 00)

            mode: Mode::OamScan,
            ticks: 0,
            frame_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            vblank_interrupt: false,
            obp0: 0,
            obp1: 0,
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

    /// Helper to read the frame buffer
    pub fn get_frame(&self) -> [u32; SCREEN_WIDTH * SCREEN_HEIGHT] {
        self.frame_buffer
    }

    /// Helper to correctly set the [`Mode`]
    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.stat = (self.stat & 0xFC) | (mode as u8 & 0x03);
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
                        self.set_mode(Mode::VBlank);
                        self.vblank_interrupt = true;
                    } else {
                        self.set_mode(Mode::OamScan);
                    }
                }
            }
            Mode::VBlank => {
                if self.ticks >= 456 {
                    self.ticks -= 456;
                    self.ly += 1;

                    if self.ly > 153 {
                        self.ly = 0;
                        self.set_mode(Mode::OamScan);
                    }
                }
            }
            Mode::OamScan => {
                if self.ticks >= 80 {
                    self.ticks -= 80;
                    self.set_mode(Mode::Drawing);
                }
            }
            Mode::Drawing => {
                if self.ticks >= 172 {
                    self.ticks -= 172;
                    self.set_mode(Mode::HBlank);
                    if self.ly < 144 {
                        self.render_scanline();
                        self.render_sprites();
                    }
                }
            }
        }
    }

    /// Helper to get the shade of gray
    fn get_color(&self, color_id: u8) -> u32 {
        let palette_index = (self.bgp >> (color_id * 2)) & 0x03;

        match palette_index {
            0 => WHITE,
            1 => LGREY,
            2 => DGREY,
            _ => BLACK,
        }
    }

    /// Helper to get the color for sprites
    fn get_sprite_color(&self, color_id: u8, palette: u8) -> u32 {
        // Color 0 is transparent
        if color_id == 0 {
            return 0;
        }

        let palette_index = (palette >> (color_id * 2)) & 0x03;
        match palette_index {
            0 => WHITE,
            1 => LGREY,
            2 => DGREY,
            _ => BLACK,
        }
    }

    /// Renders a line for the display
    pub fn render_scanline(&mut self) {
        if (self.lcdc & 0x01) == 0 {
            return;
        }

        let tile_map_base: u16 = if (self.lcdc & 0x08) != 0 {
            0x9C00
        } else {
            0x9800
        };
        let use_unsigned = (self.lcdc & 0x10) != 0;

        let y_pos = self.ly.wrapping_add(self.scy);
        let tile_y = (y_pos / 8) as u16;
        let pixel_y_in_tile = y_pos % 8;

        for x in 0..SCREEN_WIDTH as u8 {
            let x_pos = x.wrapping_add(self.scx);
            let tile_x = (x_pos / 8) as u16;
            let pixel_x_in_tile = x_pos % 8;

            // Get Tile ID directly from vram array
            let map_index = (tile_map_base + (tile_y * 32) + tile_x) - 0x8000;
            let tile_id = self.vram[map_index as usize];

            // Calculate Tile Data Address
            let tile_data_addr: u16 = if use_unsigned {
                0x8000 + (tile_id as u16 * 16)
            } else {
                if tile_id < 128 {
                    0x9000 + (tile_id as u16 * 16)
                } else {
                    0x8800 + ((tile_id - 128) as u16 * 16)
                }
            };

            // Calculate "Relative" index for the pixel data
            let vram_index = (tile_data_addr - 0x8000) + (pixel_y_in_tile as u16 * 2);

            // Fetch bytes directly from array
            let byte1 = self.vram[vram_index as usize];
            let byte2 = self.vram[(vram_index + 1) as usize];

            let bit = 7 - pixel_x_in_tile;
            let lsb = (byte1 >> bit) & 0x01;
            let msb = (byte2 >> bit) & 0x01;
            let color_id = (msb << 1) | lsb;

            let color = self.get_color(color_id);
            let buffer_idx = (self.ly as usize * SCREEN_WIDTH) + x as usize;
            self.frame_buffer[buffer_idx] = color;
        }
    }

    /// Renders sprites
    pub fn render_sprites(&mut self) {
        // Bit 1 of LCDC enables sprites
        if (self.lcdc & 0x02) == 0 {
            return;
        }

        // Loop through 40 sprites in OAM
        for i in 0..40 {
            let i = i * 4;

            // Sprite Y and X are offset by 16 and 8 on real hardware
            let sprite_y = self.oam[i] as i16 - 16;
            let sprite_x = self.oam[i + 1] as i16 - 8;
            let tile_index = self.oam[i + 2];
            let attributes = self.oam[i + 3];

            // Check if the sprite is on the current scanline (LY)
            if (self.ly as i16) >= sprite_y && (self.ly as i16) < (sprite_y + 8) {
                let pixel_y = (self.ly as i16 - sprite_y) as u16;

                let final_pixel_y = if (attributes & 0x40) != 0 {
                    7 - pixel_y
                } else {
                    pixel_y
                };

                let tile_data_addr = 0x8000 + (tile_index as u16 * 16);
                let byte1 = self.read(tile_data_addr + (final_pixel_y * 2));
                let byte2 = self.read(tile_data_addr + (final_pixel_y * 2) + 1);

                for x in 0..8 {
                    let pixel_x = sprite_x + x as i16;

                    // Ignore if pixel is off-screen
                    if !(0..160).contains(&pixel_x) {
                        continue;
                    }

                    let bit = if (attributes & 0x20) != 0 { x } else { 7 - x };

                    let lsb = (byte1 >> bit) & 0x01;
                    let msb = (byte2 >> bit) & 0x01;
                    let color_id = (msb << 1) | lsb;

                    // 0 is transparent for sprites
                    if color_id == 0 {
                        continue;
                    }

                    let palette = if (attributes & 0x10) != 0 {
                        self.obp1
                    } else {
                        self.obp0
                    };
                    let color = self.get_sprite_color(color_id, palette);

                    // TODO: Handle Priority (Background over Sprite)
                    let buffer_idx = (self.ly as usize * 160) + pixel_x as usize;
                    self.frame_buffer[buffer_idx] = color;
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
            OBP0_ADDRESS => self.obp0,
            OBP1_ADDRESS => self.obp1,
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
            OBP0_ADDRESS => self.obp0 = val,
            OBP1_ADDRESS => self.obp1 = val,
            _ => {}
        }
    }
}
