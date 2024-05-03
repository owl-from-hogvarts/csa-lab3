use core::panic;
use std::mem::size_of;

use serde::{Deserialize, Serialize};

pub const START_ADDRESS: RawOperand = 0x10;
// index to length conversion
pub const MEMORY_SIZE: usize = RawAddress::MAX as usize + 1;
pub const MEMORY_DATA_CELL_SIZE: usize = size_of::<MemoryDataType>();

pub type MemoryDataType = u32;
pub type RawOperand = u16;
pub type RawAddress = RawOperand;
pub type RawPort = u8;

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub struct Operand {
    pub operand: RawOperand,
    pub operand_type: OperandType,
}

// address
// label, (label), !label, same with number

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Opcode {
    IN,  // port
    OUT, // port

    LOAD,  // address
    STORE, // address

    ADD,         // address
    INC,         // none
    AND,         // address
    CMP,         // address
    SHIFT_LEFT,  // immediate
    SHIFT_RIGHT, // immediate

    JZC,  // address
    JZS,  // address
    JCS,  // address
    JCC,  // address
    JUMP, // address

    NOP,  // none
    HALT, // none
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum OperandType {
    None,
    Indirect,
    Absolute,
    Relative,
    Immediate,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct CompiledCommand {
    pub opcode: Opcode,
    #[serde(flatten)]
    pub operand: Operand,
}

#[derive(Serialize, Deserialize)]
pub struct CompiledSection {
    pub start_address: RawAddress,
    pub items: Vec<MemoryItem>,
}

impl CompiledSection {
    pub fn with_address(address: RawAddress) -> Self {
        Self {
            start_address: address,
            items: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CompiledProgram {
    pub sections: Vec<CompiledSection>,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum MemoryItem {
    Data(MemoryDataType),
    Command(CompiledCommand),
}

impl MemoryItem {
    pub fn unwrap_data(self) -> u32 {
        match self {
            MemoryItem::Data(payload) => payload,
            MemoryItem::Command(_) => {
                panic!("Instruction accessed! Instructions does not have binary representation!")
            }
        }
    }

    pub fn unwrap_command(self) -> CompiledCommand {
        match self {
            MemoryItem::Data(_) => panic!("Data accessed! Instruction expected!"),
            MemoryItem::Command(command) => command,
        }
    }
}

impl From<MemoryItem> for u32 {
    fn from(value: MemoryItem) -> Self {
        value.unwrap_data()
    }
}
