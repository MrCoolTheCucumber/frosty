use std::{cell::RefCell, rc::Rc};

use crate::gameboy::interupt::InterruptFlag;

use super::mmu::Mmu;

// TODO:
// "Additionally, this (DIV) register is reset when executing the stop instruction, and only begins ticking again once stop mode ends."

// Impl based on the cycle accurate docs diagram for obscure timer behaviour
// also found here: https://gbdev.gg8.se/wiki/articles/Timer_Obscure_Behaviour

pub struct Timer {
    mmu: Rc<RefCell<Mmu>>,

    count: u64,
    tima_overflow: bool,
    ticks_since_tima_overflow: u64,
    freq_to_bit: [u8; 4],
    prev_bit: bool
}

impl Timer {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        Self {
            mmu,
            
            count: 0,
            tima_overflow: false,
            ticks_since_tima_overflow: 0,
            freq_to_bit: [9, 3, 5, 7],
            prev_bit: false
        }
    }

    pub fn tick(&mut self) {
        let mut mmu = (*self.mmu).borrow_mut();

        // DIV handling
        mmu.io[0x03] = match mmu.io[0x03].checked_add(1) {
            Some(val) => val,
            None => {
                mmu.io[0x04] = mmu.io[0x04].wrapping_add(1); 
                0
            }
        };

        // TIMA handling (the diagram from comment above)
        let tac = mmu.io[0x7];
        let bit_pos = self.freq_to_bit[(tac & 3) as usize];
        let div: u16 = ((mmu.io[0x04] as u16) << 8) | mmu.io[0x03] as u16;

        let mut bit = (div & (1 << bit_pos)) != 0;
        bit = bit && (tac & (1 << 2)) != 0; // is timer enabled

        if !bit && self.prev_bit {
            let tima = mmu.io[0x05].wrapping_add(1);
            if tima == 0 {
                self.tima_overflow = true;
                self.ticks_since_tima_overflow = 0;
            }
            mmu.io[0x05] = tima;
        }

        self.prev_bit = bit;

        // TIMA overflow handling
        if !self.tima_overflow { 
            return; 
        }

        self.ticks_since_tima_overflow += 1;

        match self.ticks_since_tima_overflow {
            0..=3 => { } // do nothing

            4 => {
                mmu.interupts.request_interupt(InterruptFlag::Timer);
            }

            5 => {
                mmu.io[0x05] = mmu.io[0x06];
            }

            6 => {
                mmu.io[0x05] = mmu.io[0x06];
                self.tima_overflow = false;
                self.ticks_since_tima_overflow = 0;
                return;
            }

            _ => unreachable!()
        }
    }
}
