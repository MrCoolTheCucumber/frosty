use std::{cell::RefCell, fs::File, io::Read, rc::Rc};

use self::{cpu::Cpu, mmu::Mmu};

mod cpu;
mod mmu;
mod interupt;

/*
    System Clocks
    ==============

    CPU:  4,194,304 Hz
    RAM:  1,048,576 Hz
    PPU:  4,194,304 Hz
    VRAM: 2,097,152 Hz
*/

pub struct GameBoy {
    cpu: Cpu,
    mmu: Rc<RefCell<Mmu>>
}

impl GameBoy {
    pub fn new() -> Self {
        let mmu = Rc::new(RefCell::new(Mmu::new()));
        let cpu = Cpu::new(mmu.clone());
        
        Self {
            cpu,
            mmu
        }
    }

    pub fn load_rom(&mut self, rom_path: &str) {
        let file = File::open(rom_path);
        let file = match file {
            Ok(f) => f,
            Err(err) => panic!("Something went wrong reading the ROM: {}", err)
        };

        let mut data = Vec::new();
        file.take(0x8000).read_to_end(&mut data).ok();

        let mut mmu = (*self.mmu).borrow_mut();
        mmu.write_rom_to_bank_0(&data);
        mmu.write_rom_to_bank_1(&data);
    }

    pub fn tick(&mut self) {
        self.cpu.tick();
    }
}