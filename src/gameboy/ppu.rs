use std::{cell::{Ref, RefCell}, rc::Rc};
use super::{interupt::InterruptFlag, mmu::Mmu};


pub struct Ppu {
    mmu: Rc<RefCell<Mmu>>,
    mode: PpuMode,
    pub frame_buffer: [u8; 160 * 144],
    window_internal_line_counter: u8,

    mode_clock_cycles: u64,
    line_clock_cycles: u64,
    frame_clock_cycles: u64
}

#[derive(Debug, Copy, Clone)]
pub struct Sprite {
    pub x: u8,
    pub y: u8,
    pub tile: u8,

    pub palette: u8,
    pub xflip: u8,
    pub yflip: u8,
    pub priority: u8
}

impl Sprite {
    pub fn default() -> Self {
        Self {
            x: 0, y: 0, tile: 0, palette: 0,
            xflip: 0, yflip: 0, priority: 0
        }
    }
}

enum PpuMode {
    HBlank = 0, // mode 0
    VBlank = 1, // mode 1
    OAM    = 2, // mode 2, initial state
    VRAM   = 3  // mode 3
}

pub enum LcdControlFlag {
    // 1: on, 0: off
    BGEnable =             0b0000_0001, 

    // Display sprites (obj): 
    // 1: on 
    // 0: off
    OBJEnable =            0b0000_0010, 

    // Sprite Size: 
    // 0: 8x8 
    // 1: 8x16
    OBJSize =              0b0000_0100, 

    // Where the BG tiles are mapped: 0: 0x9800-0x9BFF, 1: 0x9C00-0x9FFF
    BGTileMapAddress =     0b0000_1000, 

    // Location of Tiles for BG and window: 
    // 0: 0x8800-0x97FF
    // 1: 0x8000-0x87FF (Same location as the sprites (OBJ) (They overlap))
    BGAndWindowTileData =  0b0001_0000,

    // Render window as part of the draw
    // 0: off
    // 1: on
    WindowEnable =         0b0010_0000,

    // where window tiles are mapped
    // 0: 0x9800-0x9BFF 
    // 1: 0x9C00-0x9FFF
    WindowTileMapAddress = 0b0100_0000,

    LCDDisplayEnable =     0b1000_0000
}

impl Ppu {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        Self {
            mmu,
            mode: PpuMode::OAM,
            frame_buffer: [0; 160 * 144],
            window_internal_line_counter: 0,

            mode_clock_cycles: 0,
            line_clock_cycles: 0,
            frame_clock_cycles: 0
        }
    }

    fn get_scan_line(&self) -> u8 {
        (*self.mmu).borrow().io[0x44]
    }

    fn inc_scan_line(&mut self) -> u8 {
        let current_scan_line = self.get_scan_line();
        self.set_scan_line(current_scan_line + 1);
        current_scan_line + 1
    }

    fn set_scan_line(&mut self, val: u8) {
        (*self.mmu).borrow_mut().io[0x44] = val;
    }

    fn set_mode_lcdc(&mut self, mode: PpuMode) {
        let mut mmu = (*self.mmu).borrow_mut();
        let mut lcdc = mmu.io[0x41];
        lcdc = (lcdc & 0b11111100) | (mode as u8);
        mmu.io[0x41] = lcdc;
    }

    fn get_lyc(&self) -> u8 {
        (*self.mmu).borrow().io[0x45]
    }

    fn lyc_check_enabled(&self) -> bool {
        (*self.mmu).borrow().io[0x41] & 0b0100_0000 != 0
    }

    fn get_bg_map_start_addr(ldlc_flags: u8) -> u16 {
        match (ldlc_flags & LcdControlFlag::BGTileMapAddress as u8) != 0 {
            true => 0x9C00,
            false => 0x9800
        }
    }

    fn get_window_map_start_addr(ldlc_flags: u8) -> u16 {
        match (ldlc_flags & LcdControlFlag::WindowTileMapAddress as u8) != 0 {
            true => 0x9C00,
            false => 0x9800
        }
    }

    pub fn tick(&mut self) {
        self.mode_clock_cycles += 1;
        self.line_clock_cycles += 1;
        self.frame_clock_cycles += 1;

        // https://robertovaccari.com/gameboy/LCD-refresh-diagram.png
        match self.mode {
            PpuMode::HBlank => {
                if self.line_clock_cycles == 456 {
                    self.mode_clock_cycles = 0;
                    self.line_clock_cycles = 0;
                    
                    // incr the scan line
                    // if scanline is 144, go to VBlank
                    // otherwise go to OAM (mode 2)
                    let current_scan_line = self.inc_scan_line();
                    self.check_ly_eq_lyc();

                    if current_scan_line == 144 {
                        (self.mmu).borrow_mut().interupts.request_interupt(InterruptFlag::VBlank);
                        self.set_mode_lcdc(PpuMode::VBlank);
                        self.mode = PpuMode::VBlank;
                    }
                    else {
                        self.set_mode_lcdc(PpuMode::OAM);
                        self.mode = PpuMode::OAM;
                    }
                }
            }

            PpuMode::VBlank => {
                if self.line_clock_cycles == 456 {
                    self.line_clock_cycles = 0;

                    let new_scan_line = self.inc_scan_line();
                    self.check_ly_eq_lyc();
                    
                    if new_scan_line == 154 {
                        self.set_scan_line(0);
                        self.check_ly_eq_lyc();

                        self.mode_clock_cycles = 0;
                        self.frame_clock_cycles = 0;
                        self.window_internal_line_counter = 0;
                        
                        self.set_mode_lcdc(PpuMode::OAM);
                        self.mode = PpuMode::OAM;
                    }
                }
            },
            
            PpuMode::OAM => {
                if self.mode_clock_cycles == 80 {
                    self.mode_clock_cycles = 0;
                    self.set_mode_lcdc(PpuMode::VRAM);
                    self.mode = PpuMode::VRAM;
                }
            }

            PpuMode::VRAM => {
                // apparently the drawing happens here?
                // (when BG tiles and sprites are rendered)
                // 172-289 clock cycles?
                // for now lets just say 172

                if self.mode_clock_cycles == 172 {
                    self.mode_clock_cycles = 0;
                    self.draw_scan_line(); // draw line!
                    self.set_mode_lcdc(PpuMode::HBlank);
                    self.mode = PpuMode::HBlank;
                }
            }
        }
    }

    fn check_ly_eq_lyc(&mut self) {
        if self.get_scan_line() == self.get_lyc() && self.lyc_check_enabled() {
            (self.mmu).borrow_mut().interupts.request_interupt(InterruptFlag::Stat);
        }
    }

    fn get_adjusted_tile_index(mmu: &Ref<Mmu>, addr: u16, signed_tile_index: bool) -> u16 {
        if signed_tile_index {
            let tile = mmu.read_byte(addr) as i8 as i16;
            if tile >= 0{
                tile as u16 + 256
            }
            else {
                256 - (tile.abs() as u16)
            }
        }
        else {
            mmu.read_byte(addr) as u16
        }
    }

    // given the location of a 8*8 chunk of the 32*32 bg map
    // calculate the offset in tiles from the 0th tile in the map
    fn get_bg_map_offset(tile_y: u8, tile_x: u8) -> u16 {
        (tile_y as u16 * 32) + tile_x as u16
    }

    // Screen shows 20 (row) * 18 (col) tiles
    // Background map is actually 32 * 32 tiles
    // Screen is mapped to map via ScrollX and ScrollY
    // Screen will "wrap around to the other side
    fn draw_scan_line(&mut self) {
        let mmu = (*self.mmu).borrow();
        let ldlc_flags = mmu.io[0x40];

        let scan_line = mmu.io[0x44];

        // TODO: review what happens if bg is disabled
        //       and sprites are enabled
        let mut scan_line_row: [u8; 160] = [0; 160];

        // 
        // DRAW BACKGROUND
        //
        if (ldlc_flags & LcdControlFlag::BGEnable as u8) != 0 {
            let bg_map_start_addr = Self::get_bg_map_start_addr(ldlc_flags);

            let scroll_y: u8 = mmu.io[0x42];
            let scroll_x: u8 = mmu.io[0x43];

            // top left coordinate of the view port
            // very important that these are u8's so overflowing
            // naturally handles "view port wrapping around"
            let y: u8 = scan_line.wrapping_add(scroll_y);
            let mut x: u8 = scroll_x;

            // get the "tile" (x, y), which 8x8 chunk is the above coordinate in 
            let tile_y = y / 8;
            let mut tile_x = x / 8;

            // calculate tile index
            let mut tile_map_offset = Self::get_bg_map_offset(tile_y, tile_x);
            // are we using signed addressing for accessing the tile data (not map)
            let signed_tile_addressing: bool = ldlc_flags & LcdControlFlag::BGAndWindowTileData as u8 == 0;

            // get the tile index from the map
            let mut tile_index = Self::get_adjusted_tile_index(
                &mmu, 
                bg_map_start_addr + tile_map_offset, 
                signed_tile_addressing
            );

            // above x and y are where we are relative to the whole bg map (256 * 256)
            // we need x and y to be relative to the tile we want to draw
            // so modulo by 8 (or & 7)
            // shadow over old x an y variables
            let tile_local_y = y & 7;
            let mut tile_local_x = x & 7;

            let mut tile_address = 0x8000 + (tile_index * 16) + (tile_local_y as u16 * 2);
            let mut b1 = mmu.read_byte(tile_address);
            let mut b2 = mmu.read_byte(tile_address + 1);

            let mut frame_buffer_offset = scan_line as usize * 160;
            let bg_color_palette = mmu.bg_palette;

            let mut pixels_drawn_for_current_tile: u8 = 0;
            for i in 0..160 {
                let bx = 7 - tile_local_x;
                let color_bit = ((b1 & (1 << bx)) >> bx) | ((b2 & (1 << bx)) >> bx) << 1;
                let color = bg_color_palette[color_bit as usize];

                scan_line_row[i] = color.clone();
                self.frame_buffer[frame_buffer_offset] = color;
                frame_buffer_offset += 1;

                tile_local_x += 1;
                pixels_drawn_for_current_tile += 1;

                if tile_local_x == 8 {
                    tile_local_x = 0;

                    // set up the next tile
                    // need to be carefull here (i think?) becaucse the view port can
                    // wrap around?

                    x = x.wrapping_add(pixels_drawn_for_current_tile);
                    pixels_drawn_for_current_tile = 0;

                    tile_x = x / 8;
                    tile_map_offset = Self::get_bg_map_offset(tile_y, tile_x);
                    tile_index = Self::get_adjusted_tile_index(
                        &mmu,
                        bg_map_start_addr + tile_map_offset, 
                        signed_tile_addressing
                    );

                    tile_address = 0x8000 + (tile_index * 16) + (tile_local_y as u16 * 2);
                    b1 = mmu.read_byte(tile_address);
                    b2 = mmu.read_byte(tile_address + 1);
                }
            }
        }

        //
        // DRAW WINDOW
        //
        let window_y = mmu.io[0x4A];
        let window_x = mmu.io[0x4B].wrapping_sub(7);

        let skip_window_draw = window_y > 166 || window_x > 143 || window_y > scan_line;

        if ldlc_flags & LcdControlFlag::WindowEnable as u8 != 0 && !skip_window_draw {
            let wd_map_start_addr = Self::get_window_map_start_addr(ldlc_flags);
            self.window_internal_line_counter += 1;

            let y = self.window_internal_line_counter - 1; //scan_line - window_y;
            let mut x = window_x;

            let tile_y = y / 8;

            let mut tile_map_offset = tile_y as u16 * 32;
            // tile_map_offset = (tile_y as u16) + tile_x as u16;
            // are we using signed addressing for accessing the tile data (not map)
            let signed_tile_addressing: bool = ldlc_flags & LcdControlFlag::BGAndWindowTileData as u8 == 0;

            // get the tile index from the map
            let mut tile_index = Self::get_adjusted_tile_index(
                &mmu, 
                wd_map_start_addr + tile_map_offset, 
                signed_tile_addressing
            );

            // above x and y are where we are relative to the whole bg map (256 * 256)
            // we need x and y to be relative to the tile we want to draw
            // so modulo by 8 (or & 7)
            // shadow over old x an y variables
            let tile_local_y = y & 7;
            let mut tile_local_x = x & 7;

            let mut tile_address = 0x8000 + (tile_index * 16) + (tile_local_y as u16 * 2);
            let mut b1 = mmu.read_byte(tile_address);
            let mut b2 = mmu.read_byte(tile_address + 1);

            let mut frame_buffer_offset = (scan_line as usize * 160) + x as usize;
            let bg_color_palette = mmu.bg_palette;

            let mut pixels_drawn_for_current_tile: u8 = 0;
            let start = x.clone();
            for i in start..160 {
                let bx = 7 - tile_local_x;
                let color_bit = ((b1 & (1 << bx)) >> bx) | ((b2 & (1 << bx)) >> bx) << 1;
                let color = bg_color_palette[color_bit as usize];

                scan_line_row[i as usize] = color.clone();
                self.frame_buffer[frame_buffer_offset] = color;
                frame_buffer_offset += 1;

                tile_local_x += 1;
                pixels_drawn_for_current_tile += 1;

                if tile_local_x == 8 {
                    tile_local_x = 0;

                    // set up the next tile
                    // need to be carefull here (i think?) becaucse the view port can
                    // wrap around?

                    x = x.wrapping_add(pixels_drawn_for_current_tile);
                    pixels_drawn_for_current_tile = 0;

                    tile_map_offset += 1;
                    tile_index = Self::get_adjusted_tile_index(
                        &mmu,
                        wd_map_start_addr + tile_map_offset, 
                        signed_tile_addressing
                    );

                    tile_address = 0x8000 + (tile_index * 16) + (tile_local_y as u16 * 2);
                    b1 = mmu.read_byte(tile_address);
                    b2 = mmu.read_byte(tile_address + 1);
                }
            }
        } 

        //
        // DRAW SPRITES
        //
        let sprite_size: i32 = if ldlc_flags & LcdControlFlag::OBJSize as u8 != 0 {16} else {8};
        if ldlc_flags & LcdControlFlag::OBJEnable as u8 != 0 {
            // OAM is 160 bytes, each sprite takes 4 bytes, so 40 in total
            // Can only draw maximum of 10 sprites per line

            let mut total_objects_drawn = 0;
            let mut obj_same_x_prio = [i32::MAX; 160];

            for i in 0..40 {
                let sprite_addr = (i as usize) * 4;
                
                let sprite_y = mmu.sprite_table[sprite_addr] as u16 as i32 - 16;
                let sprite_x = mmu.sprite_table[sprite_addr + 1] as u16 as i32 - 8;
                let tile_num = 
                    (mmu.sprite_table[sprite_addr + 2] & 
                    (if sprite_size == 16 { 0xFE } else { 0xFF })) as u16;

                let flags = mmu.sprite_table[sprite_addr + 3];
                let sprite_palette: usize = if flags & (1 << 4) != 0 {1} else {0};
                let xflip: bool = flags & (1 << 5) != 0;
                let yflip: bool = flags & (1 << 6) != 0;
                let belowbg: bool = flags & (1 << 7) != 0;

                let scan_line = scan_line as i32;

                // exit early if sprite is off screen
                if scan_line < sprite_y || scan_line >= sprite_y + sprite_size { continue }
                if sprite_x < - 7 || sprite_x >= 160 { continue }

                total_objects_drawn += 1;
                if total_objects_drawn > 10 { break }

                // fetch sprite tile
                let tile_y: u16 = if yflip {
                    (sprite_size - 1 - (scan_line - sprite_y)) as u16
                } else {
                    (scan_line - sprite_y) as u16
                };

                let tile_address = 0x8000 + tile_num * 16 + tile_y * 2;
                let b1 = mmu.read_byte(tile_address);
                let b2 = mmu.read_byte(tile_address + 1);

                // draw each pixel of the sprite tile
                'inner: for x in 0..8 {
                    if sprite_x + x < 0 || sprite_x + x >= 160 { continue }
                    // if sprite prio is below bg, and the drawn bg pixel is 0 (nothing) then skip to the next pixel
                    if belowbg && scan_line_row[(sprite_x + x) as usize] != mmu.bg_palette[0] { 
                        continue 'inner 
                    }

                    // has another sprite already been drawn on this pixel
                    // that has a lower or eq sprite_x val?
                    if obj_same_x_prio[(sprite_x + x) as usize] <= sprite_x {
                        continue 'inner 
                    }

                    let xbit = 1 << (if xflip { x } else { 7 - x } as u32);
                    let colnr = (if b1 & xbit != 0 { 1 } else { 0 }) |
                        (if b2 & xbit != 0 { 2 } else { 0 });
                    if colnr == 0 { continue }

                    let color = mmu.sprite_palette[sprite_palette][colnr];
                    let pixel_offset = ((scan_line * 160) + sprite_x + x) as usize;

                    self.frame_buffer[pixel_offset] = color;
                    obj_same_x_prio[(sprite_x + x) as usize] = sprite_x;
                }
            }
        }
    }
}
