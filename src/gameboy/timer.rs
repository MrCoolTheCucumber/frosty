use std::{cell::RefCell, rc::Rc};

use super::mmu::Mmu;

pub struct Timer {
    mmu: Rc<RefCell<Mmu>>
}

impl Timer {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        Self {
            mmu
        }
    }

    pub fn tick(&mut self) {
        let mut mmu = (*self.mmu).borrow_mut();
        mmu.io[0x03] = match mmu.io[0x03].checked_add(1) {
            Some(val) => val,
            None => {
                mmu.io[0x04] = mmu.io[0x04].wrapping_add(1); 
                0
            }
        };
    }
}
