use std::{collections::VecDeque, fmt};

use super::{cpu::{Cpu, disassembler::{Instruction, InstructionStep}}};

// https://eldred.fr/gb-asm-tutorial/interrupts.html

pub struct Interupt {
    pub master: u8,
    pub enable: u8,
    pub flags: u8,

    pub waiting_for_halt_if: bool,
    pub halt_interupt_pending: bool
}

#[derive(Clone, Copy)]
pub enum InterruptFlag {
    VBlank = 0b00000001,
    Stat   = 0b00000010,
    Timer  = 0b00000100,
    Serial = 0b00001000,
    Joypad = 0b00010000
}

impl fmt::Debug for Interupt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("cpu")
            .field("master", &format!("{:#X}", self.master))
            .field("enable", &format!("{:#X}", self.enable))
            .field("flags", &format!("{:#08b}", self.flags))

            .finish()
    }
}

impl Interupt {
    pub fn new() -> Self {
        Self {
            master: 0,
            enable: 0,
            flags: 0,

            waiting_for_halt_if: false,
            halt_interupt_pending: false
        }
    }

    pub fn enable_master(&mut self) {
        self.master = 1;
    }

    pub fn disable_master(&mut self) {
        self.master = 0;
    }

    pub fn is_master_enabled(&self) -> bool {
        self.master > 0
    }

    pub fn get_interupt_state(&self) -> Option<InterruptFlag> {
        self.get_interupt_state_latched(self.enable, self.flags)
    }

    pub fn get_interupt_state_latched(&self, interupt_enable_flags: u8, interupt_req_flags: u8) -> Option<InterruptFlag> {
        if (interupt_enable_flags > 0) && interupt_req_flags > 0 {
            let interupt: u8 = interupt_enable_flags & interupt_req_flags & 0x1F;

            if interupt & InterruptFlag::VBlank as u8 > 0 {
                return Some(InterruptFlag::VBlank);
            }

            if interupt & InterruptFlag::Stat as u8 > 0 {
                return Some(InterruptFlag::Stat);
            }

            if interupt & InterruptFlag::Timer as u8 > 0 {
                return Some(InterruptFlag::Timer);
            }

            if interupt & InterruptFlag::Serial as u8 > 0 {
                return Some(InterruptFlag::Serial);
            }

            if interupt & InterruptFlag::Joypad as u8 > 0 {
                return Some(InterruptFlag::Joypad);
            }
        }

        None
    }

    pub fn clear_interupt(&mut self, flag: InterruptFlag) {
        self.flags = self.flags & !(flag as u8);
    }

    pub fn request_interupt(&mut self, flag: InterruptFlag) {
        self.flags = self.flags | flag as u8;
        
        if self.waiting_for_halt_if {
            self.halt_interupt_pending = true;
        }
    }

    pub fn handle(interrupt: &mut Interupt, cpu: &mut Cpu) {
        if interrupt.is_master_enabled() && !cpu.is_processing_instruction() {
            let _interrupt_flag = match interrupt.get_interupt_state() {
                Some(flag) => flag,
                None => return
            };
            
            let interrupt_instr = Self::create_interupt_instruction();
            cpu.set_interrupt_instruction(interrupt_instr);
            cpu.halted = false; // TODO un-halting here should take an extra 4t cycles (see TCAGBD 4.9)
        }
    }

    fn get_interupt_vector(flag: InterruptFlag) -> u16 {
        match flag {
            InterruptFlag::VBlank => 0x40,
            InterruptFlag::Stat => 0x48,
            InterruptFlag::Timer => 0x50,
            InterruptFlag::Serial => 0x58,
            InterruptFlag::Joypad => 0x60
        }
    }

    // https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf
    // Section 4.9
    // It takes 20 clocks to dispatch an interrupt. And an extra 4 is the CPU is
    // in HALT mode.
    // Steps:
    // 8t: 2 NOPS 
    // 8t: current PC pushed to stack
    // 4t: PC set to the interupt handler adress

    fn create_interupt_instruction() -> Instruction {
        let mut steps: VecDeque<InstructionStep> = VecDeque::new();

        // NOP 1
        let step = Box::new(|_cpu: &mut Cpu| { });
        steps.push_back(InstructionStep::Standard(step));

        // NOP 2
        let step = Box::new(|_cpu: &mut Cpu| { });
        steps.push_back(InstructionStep::Standard(step));

        // push pc higher byte
        // latch interrupt enable flags?
        let step = Box::new(|cpu: &mut Cpu| {
            cpu.write_byte_to_stack((cpu.pc >> 8) as u8);
            cpu.temp_val8 = cpu.mmu.borrow().interupts.enable;
        });
        steps.push_back(InstructionStep::Standard(step));

        // push pc lower byte
        // latch interrupt (request) flags?
        let step = Box::new(|cpu: &mut Cpu| {
            let itr_if = cpu.mmu.borrow().interupts.flags;
            cpu.write_byte_to_stack((cpu.pc & 0x00FF) as u8);
            cpu.temp_val_16 = itr_if as u16;
        });
        steps.push_back(InstructionStep::Standard(step));

        // Set new PC
        let step = Box::new(move |cpu: &mut Cpu| {
            let address: u16;
            {
                let mut mmu = cpu.mmu.borrow_mut();
                let itr_state = mmu.interupts.get_interupt_state_latched(
                    cpu.temp_val8,
                    cpu.temp_val_16 as u8
                );

                address = match itr_state {
                    Some(flag) => {
                        mmu.interupts.clear_interupt(flag);
                        Self::get_interupt_vector(flag)
                    },
                    None => 0,
                };

                mmu.interupts.disable_master();
            }

            cpu.set_pc(address);
        });
        steps.push_back(InstructionStep::Standard(step));

        Instruction {
            opcode_val: 0,
            human_readable: String::from("Interupt Service Routine"),
            length: 0,
            steps
        }
    }
}