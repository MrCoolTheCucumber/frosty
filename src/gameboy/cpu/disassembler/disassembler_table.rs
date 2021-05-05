use std::fmt;
use fmt::Display;

use crate::gameboy::cpu::{Cpu, Flag};

/*
    ================
     CONDITION CODE
    ================
*/

pub(super) enum ConditionCode {
    NotZero  = 0,
    Zero     = 1,
    NotCarry = 2,
    Carry    = 3
}

impl Display for ConditionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConditionCode::NotZero => { write!(f, "NZ") }
            ConditionCode::Zero => { write!(f, "Z") }
            ConditionCode::NotCarry => { write!(f, "NC") }
            ConditionCode::Carry => { write!(f, "C") }
        }
    }
}

impl ConditionCode {
    pub(super) fn from_u8(index: u8) -> Self {
        match index {
            0 => Self::NotZero,
            1 => Self::Zero,
            2 => Self::NotCarry,
            3 => Self::Carry,
            _ => panic!("Invalid ConditionCode index!")
        }
    }

    pub(super) fn generate_closure(&self) -> fn(&mut Cpu) -> bool {
        match self {
            ConditionCode::NotZero  => |cpu| { !cpu.is_flag_set(Flag::Z) },
            ConditionCode::Zero     => |cpu| { cpu.is_flag_set(Flag::Z) },
            ConditionCode::NotCarry => |cpu| { !cpu.is_flag_set(Flag::C) },
            ConditionCode::Carry    => |cpu| { cpu.is_flag_set(Flag::C) }
        }
    }
}

/*
    =================
     Register Pair 1
    =================
*/

pub(super) enum RegisterPair1 {
    BC = 0,
    DE = 1,
    HL = 2,
    SP = 3
}

impl Display for RegisterPair1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegisterPair1::BC => write!(f, "BC"),
            RegisterPair1::DE => write!(f, "DE"),
            RegisterPair1::HL => write!(f, "HL"),
            RegisterPair1::SP => write!(f, "SP")
        }
    }
}

impl RegisterPair1 {
    pub(super) fn from_u8(index: u8) -> Self {
        match index {
            0 => Self::BC,
            1 => Self::DE,
            2 => Self::HL,
            3 => Self::SP,
            _ => panic!("Invalid RegisterPair1 index")
        }
    }
}

/*
    =================
     Register Pair 2
    =================
*/

#[derive(Copy, Clone)]
pub(super) enum RegisterPair2 {
    BC = 0,
    DE = 1,
    HL = 2,
    AF = 3
}

impl Display for RegisterPair2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegisterPair2::BC => write!(f, "BC"),
            RegisterPair2::DE => write!(f, "DE"),
            RegisterPair2::HL => write!(f, "HL"),
            RegisterPair2::AF => write!(f, "AF")
        }
    }
}

impl RegisterPair2 {
    pub(super) fn from_u8(index: u8) -> Self {
        match index {
            0 => Self::BC,
            1 => Self::DE,
            2 => Self::HL,
            3 => Self::AF,
            _ => panic!("Invalid RegisterPair2 index")
        }
    }
}

/*
    =================
     8 bit Registers
    =================
*/

#[derive(Copy, Clone)]
pub(super) enum Register {
    B = 0,
    C = 1,
    D = 2,
    E = 3,
    H = 4,
    L = 5,
    HLMem = 6, // (HL) = access byte at mem location HL
    A = 7
}

impl Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
            Self::E => write!(f, "E"),
            Self::H => write!(f, "H"),
            Self::L => write!(f, "L"),
            Self::HLMem => write!(f, "(HL)"),
            Self::A => write!(f, "A"),
        }
    }
}

impl Register {
    pub(super) fn from_u8(index: u8) -> Self {
        match index {
            0 => Self::B,
            1 => Self::C,
            2 => Self::D,
            3 => Self::E,
            4 => Self::H,
            5 => Self::L,
            6 => Self::HLMem,
            7 => Self::A,

            _ => panic!("Invalid ConditionCode index!")
        }
    }
}

/*
    ================
     Arithmetic Ops
    ================
*/

#[derive(Copy, Clone)]
pub(super) enum ArithmeticOp {
    ADD = 0,
    ADC = 1,
    SUB = 2,
    SBC = 3,
    AND = 4,
    XOR = 5,
    OR  = 6,
    CP  = 7
}

impl Display for ArithmeticOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ADD => write!(f, "ADD A"),
            Self::ADC => write!(f, "ADC A"),
            Self::SUB => write!(f, "SUB A"),
            Self::SBC => write!(f, "SBC A"),
            Self::AND => write!(f, "AND A"),
            Self::XOR => write!(f, "XOR A"),
            Self::OR =>  write!(f, "OR A"),
            Self::CP =>  write!(f, "CP A"),
        }
    }
}

impl ArithmeticOp {
    pub(super) fn from_u8(index: u8) -> Self {
        match index {
            0 => Self::ADD,
            1 => Self::ADC,
            2 => Self::SUB,
            3 => Self::SBC,
            4 => Self::AND,
            5 => Self::XOR,
            6 => Self::OR,
            7 => Self::CP,

            _ => panic!("Invalid ArithmeticOp index!")
        }
    }
}

/*
    ====================
     Rotation/Shift ops
    ====================
*/

#[derive(Copy, Clone)]
pub(super) enum CBOp {
    RLC = 0,
    RRC = 1,
    RL = 2,
    RR = 3,
    SLA = 4,
    SRA = 5,
    SWAP  = 6,
    SRL  = 7
}

impl CBOp {
    pub(super) fn from_u8(index: u8) -> Self {
        match index {
            0 => Self::RLC,
            1 => Self::RRC,
            2 => Self::RL,
            3 => Self::RR,
            4 => Self::SLA,
            5 => Self::SRA,
            6 => Self::SWAP,
            7 => Self::SRL,

            _ => panic!("Invalid ArithmeticOp index!")
        }
    }
}

impl Display for CBOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RLC => write!(f, "RLC"),
            Self::RRC => write!(f, "RRC"),
            Self::RL => write!(f, "RL"),
            Self::RR => write!(f, "RR"),
            Self::SLA => write!(f, "SLA"),
            Self::SRA => write!(f, "SRA"),
            Self::SWAP => write!(f, "SWAP"),
            Self::SRL => write!(f, "SRL"),
        }
    }
}

