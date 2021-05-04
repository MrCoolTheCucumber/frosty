use std::{collections::VecDeque};

use crate::gameboy::cpu::{Flag, disassembler::disassembler_table::{ConditionCode, Register, RegisterPair1}};

use self::disassembler_table::ArithmeticOp;

use super::{Cpu};
mod disassembler_table;

// TODO: from line ~100 to ~300, improve usage of closures using the move keyword. op?

/*
    In this file we are algorithmicly parsing the opcodes, using the algorithm from the site below
    https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
*/

pub struct Instruction {
    pub opcode_val: u8,
    pub human_readable: String,
    pub length: u8, // bytes
    pub steps: VecDeque<InstructionStep> // bool is if we branch or not
}

pub enum InstructionStep {
    InstantConditional(Box<dyn Fn(&mut Cpu) -> bool>),
    Standard(Box<dyn Fn(&mut Cpu)>), // takes 4 clock cycles
    Instant(Box<dyn Fn(&mut Cpu)>)
}

pub(super) fn disassemble(opcode: u8) -> Instruction {
    let x = opcode >> 6;                // bits 6 - 7
    let y = (opcode & 0b00111000) >> 3; // bits 5 - 3
    let z = opcode & 0b00000111;        // bits 2 - 0
    let p = (opcode & 0b00110000) >> 4; // bits 5 - 4
    let q = (opcode & 0b00001000) >> 3; // bit 3

    let mut  instruction = match x {
        0 => dissassemble_x_0(y, z, p, q, opcode),
        1 => dissassemble_x_1(y, z, p, q, opcode),
        2 => dissassemble_x_2(y, z, p, q, opcode),
        3 => dissassemble_x_3(y, z, p, q, opcode),

        4..=u8::MAX => panic!("Unhandled opcode: {:#04X}", opcode)
    };

    let fake_opcode_fetch = InstructionStep::Standard(Box::new(|_cpu| { }));
    instruction.steps.push_front(fake_opcode_fetch); // fake step for fetching the opcode
    instruction 
}

fn push_fetch_operand8_closure(queue: &mut VecDeque<InstructionStep>) {
    let step = InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
        cpu.operand8 = cpu.fetch();
    }));
    queue.push_back(step);
}

fn push_fetch_operand16_closures(queue: &mut VecDeque<InstructionStep>) {
    push_fetch_operand8_closure(queue);
    
    let step = InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
        let higher_bits = cpu.fetch();
        cpu.operand16 = (higher_bits as u16) << 8 | cpu.operand8 as u16
    }));
    queue.push_back(step);
}

fn dissassemble_x_0(y: u8, z: u8, p: u8, q: u8, opcode: u8) -> Instruction {
    let mut steps: VecDeque<InstructionStep> = VecDeque::new();

    match z {
        0 => {
            match y {
                0 =>  {
                    let step = Box::new(|_cpu: &mut Cpu| { });
                    steps.push_back(InstructionStep::Standard(step));
                    Instruction {
                        opcode_val: opcode,
                        human_readable: String::from("NOP"),
                        length: 1,
                        steps
                    }
                }

                1 => {
                    push_fetch_operand16_closures(&mut steps);
                    steps.push_back(InstructionStep::Standard(Box::new(|cpu: &mut Cpu| { 
                        let val = (*cpu.mmu).borrow().read_byte(cpu.operand16);
                        cpu.sp = val as u16; 
                    })));
                    steps.push_back(InstructionStep::Standard(Box::new(|cpu: &mut Cpu| { 
                        let val = (*cpu.mmu).borrow().read_byte(cpu.operand16);
                        cpu.sp = cpu.sp | ((val as u16) << 8);
                    })));

                    Instruction {
                        opcode_val: opcode,
                        human_readable: String::from("LD u16, SP"),
                        length: 3,
                        steps
                    }
                }

                2 => {
                    steps.push_back(InstructionStep::Standard(Box::new(|_cpu: &mut Cpu| { 
                        panic!("STOP unimpl!"); 
                    })));
                    Instruction {
                        opcode_val: opcode,
                        human_readable: String::from("STOP"),
                        length: 1,
                        steps
                    }
                }

                3 => {
                    push_fetch_operand8_closure(&mut steps);
                    steps.push_back(InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                        let jmp_amount = cpu.operand8 as i8;
                        if jmp_amount < 0 {
                            cpu.pc = cpu.pc.wrapping_sub(jmp_amount.abs() as u16);
                        } else {
                            cpu.pc = cpu.pc.wrapping_add(jmp_amount as u16);
                        }
                    })));

                    Instruction {
                        opcode_val: opcode,
                        human_readable: String::from("JR i8"),
                        length: 2,
                        steps
                    }
                },

                4..=7 => {
                    push_fetch_operand8_closure(&mut steps);

                    let condition_code = ConditionCode::from_u8(y - 4);
                    let condition_closure = condition_code.generate_closure();
                    steps.push_back(InstructionStep::InstantConditional(Box::new(condition_closure)));
                    
                    steps.push_back(InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                        let jmp_amount = cpu.operand8 as i8;
                        if jmp_amount < 0 {
                            cpu.pc = cpu.pc.wrapping_sub(jmp_amount.abs() as u16);
                        } else {
                            cpu.pc = cpu.pc.wrapping_add(jmp_amount as u16);
                        }
                    })));

                    Instruction {
                        opcode_val: opcode,
                        human_readable: format!("JR {}, i8", condition_code.to_string()),
                        length: 2,
                        steps
                    }
                }

                8..=u8::MAX => panic!("This is impossible?")
            }
        }

        1 => {
            match q {
                // LD rp[p], nn
                0 => {
                    let reg_pair = RegisterPair1::from_u8(p);
                    match reg_pair {
                        RegisterPair1::BC => {
                            steps.push_back(InstructionStep::Standard(Box::new(|cpu| {
                                cpu.c = cpu.fetch();
                            })));
                            steps.push_back(InstructionStep::Standard(Box::new(|cpu| {
                                cpu.b = cpu.fetch();
                            })));
                        }
                        RegisterPair1::DE => {
                            steps.push_back(InstructionStep::Standard(Box::new(|cpu| {
                                cpu.e = cpu.fetch();
                            })));
                            steps.push_back(InstructionStep::Standard(Box::new(|cpu| {
                                cpu.d = cpu.fetch();
                            })));
                        }
                        RegisterPair1::HL => {
                            steps.push_back(InstructionStep::Standard(Box::new(|cpu| {
                                cpu.l = cpu.fetch();
                            })));
                            steps.push_back(InstructionStep::Standard(Box::new(|cpu| {
                                cpu.h = cpu.fetch();
                            })));
                        }
                        RegisterPair1::SP => {
                            steps.push_back(InstructionStep::Standard(Box::new(|cpu| {
                                let lower_bits = cpu.fetch();
                                cpu.sp = (cpu.sp << 8) | (lower_bits as u16)
                            })));
                            steps.push_back(InstructionStep::Standard(Box::new(|cpu| {
                                let higher_bits = cpu.fetch();
                                cpu.sp = ((higher_bits as u16) << 8) | cpu.sp;
                            })));
                        }
                    };
                    
                    Instruction {
                        opcode_val: opcode,
                        human_readable: format!("LD {}, u16", reg_pair.to_string()),
                        length: 3,
                        steps
                    }
                }

                1 => {
                    let reg_pair = RegisterPair1::from_u8(p);
                    let closure: Box<dyn Fn(&mut Cpu)> = Box::new(match reg_pair {
                        RegisterPair1::BC => |cpu| {
                            let result = cpu.add_two_reg_u16(cpu.hl(), cpu.bc());
                            cpu.set_hl(result);
                        },
                        RegisterPair1::DE => |cpu| {
                            let result = cpu.add_two_reg_u16(cpu.hl(), cpu.de());
                            cpu.set_hl(result);
                        },
                        RegisterPair1::HL => |cpu| {
                            let result = cpu.add_two_reg_u16(cpu.hl(), cpu.hl());
                            cpu.set_hl(result);
                        },
                        RegisterPair1::SP => |cpu| {
                            let result = cpu.add_two_reg_u16(cpu.hl(), cpu.pc);
                            cpu.set_hl(result);
                        }
                    });

                    steps.push_back(InstructionStep::Standard(closure));

                    Instruction {
                        opcode_val: opcode,
                        human_readable: format!("ADD HL, {}", reg_pair.to_string()),
                        length: 1,
                        steps
                    }
                }

                2..=u8::MAX => panic!("This is impossible?")
            }
        }

        2 => {
            match q {
                0 => {
                    let closure = match p {
                        0 => InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                            (*cpu.mmu).borrow_mut().write_byte(cpu.bc(), cpu.a);
                        })),
                        1 => InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                            (*cpu.mmu).borrow_mut().write_byte(cpu.de(), cpu.a);
                        })),
                        2 => InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                            (*cpu.mmu).borrow_mut().write_byte(cpu.hl(), cpu.a);
                            cpu.set_hl(cpu.hl().wrapping_add(1));
                        })),
                        3 => InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                            (*cpu.mmu).borrow_mut().write_byte(cpu.hl(), cpu.a);
                            cpu.set_hl(cpu.hl().wrapping_sub(1));
                        })),
                        _ => panic!("Invalid p value")
                    };
                    steps.push_back(closure);

                    let human_readable = match p {
                        0 => String::from("LD (BC), A"),
                        1 => String::from("LD (DE), A"),
                        2 => String::from("LD (HL+), A"),
                        3 => String::from("LD (HL-), A"),
                        _ => { panic!() } // this will never hit due to above
                    };

                    Instruction {
                        opcode_val: opcode,
                        human_readable,
                        length: 1,
                        steps
                    }
                }

                1 => {
                    let closure = match p {
                        0 => InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                            cpu.a = (*cpu.mmu).borrow_mut().read_byte(cpu.bc());
                        })),
                        1 => InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                            cpu.a = (*cpu.mmu).borrow_mut().read_byte(cpu.de());
                        })),
                        2 => InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                            cpu.a = (*cpu.mmu).borrow_mut().read_byte(cpu.hl());
                            cpu.set_hl(cpu.hl().wrapping_add(1));
                        })),
                        3 => InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                            cpu.a = (*cpu.mmu).borrow_mut().read_byte(cpu.hl());
                            cpu.set_hl(cpu.hl().wrapping_sub(1));
                        })),
                        _ => panic!("Invalid p value")
                    };
                    steps.push_back(closure);

                    let human_readable = match p {
                        0 => String::from("LD A, BC"),
                        1 => String::from("LD A, DE"),
                        2 => String::from("LD A, (HL+)"),
                        3 => String::from("LD A, (HL-)"),
                        _ => { panic!() } // this will never hit due to above
                    };

                    Instruction {
                        opcode_val: opcode,
                        human_readable,
                        length: 1,
                        steps
                    }
                }

                _ => panic!("Invalid q value")
            }
        }

        3 => {
            let reg_val = RegisterPair1::from_u8(p);
            let reg_val_string = reg_val.to_string();
            let is_inc = match q {
                0 => true,
                1 => false,
                _ => panic!("invalid q value!")
            };
            let op_fn = |cpu: &mut Cpu, setter: fn(&mut Cpu, val: u16), getter: fn(&Cpu) -> u16, is_inc: bool| {
                let result = match is_inc {
                    true => getter(cpu).wrapping_add(1),
                    false => getter(cpu).wrapping_sub(1)
                };
                setter(cpu, result);
            };

            let instruction_step = InstructionStep::Standard(Box::new(move |cpu| {
                match reg_val {
                    RegisterPair1::BC => op_fn(cpu, Cpu::set_bc, Cpu::bc, is_inc),
                    RegisterPair1::DE => op_fn(cpu, Cpu::set_de, Cpu::de, is_inc),
                    RegisterPair1::HL => op_fn(cpu, Cpu::set_hl, Cpu::hl, is_inc),
                    RegisterPair1::SP => op_fn(cpu, |cpu, val| cpu.sp = val, |cpu: &Cpu| -> u16 { cpu.sp }, is_inc),
                }
            }));

            steps.push_back(instruction_step);
            let prefix = if is_inc { String::from("INC") } else { String::from("DEC") };
            Instruction {
                opcode_val: opcode,
                human_readable: format!("{} {}", prefix, reg_val_string.to_string()),
                length: 1,
                steps
            }
        }

        4 | 5 => {
            let reg = Register::from_u8(y);
            let is_inc = z == 4;

            let op_fn = |cpu: &mut Cpu, operation: fn(&mut Cpu, val: u8) -> u8, register: Register| {
                match register {
                    Register::B => cpu.b = operation(cpu, cpu.b),
                    Register::C => cpu.c = operation(cpu, cpu.c),
                    Register::D => cpu.d = operation(cpu, cpu.d),
                    Register::E => cpu.e = operation(cpu, cpu.e),
                    Register::H => cpu.h = operation(cpu, cpu.h),
                    Register::L => cpu.l = operation(cpu, cpu.l),
                    Register::HLMem => {
                        let val = (*cpu.mmu).borrow().read_byte(cpu.hl());
                        cpu.temp_val8 = operation(cpu, val);
                    },
                    Register::A => cpu.a = operation(cpu, cpu.a),
                };
            };

            let operation = if is_inc { Cpu::inc } else { Cpu::dec };
            let op_str = if is_inc { String::from("INC") } else { String::from("DEC") };

            let instruction_step = match reg {
                Register::HLMem => InstructionStep::Standard(Box::new(move |cpu| { op_fn(cpu, operation, reg); })),
                _ => InstructionStep::Instant(Box::new(move |cpu| { op_fn(cpu, operation, reg); }))
            };

            steps.push_back(instruction_step);

            if matches!(reg, Register::HLMem) {
                let instruction = InstructionStep::Standard(Box::new(|cpu| {
                    (*cpu.mmu).borrow_mut().write_byte(cpu.hl(), cpu.temp_val8);
                }));

                steps.push_back(instruction)
            }

            Instruction {
                opcode_val: opcode,
                human_readable: format!("{} {}", op_str, reg.to_string()),
                length: if matches!(reg, Register::HLMem) { 12 } else { 4 },
                steps
            }
        }

        6 => {
            push_fetch_operand8_closure(&mut steps);
            let reg = Register::from_u8(y);

            let closure = Box::new(move |cpu: &mut Cpu| {
                match reg {
                    Register::B => cpu.b = cpu.operand8,
                    Register::C => cpu.c = cpu.operand8,
                    Register::D => cpu.d = cpu.operand8,
                    Register::E => cpu.e = cpu.operand8,
                    Register::H => cpu.h = cpu.operand8,
                    Register::L => cpu.l = cpu.operand8,
                    Register::HLMem => (*cpu.mmu).borrow_mut().write_byte(cpu.hl(), cpu.operand8),
                    Register::A => cpu.a = cpu.operand8,
                }
            });

            let instruction_step = match reg {
                Register::HLMem => InstructionStep::Standard(closure),
                _ => InstructionStep::Instant(closure)
            };

            steps.push_back(instruction_step);

            Instruction {
                opcode_val: opcode,
                human_readable: format!("LD {}", reg.to_string()),
                length: if matches!(reg, Register::HLMem) { 12 } else { 8 },
                steps
            }
        }

        7 => {
            let human_readable = match y {
                // RLCA
                0 => {
                    let instruction_step = InstructionStep::Instant(Box::new(|cpu| {
                        let carry = (cpu.a & 0x80) >> 7;
                        cpu.set_flag_if_cond_else_clear(carry != 0, Flag::C);

                        cpu.a = (cpu.a << 1).wrapping_add(carry);

                        cpu.clear_flag(Flag::N);
                        cpu.clear_flag(Flag::Z);
                        cpu.clear_flag(Flag::H);
                    }));
                    steps.push_back(instruction_step);
                    String::from("RLCA")
                }

                1 => {
                    let instruction_step = InstructionStep::Instant(Box::new(|cpu| {
                        let carry = cpu.a & 0b00000001 > 0;
                        cpu.a = cpu.a >> 1;
                        if carry { cpu.a = cpu.a | 0b10000000; }
                        
                        cpu.set_flag_if_cond_else_clear(carry, Flag::C);
                        cpu.clear_flag(Flag::Z);
                        cpu.clear_flag(Flag::N);
                        cpu.clear_flag(Flag::H);
                    }));
                    steps.push_back(instruction_step);
                    String::from("RRCA")
                }

                2 => {
                    let instruction_step = InstructionStep::Instant(Box::new(|cpu| {
                        let is_carry_set = cpu.is_flag_set(Flag::C);
                        cpu.set_flag_if_cond_else_clear(cpu.a & 0x80 > 0, Flag::C);

                        cpu.a = cpu.a << 1;
                        if is_carry_set { cpu.a += 1 };

                        cpu.clear_flag(Flag::N);
                        cpu.clear_flag(Flag::Z);
                        cpu.clear_flag(Flag::H);
                    }));
                    steps.push_back(instruction_step);
                    String::from("RLA")
                }

                3 => {
                    let instruction_step = InstructionStep::Instant(Box::new(|cpu| {
                        let carry = if cpu.is_flag_set(Flag::C) {1 << 7} else {0};
                        cpu.set_flag_if_cond_else_clear(cpu.a & 0x01 != 0, Flag::C);

                        cpu.a = (cpu.a >> 1).wrapping_add(carry);

                        cpu.clear_flag(Flag::N);
                        cpu.clear_flag(Flag::Z);
                        cpu.clear_flag(Flag::H);
                    }));
                    steps.push_back(instruction_step);
                    String::from("RRA")
                }

                _ => panic!("Unhandled opcode: {:#04X}", opcode)
            };

            Instruction {
                opcode_val: opcode,
                human_readable,
                length: 1,
                steps
            }
        }

        _ => panic!("Unhandled opcode: {:#04X}", opcode)
    }
}

fn dissassemble_x_1(y: u8, z: u8, _p: u8, _q: u8, opcode: u8) -> Instruction {
    if opcode == 0x76 {
        panic!("HALT not implemented");
    }

    let mut steps: VecDeque<InstructionStep> = VecDeque::new();

    let destination_reg = Register::from_u8(y);
    let src_val_reg = Register::from_u8(z);

    let fetch_closure = Box::new(move |cpu: &mut Cpu| {
        cpu.temp_val8 = match src_val_reg {
            Register::B => cpu.b,
            Register::C => cpu.c,
            Register::D => cpu.d,
            Register::E => cpu.e,
            Register::H => cpu.h,
            Register::L => cpu.l,
            Register::HLMem => (*cpu.mmu).borrow().read_byte(cpu.hl()),
            Register::A => cpu.a
        }
    });

    let fetch_instruction_step = match src_val_reg {
        Register::HLMem => InstructionStep::Standard(fetch_closure),
        _ => InstructionStep::Instant(fetch_closure)
    };

    let assign_closure = Box::new(move |cpu: &mut Cpu| {
        match destination_reg {
            Register::B => cpu.b = cpu.temp_val8,
            Register::C => cpu.c = cpu.temp_val8,
            Register::D => cpu.d = cpu.temp_val8,
            Register::E => cpu.e = cpu.temp_val8,
            Register::H => cpu.h = cpu.temp_val8,
            Register::L => cpu.l = cpu.temp_val8,
            Register::HLMem => (*cpu.mmu).borrow_mut().write_byte(cpu.hl(), cpu.temp_val8),
            Register::A => cpu.a = cpu.temp_val8
        }
    });

    let assign_instruction_step = match destination_reg {
        Register::HLMem => InstructionStep::Standard(assign_closure),
        _ => InstructionStep::Instant(assign_closure)
    };

    steps.push_back(fetch_instruction_step);
    steps.push_back(assign_instruction_step);

    Instruction {
        opcode_val: opcode,
        human_readable: format!("LD {}, {}", destination_reg.to_string(), src_val_reg.to_string()),
        length: 1,
        steps
    }
}

fn dissassemble_x_2(y: u8, z: u8, _p: u8, _q: u8, opcode: u8) -> Instruction {
    let mut steps: VecDeque<InstructionStep> = VecDeque::new();
    let arithmetic_op = ArithmeticOp::from_u8(y);
    let register_operand = Register::from_u8(z);

    let fetch_op_arg_closure = Box::new(move |cpu: &mut Cpu| {
        match register_operand {
            Register::B => cpu.temp_val8 = cpu.b,
            Register::C => cpu.temp_val8 = cpu.c,
            Register::D => cpu.temp_val8 = cpu.d,
            Register::E => cpu.temp_val8 = cpu.e,
            Register::H => cpu.temp_val8 = cpu.h,
            Register::L => cpu.temp_val8 = cpu.l,
            Register::HLMem => cpu.temp_val8 = (*cpu.mmu).borrow().read_byte(cpu.hl()),
            Register::A => cpu.temp_val8 = cpu.a
        }
    });

    let fetch_instruction_step = match register_operand {
        Register::HLMem => InstructionStep::Standard(fetch_op_arg_closure),
        _ => InstructionStep::Instant(fetch_op_arg_closure)
    };

    let arithmetic_closure = Box::new(move |cpu: &mut Cpu| {
        match arithmetic_op {
            ArithmeticOp::ADD => cpu.a = cpu.add(cpu.a, cpu.temp_val8),
            ArithmeticOp::ADC => cpu.adc(cpu.temp_val8),
            ArithmeticOp::SUB => cpu.sub(cpu.temp_val8),
            ArithmeticOp::SBC => panic!("SBC op not implemented!"),
            ArithmeticOp::AND => cpu.and(cpu.temp_val8),
            ArithmeticOp::XOR => cpu.xor(cpu.temp_val8),
            ArithmeticOp::OR =>  cpu.or(cpu.temp_val8),
            ArithmeticOp::CP =>  cpu.cp(cpu.temp_val8)
        }
    });

    let arithmethic_instruction_step = InstructionStep::Instant(arithmetic_closure);

    steps.push_back(fetch_instruction_step);
    steps.push_back(arithmethic_instruction_step);

    Instruction {
        opcode_val: opcode,
        human_readable: format!("{}, {}", arithmetic_op.to_string(), register_operand.to_string()),
        length: if matches!(register_operand, Register::HLMem) { 8 } else { 4 },
        steps
    }
}

fn dissassemble_x_3(y: u8, z: u8, _p: u8, _q: u8, opcode: u8) -> Instruction {
    let mut steps: VecDeque<InstructionStep> = VecDeque::new();

    match z {
        0 => {
            match y {
                // RET CC
                // 8t without branch, 20 with
                0 | 1 | 2 | 3 => {
                    // 4
                    let condition_code = ConditionCode::from_u8(y);
                    // 4
                    let instruction_step = InstructionStep::Standard(Box::new(|cpu| {
                        cpu.pc = cpu.operand16;
                    }));
                    steps.push_back(InstructionStep::InstantConditional(Box::new(condition_code.generate_closure())));
                    steps.push_back(instruction_step);

                    // 4
                    steps.push_back(InstructionStep::Standard(Box::new(|_cpu|{})));
                    // 4
                    steps.push_back(InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                        cpu.temp_val_16 = cpu.read_word_from_stack();
                    })));
                    // 4
                    steps.push_back(InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                        cpu.pc = cpu.temp_val_16;
                    })));

                    Instruction {
                        opcode_val: opcode,
                        human_readable: format!("RET {}", condition_code.to_string()),
                        length: 1,
                        steps
                    }
                },

                4 => {
                    push_fetch_operand8_closure(&mut steps);
                    steps.push_back(InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                        (*cpu.mmu).borrow_mut().write_byte(0xFF00 + (cpu.operand8 as u16), cpu.a);
                    })));

                    Instruction {
                        opcode_val: opcode,
                        human_readable: String::from("LD (0xFF00 + u8), A"),
                        length: 2,
                        steps
                    }
                }

                // TODO THIS OPCODE SETS FLAGS!!!!!!!!!!!!!!!!!!!!!!!!
                // 5 => {
                //     push_fetch_operand8_closure(&mut steps);
                //     steps.push_back(InstructionStep::Standard(Box::new(|cpu|{})));
                //     steps.push_back(InstructionStep::Standard(Box::new(|cpu: &mut Cpu|{ 
                //         cpu.sp.wrapping_add(cpu.operand8 as u16);
                //     })));

                //     Instruction {
                //         opcode_val: opcode,
                //         human_readable: String::from("ADD SP, i8"),
                //         length: 2,
                //         steps
                //     }
                // },

                6 => {
                    push_fetch_operand8_closure(&mut steps);
                    steps.push_back(InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                        cpu.a = (*cpu.mmu).borrow_mut().read_byte(0xFF00 + (cpu.operand8 as u16));
                    })));

                    Instruction {
                        opcode_val: opcode,
                        human_readable: String::from("LD A, (0xFF00 + u8)"),
                        length: 2,
                        steps
                    }
                }

                // THIS OPCODE SETS FLAGS!!!
                // 7 => {

                // }

                _ => panic!("Unhandled opcode: {:#04X}", opcode)
            }
        }

        2 => {
            match y {
                // JP CC, u16
                // 12t without branch, 16
                0 | 1 | 2 | 3 => {
                    // 12th t
                    push_fetch_operand16_closures(&mut steps);
                    
                    let cond_code = ConditionCode::from_u8(y);
                    let cond_closure = cond_code.generate_closure();
                    // instant
                    steps.push_back(InstructionStep::InstantConditional(Box::new(cond_closure)));

                    let jump_step = InstructionStep::Standard(Box::new(|cpu| {
                        cpu.pc = cpu.operand16;
                    }));
                    // 16th t
                    steps.push_back(jump_step);

                    Instruction {
                        opcode_val: opcode,
                        human_readable: format!("JP {} u16", cond_code.to_string()),
                        length: 3,
                        steps
                    }
                }

                _ => panic!("Unhandled opcode: {:#04X}", opcode)
            }
        }

        3 => {
            match y {
                0 => {
                    push_fetch_operand16_closures(&mut steps);
                    let instruction_step = InstructionStep::Standard(Box::new(|cpu| {
                        cpu.pc = cpu.operand16;
                    }));
                    steps.push_back(instruction_step);

                    Instruction {
                        opcode_val: opcode,
                        human_readable: String::from("JP u16"),
                        length: 3,
                        steps
                    }
                }

                6 => {
                    let instruction_step = InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                        (*cpu.mmu).borrow_mut().interupts.disable_master();
                    }));
                    steps.push_back(instruction_step);

                    Instruction {
                        opcode_val: opcode,
                        human_readable: String::from("DI"),
                        length: 1,
                        steps
                    }
                }

                7 => {
                    let instruction_step = InstructionStep::Standard(Box::new(|cpu: &mut Cpu| {
                        (*cpu.mmu).borrow_mut().interupts.enable_master();
                    }));
                    steps.push_back(instruction_step);

                    Instruction {
                        opcode_val: opcode,
                        human_readable: String::from("EI"),
                        length: 1,
                        steps
                    }
                }

                _ => panic!("Unhandled opcode: {:#04X}", opcode)
            }
        }

        _ => panic!("Unhandled opcode: {:#04X}", opcode)
    }
}
