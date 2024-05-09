
//! Utility module to match mnemonic to opcode and command's argument type

use isa::Opcode;

use crate::parser::errors::ParsingError;
use crate::parser::token::TokenStream;
use crate::source_code::command::Argument;

pub struct SourceCommandMetadata {
    pub opcode: Opcode,
    pub argument_type: fn(&mut TokenStream) -> Result<Argument, ParsingError>,
}

impl SourceCommandMetadata {
    pub fn get_metadata_by_opcode(
        opcode: &str,
    ) -> Result<&'static SourceCommandMetadata, ParsingError> {
        match opcode.to_uppercase().as_str() {
            "IN" => Ok(&SourceCommandMetadata {
                opcode: Opcode::IN,
                argument_type: Argument::parse_port,
            }),
            "OUT" => Ok(&SourceCommandMetadata {
                opcode: Opcode::OUT,
                argument_type: Argument::parse_port,
            }),
            "LOAD" => Ok(&SourceCommandMetadata {
                opcode: Opcode::LOAD,
                argument_type: Argument::parse_address,
            }),
            "STORE" => Ok(&SourceCommandMetadata {
                opcode: Opcode::STORE,
                argument_type: Argument::parse_address,
            }),
            "ADD" => Ok(&SourceCommandMetadata {
                opcode: Opcode::ADD,
                argument_type: Argument::parse_address,
            }),
            "INC" => Ok(&SourceCommandMetadata {
                opcode: Opcode::INC,
                argument_type: Argument::parse_none,
            }),
            "AND" => Ok(&SourceCommandMetadata {
                opcode: Opcode::AND,
                argument_type: Argument::parse_address,
            }),
            "ANDI" => Ok(&SourceCommandMetadata {
                opcode: Opcode::AND,
                argument_type: Argument::parse_immediate,
            }),
            "CMP" => Ok(&SourceCommandMetadata {
                opcode: Opcode::CMP,
                argument_type: Argument::parse_address,
            }),
            "SHIFT_LEFT" => Ok(&SourceCommandMetadata {
                opcode: Opcode::SHIFT_LEFT,
                argument_type: Argument::parse_none,
            }),
            "SHIFT_RIGHT" => Ok(&SourceCommandMetadata {
                opcode: Opcode::SHIFT_RIGHT,
                argument_type: Argument::parse_none,
            }),
            "JZC" => Ok(&SourceCommandMetadata {
                opcode: Opcode::JZC,
                argument_type: Argument::parse_address,
            }),
            "JZS" => Ok(&SourceCommandMetadata {
                opcode: Opcode::JZS,
                argument_type: Argument::parse_address,
            }),
            "JZ" => Ok(&SourceCommandMetadata {
                opcode: Opcode::JZS,
                argument_type: Argument::parse_address,
            }),
            "JCC" => Ok(&SourceCommandMetadata {
                opcode: Opcode::JCC,
                argument_type: Argument::parse_address,
            }),
            "JCS" => Ok(&SourceCommandMetadata {
                opcode: Opcode::JCS,
                argument_type: Argument::parse_address,
            }),
            "JC" => Ok(&SourceCommandMetadata {
                opcode: Opcode::JCS,
                argument_type: Argument::parse_address,
            }),
            "JUMP" => Ok(&SourceCommandMetadata {
                opcode: Opcode::JUMP,
                argument_type: Argument::parse_address,
            }),
            "NOP" => Ok(&SourceCommandMetadata {
                opcode: Opcode::NOP,
                argument_type: Argument::parse_none,
            }),
            "HALT" => Ok(&SourceCommandMetadata {
                opcode: Opcode::HALT,
                argument_type: Argument::parse_none,
            }),
            _ => Err(ParsingError::UnknownCommand(opcode.to_owned())),
        }
    }
}
