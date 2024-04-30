use isa::{OperandType, RawAddress};

use super::{parse_number, Label, ParsingError, WORD_REGEX};

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
    mode: AddressingMode,
    address: Reference,
}

impl AddressWithMode {
    fn parse_mode(input: &str) -> Result<(AddressingMode, &str), ParsingError> {
        if input.starts_with("!") {
            return Ok((AddressingMode::Absolute, &input[1..]));
        }

        let start_parentheses = input.starts_with("(");
        let ends_parentheses = input.ends_with(")");

        if start_parentheses != ends_parentheses {
            return Err(ParsingError::SyntaxError(
                r#"Because single parentheses found present assumes Relative mode.
                No matching parentheses was found!"#
                    .to_string(),
            ));
        }

        if start_parentheses && ends_parentheses {
            return Ok((AddressingMode::Indirect, &input[1..input.len() - 2]));
        }

        return Ok((AddressingMode::Relative, input));
    }

    fn parse_address(address: &str) -> Result<Reference, ParsingError> {
        let address = address.trim();
        if address.starts_with(|value: char| ('0'..'9').contains(&value)) {
            // probably number
            let address = parse_number(address)?;

            return Ok(Reference::RawAddress(address));
        };

        let Some(label) = WORD_REGEX.find(address) else {
            return Err(ParsingError::CouldNotParseArgument);
        };

        Ok(Reference::Label(label.as_str().to_owned()))
    }
}

impl TryFrom<&str> for AddressWithMode {
    type Error = ParsingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (mode, address) = AddressWithMode::parse_mode(value)?;
        let address = AddressWithMode::parse_address(address)?;

        Ok(Self { mode, address })
    }
}