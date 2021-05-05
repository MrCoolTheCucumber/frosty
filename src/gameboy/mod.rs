use std::{cell::RefCell, fs::File, io::Read, rc::Rc};

use self::{cpu::Cpu, interupt::Interupt, mmu::Mmu, ppu::Ppu, timer::Timer};

mod cpu;
mod mmu;
mod interupt;
mod ppu;
mod timer;
mod input;

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
    mmu: Rc<RefCell<Mmu>>,
    ppu: Ppu,
    timer: Timer
}

impl GameBoy {
    pub fn new() -> Self {
        let mmu = Rc::new(RefCell::new(Mmu::new()));
        let cpu = Cpu::new(mmu.clone());
        let ppu = Ppu::new(mmu.clone());
        let timer = Timer::new(mmu.clone());

        Self {
            cpu,
            mmu,
            ppu,
            timer
        }
    }

    pub fn get_frame_buffer(&self) -> &[u8] {
        &self.ppu.frame_buffer
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
        self.ppu.tick();
        self.timer.tick();

        // handle interupts at the end (before the next cpu cycle)
        let mut mmu = (*self.mmu).borrow_mut();
        Interupt::handle(&mut mmu.interupts, &mut self.cpu);
    }
}