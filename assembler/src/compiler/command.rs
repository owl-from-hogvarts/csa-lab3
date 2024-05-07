use isa::{CompiledCommand, Operand, OperandType, RawAddress};

use crate::source_code::command::{AddressingMode, Argument, Reference, SourceCodeCommand};

use super::{errors::CompilationError, ResolvedLabels};

impl SourceCodeCommand {
    pub fn compile(
        &self,
        labels: &ResolvedLabels,
        current_address: RawAddress,
    ) -> Result<CompiledCommand, CompilationError> {
        let Self { metadata, .. } = self;

        Ok(CompiledCommand {
            opcode: metadata.opcode,
            operand: self.argument.to_operand(labels, current_address)?,
        })
    }
}

impl Argument {
    fn to_operand(
        &self,
        labels: &ResolvedLabels,
        current_address: RawAddress,
    ) -> Result<Operand, CompilationError> {
        use isa::OperandType::*;
        Ok(match self {
            Argument::None => Operand {
                operand: 0,
                operand_type: None,
            },
            &Argument::Port(port) => Operand {
                operand: port as u16,
                operand_type: Immediate,
            },
            &Argument::Immediate(value) => Operand {
                operand: value,
                operand_type: Immediate,
            },
            Argument::Address(address) => {
                let actual_address =
                    match &address.address {
                        &Reference::RawAddress(address) => address,
                        Reference::Label(label) => labels.get(label).copied().ok_or(
                            CompilationError::LabelDoesNotExists {
                                label: label.clone(),
                            },
                        )?,
                    };
                let operand = match address.mode {
                    AddressingMode::Absolute => actual_address,
                    AddressingMode::Relative | AddressingMode::Indirect => {
                        // relative addressing mode is relative to Program Counter.
                        // Program counter always points to the next command
                        actual_address.overflowing_sub(current_address + 1).0
                    }
                };

                let operand_type: OperandType = address.mode.into();

                Operand {
                    operand,
                    operand_type,
                }
            }
        })
    }
}
