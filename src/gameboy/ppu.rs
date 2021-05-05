use std::{cell::RefCell, rc::Rc};
use super::{interupt::InterruptFlag, mmu::Mmu};


pub struct Ppu {
    mmu: Rc<RefCell<Mmu>>,
    mode: PpuMode,
    pub frame_buffer: [u8; 160 * 144],

    mode_clock_cycles: u64,
    line_clock_cycles: u64,
    frame_clock_cycles: u64
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

    WindowTileMapAddress = 0b0100_0000,

    LCDDisplayEnable =     0b1000_0000
}

impl Ppu {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        Self {
            mmu,
            mode: PpuMode::OAM,
            frame_buffer: [0; 160 * 144],

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

    fn get_bg_map_start_addr(ldlc_flags: u8) -> u16 {
        match (ldlc_flags & LcdControlFlag::BGTileMapAddress as u8) != 0 {
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
                    if current_scan_line == 144 {
                        // TODO: request a VBlank interupt too?
                        (&self.mmu).borrow_mut().interupts.request_interupt(InterruptFlag::VBlank);
                        self.mode = PpuMode::VBlank;
                    }
                    else {
                        self.mode = PpuMode::OAM;
                    }
                }
            }

            PpuMode::VBlank => {
                if self.line_clock_cycles == 456 {
                    self.line_clock_cycles = 0;
                    let new_scan_line = self.inc_scan_line();
                    
                    if new_scan_line == 154 {
                        self.set_scan_line(0);
                        self.mode_clock_cycles = 0;
                        self.frame_clock_cycles = 0;
                        
                        self.mode = PpuMode::OAM;
                    }
                }
            },
            
            PpuMode::OAM => {
                if self.mode_clock_cycles == 80 {
                    self.mode_clock_cycles = 0;
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
                    self.scan_line(); // draw line!
                    self.mode = PpuMode::HBlank;
                }
            }
        }
    }

    fn scan_line(&mut self) {
        // BG Tiles are 8x8 pixels
        // tile maps are 32 * 32 tiles
        // but only 20 * 18 are visible in the window
        let mmu = (*self.mmu).borrow();
        let ldlc_flags = mmu.io[0x40];

        let bg_map_tile_offset = Self::get_bg_map_start_addr(ldlc_flags);
        let scan_line = mmu.io[0x44];

        // get the top left co-ords of the window
        // to offset the 160 * 144 onto the 256 * 256 bg map
        let scroll_y: u8 = mmu.io[0x42];
        let scroll_x: u8 = mmu.io[0x43];

        // where we actually are
        let y = scroll_y.wrapping_add(scan_line) as u16;
        let x = scroll_x as u16;

        let tiles_per_dimension: u16 = 8; // = 256 / 32

        let tile_y = y / tiles_per_dimension;
        let tile_x = x / tiles_per_dimension;
        
        let tiles_per_row = 32; // (32 * 32) / 32
        let mut map_offset = (tile_y * tiles_per_row) + tile_x;

        // tile data can either start at 0x8000 or 0x8800
        // if it starts at 0x8800, in other words, if the BGAndWindowTileData flag is NOT set
        // then tiles are addressed with signed 8 bit ints rather than unsigned 8 bit ints
        // and base offset is 0x9000
        let signed_tile_addressing: bool = ldlc_flags & LcdControlFlag::BGAndWindowTileData as u8 == 0;

        let mut tile: u16 = mmu.read_byte(bg_map_tile_offset + map_offset) as u16;

        if signed_tile_addressing {
            let mut i_tile = tile as i16;
            i_tile += 128;
            tile = i_tile as u16;
        }

        // x and y above (where we actually are) need to be modded by 8
        // do give us where we are inside the current tile!
        // or you can do the & of 7 as we are working with pwers of 2 here!
        let mut x = (x & 7) as u8;
        let y = (y & 7) as u8;

        let mut frame_buffer_offset = scan_line as usize * 160;
        let bg_color_palette = mmu.bg_palette;

        let mut scan_line_row: [u8; 160] = [0; 160];

        // screen width is 160
        if (ldlc_flags & LcdControlFlag::BGEnable as u8) != 0 {
            for i in 0..160 {
                // get color of pixel in tile
                let color = bg_color_palette[
                    mmu.tileset[tile as usize][y as usize][x as usize] as usize
                ];
                scan_line_row[i] = color.clone();
                self.frame_buffer[frame_buffer_offset] = color;
                frame_buffer_offset += 1;

                // move along the tile
                x += 1; 

                // if we move onto the next tile, reset x and fetch the new tile addr
                if x == 8 {
                    x = 0;
                    map_offset += 1; // wrapping add if over 32
                    tile = mmu.read_byte(bg_map_tile_offset + map_offset) as u16;

                    if signed_tile_addressing {
                        let mut i_tile = tile as i16;
                        i_tile += 128;
                        tile = i_tile as u16;
                    }
                }
            }
        }
    }
}
