
use isa::{OperandType, RawAddress, RawOperand, RawPort};

use crate::command_metadata::SourceCommandMetadata;

use super::Label;

#[derive(Clone)]
pub struct SourceCodeCommand {
    pub metadata: &'static SourceCommandMetadata,
    pub argument: Argument,
}


#[derive(Clone)]
pub enum Argument {
    None,
    Port(RawPort),
    Immediate(RawOperand),
    Address(AddressWithMode),
}

#[derive(Clone, Copy)]
pub enum AddressingMode {
    Absolute, // !number, !label
    Relative, // number, label
    // this is to deref pointers
    Indirect, // (number), (label)
}

impl From<AddressingMode> for OperandType {
    fn from(value: AddressingMode) -> Self {
        use isa::OperandType::*;
        match value {
            AddressingMode::Absolute => Absolute,
            AddressingMode::Relative => Relative,
            AddressingMode::Indirect => Indirect,
        }
    }
}

#[derive(Clone)]
pub enum Reference {
    RawAddress(RawAddress),
    Label(Label),
}

#[derive(Clone)]
pub struct AddressWithMode {
    pub mode: AddressingMode,
    pub address: Reference,
}

