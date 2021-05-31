use std::{cell::RefCell, rc::Rc};

use sdl2::{audio::AudioQueue, keyboard::Keycode};

use self::{cpu::Cpu, interupt::{InterruptFlag, Interupt}, mmu::Mmu, ppu::Ppu, spu::{Spu}};

mod cpu;
mod mmu;
mod interupt;
mod ppu;
pub mod spu;
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
    ppu: Ppu
}

impl GameBoy {
    pub fn new(rom_path: &str, device: Option<Rc<RefCell<AudioQueue<f32>>>>) -> Self {
        let cartridge = cartridge::create(rom_path);
        let spu = Spu::new(device);
        let mmu = Rc::new(RefCell::new(Mmu::new(cartridge, spu)));
        
        let cpu = Cpu::new(mmu.clone());
        let ppu = Ppu::new(mmu.clone());
        
        Self {
            cpu,
            mmu,
            ppu
        }
    }

    pub fn key_down(&mut self, key: Keycode) {
        let mut mmu = (*self.mmu).borrow_mut();
        let sucessful_press = mmu.input.key_down(key);
        
        if sucessful_press {
            self.cpu.stopped = false;
            mmu.interupts.request_interupt(interupt::InterruptFlag::Joypad);
        }
    }

    pub fn key_up(&mut self, key: Keycode) {
        (*self.mmu).borrow_mut().input.key_up(key);
    }

    pub fn get_frame_buffer(&self) -> &[u8] {
        &self.ppu.frame_buffer
    }

    pub fn tick(&mut self) {
        if self.cpu.stopped { return }

        self.cpu.tick();
        self.ppu.tick();
        
        let mut mmu = (*self.mmu).borrow_mut();

        mmu.spu.tick();
        
        let request_timer_interupt = mmu.timer.tick();
        if request_timer_interupt {
            mmu.interupts.request_interupt(InterruptFlag::Timer)
        }

        Interupt::handle(&mut mmu.interupts, &mut self.cpu);
    }
}