use isa::{CompiledCommand, MemoryItem, Opcode::NOP, Operand, OperandType::None, RawAddress};

use super::TRegisterValue;

#[derive(Debug)]
pub struct Registers {
    pub accumulator: TRegisterValue,
    pub data: MemoryItem,
    // command is special register
    // it holds high level data structure
    pub command: CompiledCommand,
    pub program_counter: RawAddress,
    pub address: RawAddress,
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            accumulator: 0,
            data: MemoryItem::Data(0),
            command: CompiledCommand {
                opcode: NOP,
                operand: Operand {
                    operand: 0,
                    operand_type: None,
                },
            },
            program_counter: 0,
            address: 0,
        }
    }
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
    pub SHIFT: bool,
    pub SHIFT_LEFT: bool,
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
        SHIFT, 
        SHIFT_LEFT,
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
        left.overflowing_add(right + INC as u32)
    };

    if SHIFT {
        if SHIFT_LEFT {
            value <<= 1;
        } else {
            value >>= 1;
        }
    }

    let zero = value == 0;

    ALU_Output { zero, carry, value }
}
