use std::{error::Error, fmt::Display};

use super::{token::TokenStreamError, Label};

#[derive(Debug)]
pub enum ParsingError {
    UnknownCommand(String),
    TokenError(TokenStreamError),
    Other(String),
    CouldNotParseArgument,
    MultipleDefinitions(Label),
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::UnknownCommand(command) => writeln!(f, "Unknown Command: {command}"),
            ParsingError::Other(err) => writeln!(f, "Syntax error: {}", err),
            ParsingError::CouldNotParseArgument => writeln!(f, "Could not parse argument!"),
            ParsingError::MultipleDefinitions(label) => {
                writeln!(f, "Label {label} defined multiple times!")
            }
            ParsingError::TokenError(token) => writeln!(f, "{token}"),
        }
    }
}

impl From<TokenStreamError> for ParsingError {
    fn from(value: TokenStreamError) -> Self {
        Self::TokenError(value)
    }
}

#[derive(Debug)]
pub struct ParsingErrorOnLine {
    pub error: ParsingError,
    pub line_number: usize,
}

impl Error for ParsingErrorOnLine {}

impl Display for ParsingErrorOnLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Parsing Error occured at line: {}", self.line_number)?;
        writeln!(
            f,
            "{}",
            self.error
                .to_string()
                .lines()
                .map(|line| {
                    let mut line = line.to_owned();
                    line.insert_str(0, "  ");
                    line.push('\n');
                    line
                })
                .collect::<String>()
        )
    }
}
