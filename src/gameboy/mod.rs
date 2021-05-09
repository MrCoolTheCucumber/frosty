use std::{cell::RefCell, rc::Rc};

use ggez::event::KeyCode;

use self::{cpu::Cpu, interupt::Interupt, mmu::Mmu, ppu::Ppu, timer::Timer};

mod cpu;
mod mmu;
mod interupt;
mod ppu;
mod timer;
mod input;
mod cartridge;

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
    pub fn new(rom_path: &str) -> Self {
        let cartridge = cartridge::create(rom_path);
        let mmu = Rc::new(RefCell::new(Mmu::new(cartridge)));
        
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

    pub fn key_down(&mut self, key: KeyCode) {
        let mut mmu = (*self.mmu).borrow_mut();
        let sucessful_press = mmu.input.key_down(key);
        
        if sucessful_press {
            self.cpu.stopped = false;
            mmu.interupts.request_interupt(interupt::InterruptFlag::Joypad);
        }
    }

    pub fn key_up(&mut self, key: KeyCode) {
        (*self.mmu).borrow_mut().input.key_up(key);
    }

    pub fn get_frame_buffer(&self) -> &[u8] {
        &self.ppu.frame_buffer
    }

    pub fn tick(&mut self) {
        if self.cpu.stopped { return }

        self.cpu.tick();
        self.ppu.tick();
        self.timer.tick();

        // handle interupts at the end (before the next cpu cycle)
        let mut mmu = (*self.mmu).borrow_mut();
        Interupt::handle(&mut mmu.interupts, &mut self.cpu);
    }
}