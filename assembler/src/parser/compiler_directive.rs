use crate::source_code::CompilerDirective;

use super::{token::{Token, TokenStream}, ParsingError};

impl CompilerDirective {
    pub fn from_token_stream(stream: &mut TokenStream) -> Result<Option<Self>, ParsingError> {
        if let Token::Word(command) = stream.peek(1)? {
            match command.to_uppercase().as_str() {
                "WORD" => {
                    // advance stream on match only
                    stream.next_word()?;
                    let mut data = Vec::new();

                    if let Ok(label) = stream.next_word() {
                        return Ok(Some(Self::Pointer(label)));
                    }

                    while let Ok(number) = stream.next_long_number() {
                        data.push(number as u32);
                    }

                    // ensure that no unparsed input left
                    stream.next_end_of_input()?;

                    return Ok(Some(Self::Data(data)));
                }
                "ORG" => {
                    stream.next_word()?;
                    let address = stream.next_number()?;

                    return Ok(Some(Self::SetAddress(address)));
                }
                _ => return Ok(None),
            };
        }

        Ok(None)
    }
}
