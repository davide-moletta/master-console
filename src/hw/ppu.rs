use crate::hw;

/// Defines the mode in which the [`Ppu`] is operating
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}

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
/// `wx` & `wy` -> window register to mange background
/// `bg_line_color_indices` -> used for priority
#[derive(Debug)]
pub struct Ppu {
    vram: [u8; hw::VRAM_SIZE],
    oam: [u8; hw::OAM_SIZE],
    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    bgp: u8,
    mode: Mode,
    ticks: u32,
    frame_buffer: [u32; hw::SCREEN_WIDTH * hw::SCREEN_HEIGHT],
    vblank_interrupt: bool,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,
    bg_line_color_indices: [u8; hw::OAM_SIZE],
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            vram: [hw::U8_ZERO; hw::VRAM_SIZE],
            oam: [hw::U8_ZERO; hw::OAM_SIZE],
            // TODO this is the setup after boot rom, add real in the future
            lcdc: 0x91, // LCD Enabled, BG Enabled
            stat: 0x85, // Starting in VBlank
            scy: hw::U8_ZERO,
            scx: hw::U8_ZERO,
            ly: hw::U8_ZERO,
            lyc: hw::U8_ZERO,
            bgp: 0xFC, // Standard palette (11 11 11 00)
            mode: Mode::OamScan,
            ticks: hw::U32_ZERO,
            frame_buffer: [hw::WHITE; hw::SCREEN_WIDTH * hw::SCREEN_HEIGHT],
            vblank_interrupt: false,
            obp0: hw::U8_ZERO, // Default sprite palette 0 (hardware default)
            obp1: hw::U8_ZERO, // Default sprite palette 1 (hardware default)
            wy: hw::U8_ZERO,
            wx: hw::U8_ZERO,
            bg_line_color_indices: [hw::U8_ZERO; hw::OAM_SIZE],
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
    pub fn get_frame(&self) -> [u32; hw::SCREEN_WIDTH * hw::SCREEN_HEIGHT] {
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
            0 => hw::WHITE,
            1 => hw::LGREY,
            2 => hw::DGREY,
            _ => hw::BLACK,
        }
    }

    /// Helper to get the color for sprites
    fn get_sprite_color(&self, color_id: u8, palette: u8) -> u32 {
        let palette_index = (palette >> (color_id * 2)) & 0x03;
        match palette_index {
            0 => hw::WHITE,
            1 => hw::LGREY,
            2 => hw::DGREY,
            _ => hw::BLACK,
        }
    }

    /// Renders a line for the display
    pub fn render_scanline(&mut self) {
        // Bit 7 of LCDC is the LCD enable flag. If 0, display should be off (white)
        let lcd_enable = (self.lcdc & 0x80) != 0;
        if !lcd_enable {
            // Fill the scanline with white (palette index 0 = white)
            for x in 0..hw::SCREEN_WIDTH {
                let buffer_idx = (self.ly as usize * hw::SCREEN_WIDTH) + x;
                self.frame_buffer[buffer_idx] = hw::WHITE;
            }
            return;
        }

        // Bit 0 of lcdc acts as a master enable for the BG and Window on DMG
        let bg_window_enable = (self.lcdc & 0x01) != 0;
        let window_enable = (self.lcdc & 0x20) != 0 && self.ly >= self.wy;

        let bg_tile_map_base: u16 = if (self.lcdc & 0x08) != 0 {
            0x9C00
        } else {
            0x9800
        };

        let win_tile_map_base: u16 = if (self.lcdc & 0x40) != 0 {
            0x9C00
        } else {
            0x9800
        };

        let use_unsigned = (self.lcdc & 0x10) != 0;

        for x in 0..hw::SCREEN_WIDTH as u8 {
            // Determine if current pixel belongs to the Window or Background
            let is_window = window_enable && x + 7 >= self.wx;

            let (tile_map_base, x_pos, y_pos) = if is_window {
                let win_x = (x + 7 - self.wx) as u16;
                let win_y = (self.ly - self.wy) as u16;
                (win_tile_map_base, win_x, win_y)
            } else {
                if !bg_window_enable {
                    let buffer_idx = (self.ly as usize * hw::SCREEN_WIDTH) + x as usize;
                    self.frame_buffer[buffer_idx] = hw::WHITE;
                    continue;
                }
                let bg_x = x.wrapping_add(self.scx) as u16;
                let bg_y = self.ly.wrapping_add(self.scy) as u16;
                (bg_tile_map_base, bg_x, bg_y)
            };

            let tile_y = (y_pos / 8) % 32;
            let tile_x = (x_pos / 8) % 32;
            let pixel_y_in_tile = y_pos % 8;
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

            // Calculate relative index for the pixel data
            let vram_index = (tile_data_addr - 0x8000) + (pixel_y_in_tile * 2);

            // Fetch bytes directly from array
            let byte1 = self.vram[vram_index as usize];
            let byte2 = self.vram[(vram_index + 1) as usize];

            let bit = 7 - pixel_x_in_tile;
            let lsb = (byte1 >> bit) & 0x01;
            let msb = (byte2 >> bit) & 0x01;
            let color_id = (msb << 1) | lsb;
            self.bg_line_color_indices[x as usize] = color_id;

            let color = self.get_color(color_id);
            let buffer_idx = (self.ly as usize * hw::SCREEN_WIDTH) + x as usize;
            self.frame_buffer[buffer_idx] = color;
        }
    }

    /// Renders sprites
    pub fn render_sprites(&mut self) {
        if (self.lcdc & 0x02) == 0 {
            return;
        }

        // Check LCDC bit 2 for sprite size (0=8x8, 1=8x16)
        let is_8x16 = (self.lcdc & 0x04) != 0;
        let sprite_height = if is_8x16 { 16 } else { 8 };

        for i in 0..40 {
            let i = i * 4;
            let sprite_y = self.oam[i] as i16 - 16;
            let sprite_x = self.oam[i + 1] as i16 - 8;
            let mut tile_index = self.oam[i + 2];
            let attributes = self.oam[i + 3];

            if (self.ly as i16) >= sprite_y && (self.ly as i16) < (sprite_y + sprite_height) {
                let mut pixel_y = (self.ly as i16 - sprite_y) as u16;

                // Handle Y-Flip
                if (attributes & 0x40) != 0 {
                    pixel_y = (sprite_height as u16 - 1) - pixel_y;
                }

                // In 8x16 mode, bit 0 of the tile index is ignored for the top tile.
                // The top 8 pixels use (index & 0xFE), bottom 8 use (index | 0x01).
                if is_8x16 {
                    if pixel_y < 8 {
                        tile_index &= 0xFE;
                    } else {
                        tile_index |= 0x01;
                    }
                }

                // Calculate address: Note that for 8x16, we treat it as two 8x8 tiles
                // If we are on the second tile (pixel_y >= 8), our address calculation
                // should be relative to that specific 8x8 tile.
                let tile_data_addr = 0x8000 + (tile_index as u16 * 16);
                let line_in_tile = pixel_y % 8;
                let byte1 = self.read(tile_data_addr + (line_in_tile * 2));
                let byte2 = self.read(tile_data_addr + (line_in_tile * 2) + 1);

                for x in 0..8 {
                    let pixel_x_on_screen = sprite_x + x as i16;
                    if !(0..160).contains(&pixel_x_on_screen) {
                        continue;
                    }

                    let bit = if (attributes & 0x20) != 0 { x } else { 7 - x };
                    let lsb = (byte1 >> bit) & 0x01;
                    let msb = (byte2 >> bit) & 0x01;
                    let color_id = (msb << 1) | lsb;

                    if color_id == 0 {
                        continue;
                    } // Transparent

                    let bg_priority = (attributes & 0x80) != 0;
                    if bg_priority && self.bg_line_color_indices[pixel_x_on_screen as usize] != 0 {
                        continue;
                    }

                    let palette = if (attributes & 0x10) != 0 {
                        self.obp1
                    } else {
                        self.obp0
                    };
                    let color = self.get_sprite_color(color_id, palette);

                    let buffer_idx =
                        (self.ly as usize * hw::SCREEN_WIDTH) + pixel_x_on_screen as usize;
                    self.frame_buffer[buffer_idx] = color;
                }
            }
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            hw::VRAM_START..=hw::VRAM_END => self.vram[(addr - hw::VRAM_START) as usize],
            hw::OAM_START..=hw::OAM_END => self.oam[(addr - hw::OAM_START) as usize],
            hw::LCDC_ADDRESS => self.lcdc,
            hw::STAT_ADDRESS => self.stat,
            hw::SCY_ADDRESS => self.scy,
            hw::SCX_ADDRESS => self.scx,
            hw::LY_ADDRESS => self.ly,
            hw::LYC_ADDRESS => self.lyc,
            hw::BGP_ADDRESS => self.bgp,
            hw::OBP0_ADDRESS => self.obp0,
            hw::OBP1_ADDRESS => self.obp1,
            hw::WY_ADDRESS => self.wy,
            hw::WX_ADDRESS => self.wx,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            hw::VRAM_START..=hw::VRAM_END => self.vram[(addr - hw::VRAM_START) as usize] = val,
            hw::OAM_START..=hw::OAM_END => self.oam[(addr - hw::OAM_START) as usize] = val,
            hw::LCDC_ADDRESS => self.lcdc = val,
            hw::STAT_ADDRESS => self.stat = val,
            hw::SCY_ADDRESS => self.scy = val,
            hw::SCX_ADDRESS => self.scx = val,
            hw::LY_ADDRESS => self.ly = 0, // Writes to ly reset it
            hw::LYC_ADDRESS => self.lyc = val,
            hw::BGP_ADDRESS => self.bgp = val,
            hw::OBP0_ADDRESS => self.obp0 = val,
            hw::OBP1_ADDRESS => self.obp1 = val,
            hw::WY_ADDRESS => self.wy = val,
            hw::WX_ADDRESS => self.wx = val,
            _ => {}
        }
    }
}
