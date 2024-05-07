// argument notion is related to source code
// while operand is all about compiled representation


use num::ToPrimitive;

use crate::source_code::command::AddressWithMode;
use crate::source_code::command::Argument;

use super::ParsingError;
use super::token::TokenStream;

impl Argument {
    pub fn parse_none(stream: &mut TokenStream) -> Result<Argument, ParsingError> {
        stream.next_end_of_input()?;
        Ok(Argument::None)
    }

    pub fn parse_port(stream: &mut TokenStream) -> Result<Argument, ParsingError> {
        let port_number = stream.next_number()?;

        Ok(Argument::Port(
            port_number
                .to_u8()
                .ok_or(ParsingError::CouldNotParseArgument)?,
        ))
    }

    pub fn parse_immediate(stream: &mut TokenStream) -> Result<Argument, ParsingError> {
        Ok(Argument::Immediate(stream.next_number()?))
    }

    pub fn parse_address(stream: &mut TokenStream) -> Result<Argument, ParsingError> {
        let address: AddressWithMode = stream.try_into()?;

        Ok(Argument::Address(address))
    }

}
