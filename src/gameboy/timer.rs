use std::{cell::RefCell, rc::Rc};

use super::mmu::Mmu;

pub struct Timer {
    mmu: Rc<RefCell<Mmu>>,
    div_cycles: u64
}

impl Timer {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        Self {
            mmu,
            div_cycles: 0
        }
    }

    pub fn tick(&mut self) {
        self.div_cycles += 1;
        if self.div_cycles == 256 {
            self.div_cycles = 0;
            let mut mmu = (*self.mmu).borrow_mut();
            // TODO: what happens if DIV overflows?
            mmu.io[0x04] = mmu.io[0x04].wrapping_add(1);
        }
    }
}