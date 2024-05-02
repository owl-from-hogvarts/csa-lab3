use isa::{OperandType, RawAddress};

use super::{token::TokenStream, Label, ParsingError};

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

impl AddressWithMode {
    fn parse_mode(stream: &mut TokenStream) -> Result<AddressingMode, ParsingError> {
        if stream.next_special_symbol('!').is_ok() {
            return Ok(AddressingMode::Absolute);
        }

        let start_parentheses = stream.next_special_symbol('(').is_ok();
        let ends_parentheses = stream.peek(2)?.is_special_symbol(')');

        if start_parentheses != ends_parentheses {
            return Err(ParsingError::Other(
                r#"Because single parentheses found present assumes Relative mode.
                No matching parentheses was found!"#
                    .to_string(),
            ));
        }

        if start_parentheses && ends_parentheses {
            return Ok(AddressingMode::Indirect);
        }

        return Ok(AddressingMode::Relative);
    }

    fn parse_address(stream: &mut TokenStream) -> Result<Reference, ParsingError> {
        if let Ok(address) = stream.next_number() {
            return Ok(Reference::RawAddress(address));
        };

        let label = stream
            .next_word()
            .map_err(|_| ParsingError::CouldNotParseArgument)?;

        Ok(Reference::Label(label.clone()))
    }
}

impl TryFrom<&mut TokenStream> for AddressWithMode {
    type Error = ParsingError;

    fn try_from(stream: &mut TokenStream) -> Result<Self, Self::Error> {
        let mode = AddressWithMode::parse_mode(stream)?;
        let address = AddressWithMode::parse_address(stream)?;

        Ok(Self { mode, address })
    }
}
