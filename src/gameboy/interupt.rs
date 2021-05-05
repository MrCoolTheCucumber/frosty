use std::{collections::VecDeque, fmt};

use super::{cpu::{Cpu, disassembler::{Instruction, InstructionStep}}};

// https://eldred.fr/gb-asm-tutorial/interrupts.html

pub struct Interupt {
    pub master: u8,
    pub enable: u8,
    pub flags: u8
}

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
            master: 1,
            enable: 0,
            flags: 0
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
        if self.enable > 0 && self.flags > 0 {
            let interupt: u8 = self.enable & self.flags & 0x1F;

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
    }

    pub fn handle(interrupt: &mut Interupt, cpu: &mut Cpu) {
        if interrupt.is_master_enabled() && !cpu.is_processing_instruction() {
            let interrupt_flag = match interrupt.get_interupt_state() {
                Some(flag) => flag,
                None => return
            };
            
            let interrupt_addr: u16 = match interrupt_flag {
                InterruptFlag::VBlank => 0x40,
                InterruptFlag::Stat => 0x48,
                InterruptFlag::Timer => 0x50,
                InterruptFlag::Serial => 0x58,
                InterruptFlag::Joypad => 0x60
            };

            interrupt.clear_interupt(interrupt_flag);
            interrupt.disable_master();

            let interrupt_instr = Self::create_interupt_instruction(interrupt_addr);
            cpu.set_interrupt_instruction(interrupt_instr);
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

    fn create_interupt_instruction(addr: u16) -> Instruction {
        let mut steps: VecDeque<InstructionStep> = VecDeque::new();

        // NOP 1
        let step = Box::new(|_cpu: &mut Cpu| { });
        steps.push_back(InstructionStep::Standard(step));

        // NOP 2
        let step = Box::new(|_cpu: &mut Cpu| { });
        steps.push_back(InstructionStep::Standard(step));

        // NOP 3 (fake nop)
        let step = Box::new(|_cpu: &mut Cpu| { });
        steps.push_back(InstructionStep::Standard(step));

        // Push PC to stack
        let step = Box::new(|cpu: &mut Cpu| { cpu.push_pc_to_stack(); });
        steps.push_back(InstructionStep::Standard(step));

        // Push PC to stack
        let step = Box::new(move |cpu: &mut Cpu| { cpu.set_pc(addr); });
        steps.push_back(InstructionStep::Standard(step));

        Instruction {
            opcode_val: 0,
            human_readable: String::from("Interupt Service Routine"),
            length: 0,
            steps
        }
    }
}