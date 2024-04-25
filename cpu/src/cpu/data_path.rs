use isa::{CompiledCommand, MemoryItem};

use super::TRegisterValue;

pub struct Registers {
    pub accumulator: TRegisterValue,
    pub data: MemoryItem,
    // command is special register
    // it holds high level data structure
    pub command: CompiledCommand,
    pub program_counter: TRegisterValue,
    pub address: TRegisterValue,
}

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
pub struct ALU_Config {
    pub left: TRegisterValue,
    pub right: TRegisterValue,
    pub AND: bool,
    pub NOT_LEFT: bool,
    pub NOT_RIGHT: bool,
    pub INC: bool,
}

#[allow(non_camel_case_types)]
pub struct ALU_Output {
    pub zero: bool,
    pub carry: bool,
    pub value: TRegisterValue,
}

#[allow(non_snake_case)]
pub fn ALU(
    ALU_Config {
        mut left,
        mut right,
        AND,
        NOT_LEFT,
        NOT_RIGHT,
        INC,
    }: ALU_Config,
) -> ALU_Output {
    if NOT_LEFT {
        left = !left;
    }

    if NOT_RIGHT {
        right = !right;
    }

    let (mut value, carry) = if AND {
        (left & right, false)
    } else {
        left.overflowing_add(right)
    };

    if INC {
        value += 1;
    }

    let zero = value == 0;

    ALU_Output { zero, carry, value }
}
