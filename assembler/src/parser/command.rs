use crate::command_metadata::SourceCommandMetadata;
use crate::parser::ParsingError;
use crate::source_code::command::SourceCodeCommand;

use super::token::TokenStream;

impl SourceCodeCommand {
    pub fn from_token_stream(stream: &mut TokenStream) -> Result<Self, ParsingError> {
        let opcode = stream.next_word()?;

        let metadata = SourceCommandMetadata::get_metadata_by_opcode(opcode.as_str())?;
        let argument = (metadata.argument_type)(stream)?;

        Ok(SourceCodeCommand { metadata, argument })
    }
}
