use std::{cell::RefCell, fmt, fs::File, io::Write, rc::Rc, time::{SystemTime, UNIX_EPOCH}};
use crate::gameboy::cpu::disassembler::disassemble_cb_prefix_op;

use self::disassembler::{Instruction, InstructionStep, disassemble};
use super::mmu::Mmu;

pub mod disassembler;

enum Flag {
    Z = 0b10000000,
    N = 0b01000000, // N = last math op was subtract
    H = 0b00100000,
    C = 0b00010000
}

pub struct Cpu {
    mmu: Rc<RefCell<Mmu>>,

    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8, 
    h: u8,
    l: u8,

    pc: u16,
    sp: u16,

    operand8: u8,
    operand16: u16,
    temp_val8: u8,
    temp_val_16: u16,

    instruction: Option<Instruction>,
    machine_cycles_taken_for_current_step: u8,

    pub stopped: bool,
    pub halted: bool,
    pub halted_waiting_for_interupt_pending: bool,
    halt_bug: bool,
    ei_delay: bool,

    debug: bool,
    pub start_log: bool,
    log: Option<File>
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("cpu")
            .field("af", &format!("{:#06X}", self.af()))
            .field("bc", &format!("{:#06X}", self.bc()))
            .field("de", &format!("{:#06X}", self.de()))
            .field("hl", &format!("{:#06X}", self.hl()))
            
            .field("sp", &format!("{:#06X}", self.sp))
            .field("pc", &format!("{:#06X}", self.pc))

            .finish()
    }
}

impl Cpu {
    pub fn new(mmu: Rc<RefCell<Mmu>>) -> Self {
        let log = false;
        let mut file: Option<File> = None;
        if log {
            file = Some(File::create("I:\\Dev\\gameboy-rs\\log.txt").unwrap());
        }

        Self {
            mmu,

            a: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            f: 0x00,
            h: 0x00,
            l: 0x00,

            pc: 0x0,
            sp: 0x0,

            operand8: 0,
            operand16: 0,
            temp_val8: 0,
            temp_val_16: 0,

            instruction: None,
            machine_cycles_taken_for_current_step: 0,

            stopped: false,
            halted: false,
            halted_waiting_for_interupt_pending: false,
            halt_bug: false,
            ei_delay: false,

            debug: true,
            start_log: false,
            log: file
        }
    }

    pub fn is_processing_instruction(&self) -> bool {
        self.instruction.is_some()
    }

    pub fn set_interrupt_instruction(&mut self, instruction: Instruction) {
        self.instruction = Some(instruction);
    }

    // FLAG FUNCS
    #[inline]
    fn set_flag(&mut self, flag: Flag) {
        self.f = self.f | flag as u8;
    }

    #[inline]
    fn clear_flag(&mut self, flag: Flag) {
        self.f = self.f & !(flag as u8); 
    }

    #[inline]
    fn is_flag_set(&self, flag: Flag) -> bool {
        self.f & (flag as u8) > 0
    }

    #[inline]
    fn set_flag_if_cond_else_clear(&mut self, cond: bool, flag: Flag) {
        if cond {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    #[inline]
    fn handle_zero_flag(&mut self, register: u8) {
        if register == 0 {
            self.set_flag(Flag::Z);
        } else {
            self.clear_flag(Flag::Z);
        }
    }

    fn inc(&mut self, val: u8) -> u8 {
        self.set_flag_if_cond_else_clear((val & 0x0F) == 0x0F, Flag::H);

        let result: u8 = val.wrapping_add(1);

        self.handle_zero_flag(result);
        self.clear_flag(Flag::N);
        result
    }

    fn cp(&mut self, val: u8) {
        self.set_flag_if_cond_else_clear(self.a == val, Flag::Z);
        self.set_flag_if_cond_else_clear(val > self.a, Flag::C);
        self.set_flag_if_cond_else_clear(
            (val & 0xF) > (self.a & 0x0F), 
            Flag::H
        );
        self.set_flag(Flag::N);
    }

    // ARITHMETIC
    fn dec(&mut self, val: u8) -> u8 {
        let result: u8 = val.wrapping_sub(1);

        self.set_flag_if_cond_else_clear((val & 0x0F) == 0, Flag::H);
        self.handle_zero_flag(result);
        self.set_flag(Flag::N);
        result
    }

    fn or(&mut self, val: u8) {
        self.a = self.a | val;

        self.handle_zero_flag(self.a);
        self.clear_flag(Flag::C);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);
    }

    fn xor(&mut self, val: u8) {
        self.a = self.a ^ val;

        self.handle_zero_flag(self.a);
        self.clear_flag(Flag::C);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);
    }

    fn and(&mut self, val: u8) {
        self.a = self.a & val;

        self.handle_zero_flag(self.a);
        self.clear_flag(Flag::C);
        self.clear_flag(Flag::N);
        self.set_flag(Flag::H);
    }

    fn add(&mut self, val1: u8, val2: u8) -> u8 {
        let result = val1.wrapping_add(val2);

        self.clear_flag(Flag::N);
        self.set_flag_if_cond_else_clear(
            val1.checked_add(val2).is_none(),
            Flag::C
        );
        self.handle_zero_flag(result);

        let is_half_carry = (((val1 & 0x0F)) + (val2 & 0x0F)) > 0x0F;
        self.set_flag_if_cond_else_clear(is_half_carry, Flag::H);

        result
    }

    fn adc(&mut self, val: u8) {
        let carry: u8 = if self.is_flag_set(Flag::C) {1} else {0};

        let is_half_carry = ((self.a & 0x0F) + (val & 0x0F) + carry) & 0x10 == 0x10;
        let is_carry = ((self.a as u16) + (val as u16) + (carry as u16)) & 0x100 == 0x100;

        let result = self.a.wrapping_add(val).wrapping_add(carry);

        self.handle_zero_flag(result);
        self.set_flag_if_cond_else_clear(is_half_carry, Flag::H);
        self.set_flag_if_cond_else_clear(is_carry, Flag::C);

        self.clear_flag(Flag::N);
        self.a = result;
    }

    fn sub(&mut self, val: u8) {
        self.set_flag_if_cond_else_clear(val > self.a, Flag::C);
        self.set_flag_if_cond_else_clear((val & 0x0F) > (self.a & 0x0F), Flag::H);

        self.a = self.a.wrapping_sub(val);

        self.handle_zero_flag(self.a);
        self.set_flag(Flag::N);
    }

    fn sbc(&mut self, val: u8) {
        let carry: u8 = if self.is_flag_set(Flag::C) {1} else {0};

        let is_half_carry = 
            ((self.a & 0x0F) as i16) -
            ((val & 0x0F) as i16) -
            (carry as i16) < 0;

        let is_full_carry = 
            (self.a as i16) -
            (val as i16) - 
            (carry as i16) < 0;

        let result = self.a.wrapping_sub(val).wrapping_sub(carry);

        self.handle_zero_flag(result);
        self.set_flag_if_cond_else_clear(is_half_carry, Flag::H);
        self.set_flag_if_cond_else_clear(is_full_carry, Flag::C);

        self.set_flag(Flag::N);
        self.a = result;
    }

    fn add_hl_r16(&mut self, hl: u16, r16: u16) -> u16 {
        let result = hl.wrapping_add(r16);

        self.clear_flag(Flag::N);

        let overflowed = hl.checked_add(r16).is_none();
        self.set_flag_if_cond_else_clear(overflowed, Flag::C);

        let half_carry_occured = ((hl & 0xFFF) + (r16 & 0xFFF)) > 0xFFF;
        self.set_flag_if_cond_else_clear(half_carry_occured, Flag::H);

        result
    }

    // CB ARITHMETIC

    fn rlc(&mut self, val: u8) -> u8 {
        self.set_flag_if_cond_else_clear((val & 0x80) != 0, Flag::C);
        
        let carry = (val & 0x80) >> 7;
        let result = (val << 1).wrapping_add(carry);

        self.handle_zero_flag(val);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn rrc(&mut self, val: u8) -> u8 {
        let carry = val & 0x01;
        let mut result = val >> 1;

        if carry != 0 {
            self.set_flag(Flag::C);
            result = result | 0x80;
        } 
        else { self.clear_flag(Flag::C); }

        self.handle_zero_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn rl(&mut self, val: u8) -> u8 {
        let carry = if self.is_flag_set(Flag::C)
            { 1 } else { 0 };
        let result = (val << 1).wrapping_add(carry);

        self.set_flag_if_cond_else_clear(val & 0x80 != 0, Flag::C);
        self.handle_zero_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn rr(&mut self, val: u8) -> u8 {
        let mut result = val >> 1;
        if self.is_flag_set(Flag::C) {
            result = result | 0x80;
        }

        self.set_flag_if_cond_else_clear(val & 0x01 != 0, Flag::C);
        self.handle_zero_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn sla(&mut self, val: u8) -> u8 {
        let result = val << 1;

        self.set_flag_if_cond_else_clear(val & 0x80 != 0, Flag::C);
        self.handle_zero_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn sra(&mut self, val: u8) -> u8 {
        let result = (val & 0x80) | (val >> 1);

        self.set_flag_if_cond_else_clear(val & 0x01 != 0, Flag::C);
        self.handle_zero_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn swap(&mut self, val: u8) -> u8 {
        let result = ((val & 0x0F) << 4) | ((val & 0xF0) >> 4);

        self.handle_zero_flag(result);
        self.clear_flag(Flag::C);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn srl(&mut self, val: u8) -> u8 {
        let result = val >> 1;

        self.set_flag_if_cond_else_clear(val & 0x01 != 0, Flag::C);
        self.handle_zero_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn bit(&mut self, bit: u8, val: u8) {
        self.set_flag_if_cond_else_clear(val & (1 << bit) == 0, Flag::Z);
        self.clear_flag(Flag::N);
        self.set_flag(Flag::H);
    }

    // 16bit REGISTER HELPER FUNCS

    fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }

    fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    fn set_af(&mut self, val: u16) {
        // a = high bits, f = low bits
        self.a = ((val & 0xFF00) >> 8) as u8;
        self.f = (val & 0x00FF) as u8; // not sure if we need to & here
    }

    fn set_bc(&mut self, val: u16) {
        self.b = ((val & 0xFF00) >> 8) as u8;
        self.c = (val & 0x00FF) as u8;
    }

    fn set_de(&mut self, val: u16) {
        self.d = ((val & 0xFF00) >> 8) as u8;
        self.e = (val & 0x00FF) as u8;
    }

    fn set_hl(&mut self, val: u16) {
        self.h = ((val & 0xFF00) >> 8) as u8;
        self.l = (val & 0x00FF) as u8;
    }

    fn fetch(&mut self) -> u8 {
        let op = (*self.mmu).borrow().read_byte(self.pc);
        self.pc += 1;
        op
    }

    // STACK FUNCTIONS

    pub(super) fn push_pc_to_stack(&mut self) {
        self.write_word_to_stack(self.pc);
    }

    fn write_word_to_stack(&mut self, val: u16) {
        // self.sp -= 2;
        self.sp = self.sp.wrapping_sub(2);
        (*self.mmu).borrow_mut().write_word(self.sp, val);
    }

    fn write_byte_to_stack(&mut self, val: u8) {
        self.sp = self.sp.wrapping_sub(1);
        (*self.mmu).borrow_mut().write_byte(self.sp, val);
    }

    fn read_word_from_stack(&mut self) -> u16 {
        let val: u16 = (*self.mmu).borrow().read_word(self.sp);
        self.sp += 2;
        val
    }

    fn read_byte_from_stack(&mut self) -> u8 {
        let val: u8 = (*self.mmu).borrow().read_byte(self.sp);
        self.sp += 1;
        val
    }

    // MISC

    pub(super) fn set_pc(&mut self, pc: u16) {
        self.pc = pc;
    }

    // CYCLE FUNCTIONS

    pub fn tick(&mut self) {
        if self.halted { 
            if self.halted_waiting_for_interupt_pending {
                let mut mmu = (*self.mmu).borrow_mut();
                
                if !mmu.interupts.halt_interupt_pending { return }

                mmu.interupts.halt_interupt_pending = false;
                mmu.interupts.waiting_for_halt_if = false;
                self.halted_waiting_for_interupt_pending = false;
                self.halted = false;
            } 
            else {
                return
            }
        }

        if self.instruction.is_none() {
            let opcode = self.fetch();
            
            {
                let mut mmu = self.mmu.borrow_mut();
                if mmu.bios_enabled && self.pc >= 0x100{
                    mmu.bios_enabled = false;
                }
            }

            if self.halt_bug {
                self.pc -= 1;
                self.halt_bug = false;
            }

            let instruction = match opcode {
                0xCB => disassemble_cb_prefix_op(self.fetch()),
                _ => disassemble(opcode)
            };

            if self.ei_delay {
                self.ei_delay = false;
                (*self.mmu).borrow_mut().interupts.enable_master();
            }

            if self.start_log {
                let mut instr_human_readable = instruction.human_readable.clone();

                if instr_human_readable.contains("u8") {
                    let op8 = (*self.mmu).borrow().read_byte(self.pc);
                    instr_human_readable = instr_human_readable.replace("u8", format!("{:#04X}", op8).as_ref());
                }
                else if instr_human_readable.contains("i8") {
                    let op8 = (*self.mmu).borrow().read_byte(self.pc) as i8;
                    instr_human_readable = instr_human_readable.replace("i8", format!("{:#04X}", op8).as_ref());
                } 
                else if instr_human_readable.contains("u16") {
                    let op16 = (*self.mmu).borrow().read_word(self.pc);
                    instr_human_readable = instr_human_readable.replace("u16", format!("{:#06X}", op16).as_ref());
                }

                
                let s = format!("PC:{:#06X} OP:{:#04X} {}", self.pc - 1, opcode, instr_human_readable);
                // println!("{}", s);
                let ly = self.mmu.borrow_mut().io[0x44];
                if self.log.is_some() {
                    self.log.as_ref().unwrap().write(format!("{} \n", s).as_ref()).unwrap();
                    self.log.as_ref().unwrap().write(format!("LY: {:#04X}, {:?} \n", ly, self).as_ref()).unwrap();
                    self.log.as_ref().unwrap().write("\n".as_ref()).unwrap();
                }
            }

            self.machine_cycles_taken_for_current_step += 1;
            self.instruction = Some(instruction);
            return;
        }

        self.machine_cycles_taken_for_current_step += 1;
        if self.machine_cycles_taken_for_current_step < 4 {
            return;
        }

        self.machine_cycles_taken_for_current_step = 0;

        let step;
        {
            let instruction = self.instruction.as_mut().unwrap();
            step = instruction.steps.pop_front().unwrap();
        }

        match step {
            InstructionStep::Standard(func) => {    
                func(self);                        
                // get the next step if possible
                self.handle_next_step();
            }

            InstructionStep::Instant(_) | InstructionStep::InstantConditional(_) => 
                panic!("We just waited to exec an instant step, the logic is bricked?")
        }
    }

    fn handle_next_step(&mut self) {
        let instruction_step_peek;
        {
            let instruction = self.instruction.as_mut().unwrap();
            
            // was this the last step
            if instruction.steps.is_empty() {
                self.instruction = None;
                return;
            }

            // peek next step
            instruction_step_peek = instruction.steps.front().unwrap();
        }

        // TODO: re-write code the below to:
        // inside a loop, peek the next instruction, if instant, then pop and execute, else return
        // current seems logic seems to work fine though

        let mut instruction_step;
            match instruction_step_peek {
                InstructionStep::Instant(_) | InstructionStep::InstantConditional(_) => {
                    let instruction = self.instruction.as_mut().unwrap();
                    instruction_step = instruction.steps.pop_front().unwrap();
                }

                _ => return
            }

        // loop through the steps incase there are multiple instant steps
        // (there technically should never be as they could just be defined as one?)
        loop {
            match &instruction_step {
                InstructionStep::Instant(func) => {
                    func(self);
                }

                InstructionStep::InstantConditional(func) => {
                    let branch = func(self);
                    if !branch {
                        self.instruction = None;
                        return;
                    }
                }

                _ => break
            }

            let instruction = self.instruction.as_mut().unwrap();
            if instruction.steps.is_empty() {
                self.instruction = None;
                return;
            }

            instruction_step = instruction.steps.pop_front().unwrap();
        }

        // put the instruction pack into the queue, at the front
        self.instruction.as_mut().unwrap().steps.push_front(instruction_step);
    }
}
