use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::gameboy::mmu::Mmu;

use super::{FifoPixel, LcdControlFlag, Sprite};


pub struct SpriteFetcher {
    mmu: Rc<RefCell<Mmu>>,
    pub cycle: u8,

    tile_addr: u16,
    data_low: u8,
    data_high: u8
}

impl SpriteFetcher {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        Self {
            mmu,
            cycle: 0,

            tile_addr: 0,
            data_low: 0,
            data_high: 0
        }
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
        self.tile_addr = 0;
        self.data_low = 0;
        self.data_high = 0;
    }

    pub fn tick(&mut self, sprite_fifo: &mut VecDeque<FifoPixel>, sprite: &Sprite) {
        self.cycle += 1;

        match self.cycle {
            2 => {
                // could do some of this in OAM and store it in the sprite struct instead?
                let mmu = (*self.mmu).borrow();
                let ldlc_flags = mmu.io[0x40];
                let sprite_size: u8 = if ldlc_flags & LcdControlFlag::OBJSize as u8 != 0 
                    {16} else {8};
                let scan_line = mmu.io[0x44];

                let tile_y: u16 = if sprite.yflip {
                    let base = if sprite_size == 16 { 30 } else { 14 };
                    base - ((scan_line - (sprite.y - 16)) as u16 * 2)
                } else {
                    (scan_line.wrapping_sub(sprite.y.wrapping_sub(16))) as u16 * 2
                };

                self.tile_addr = 0x8000 + (sprite.tile_num * 16) + tile_y;
                self.data_low = mmu.gpu_vram[(self.tile_addr - 0x8000) as usize];
            }

            4 => {
                let mmu = (*self.mmu).borrow();
                self.data_high = mmu.gpu_vram[(self.tile_addr + 1 - 0x8000) as usize];
            }
            

            /*
                FROM: https://hacktixme.ga/GBEDG/ppu/#sprite-fetching
                During this step only pixels which are actually visible on the screen 
                are loaded into the FIFO. A sprite with an X-value of 8 would have 
                all 8 pixels loaded, while a sprite with an X-value of 7 would 
                only have the rightmost 7 pixels loaded. Additionally, pixels can only be 
                loaded into FIFO slots if there is no pixel in the given slot already. 
                For example, if the Sprite FIFO contains one sprite pixel from a previously 
                fetched sprite, the first pixel of the currently fetched sprite is 
                discarded and only the last 7 pixels are loaded into the FIFO, while 
                the pixel from the first sprite is preserved.
            */
            6 => {
                let fifo_len = sprite_fifo.len();
                let mut fifo_buffer: Vec<FifoPixel> = Vec::new();

                for _ in 0..fifo_len {
                    let px = sprite_fifo.pop_front().unwrap();
                    fifo_buffer.push(px);
                }

                for x in 0..8 {
                    if sprite.x + x < 8 { continue; }

                    let xbit = 1 << (if sprite.xflip { x } else { 7 - x } as u32);
                    let colnr = (if self.data_low & xbit != 0 { 1 } else { 0 }) |
                        (if self.data_high & xbit != 0 { 2 } else { 0 });

                    let px_data = FifoPixel {
                        belowbg: sprite.belowbg,
                        sprite_color_bit: colnr,
                        sprite_palette: sprite.sprite_palette 
                    };

                    if (x as usize) < fifo_len {
                        if fifo_buffer[x as usize].sprite_color_bit == 0 {
                            fifo_buffer[x as usize] = px_data;
                        }
                    } else {
                        fifo_buffer.push(px_data);
                    }
                }     
                
                for px in fifo_buffer {
                    sprite_fifo.push_back(px);
                }
            }

            _ => { } // NOP
        }
    }
}