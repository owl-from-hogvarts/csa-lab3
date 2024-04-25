use core::panic;

use serde::Serialize;

pub const START_ADDRESS: RawOperand = 0x10;

pub type RawOperand = u16;
pub type RawAddress = RawOperand;
pub type RawPort = u8;

#[derive(Serialize)]
pub struct Operand {
    pub operand: RawOperand,
    pub operand_type: OperandType,
}

// address
// label, (label), !label, same with number

#[derive(Clone, Copy, Serialize)]
pub enum Opcode {
    IN,  // port
    OUT, // port

    LOAD,  // address
    STORE, // address

    ADD, // address
    INC, // none
    AND, // address
    CMP, // address

    JZC,  // address
    JZS,  // address
    JCS,  // address
    JCC,  // address
    JUMP, // address

    NOP,  // none
    HALT, // none
}

#[derive(Serialize)]
pub enum OperandType {
    None,
    Indirect,
    Absolute,
    Relative,
    Immediate,
}

#[derive(Serialize)]
pub struct CompiledCommand {
    pub opcode: Opcode,
    #[serde(flatten)]
    pub operand: Operand,
}

#[derive(Serialize)]
pub struct CompiledSection {
    pub start_address: RawAddress,
    pub items: Vec<MemoryItem>,
}

#[derive(Serialize)]
pub struct CompiledProgram {
    pub sections: Vec<CompiledSection>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum MemoryItem {
    Data(u32),
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
