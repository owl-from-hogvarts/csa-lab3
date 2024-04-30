use std::{error::Error, fmt::Display, num::ParseIntError, ops::Range};

use super::Label;

#[derive(Debug)]
pub enum ParsingError {
    UnknownCommand(String),
    NoArgumentProvided,
    SyntaxError(String),
    CouldNotParseArgument,
    MultipleDefinitions(Label),
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::UnknownCommand(command) => writeln!(f, "Unknown Command: {command}"),
            ParsingError::NoArgumentProvided => {
                writeln!(f, "Argument expected! No argument provided!")
            }
            ParsingError::SyntaxError(err) => writeln!(f, "Syntax error: {}", err),
            ParsingError::CouldNotParseArgument => writeln!(f, "Could not parse argument!"),
            ParsingError::MultipleDefinitions(label) => {
                writeln!(f, "Label {label} defined multiple times!")
            }
        }
    }
}

impl From<ParseIntError> for ParsingError {
    fn from(value: ParseIntError) -> Self {
        Self::SyntaxError(value.to_string())
    }
}

#[derive(Debug)]
pub enum CompilationError {
    LabelDoesNotExists {
        label: Label,
    },
    SectionTooLarge {
        address_span: Range<u16>,
        actual_size: usize,
    },
}

impl Error for CompilationError {}

impl Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationError::LabelDoesNotExists { label } => {
                writeln!(f, "Label {label} does not exist!")
            }
            CompilationError::SectionTooLarge {
                actual_size,
                address_span,
            } => writeln!(
                f,
                r"Section contains {actual_size} items!
                They do not fit into address span from {} (inclusive) to {} (exclusive)!
                Max size for the section is {}",
                address_span.start,
                address_span.end,
                address_span.end - address_span.start,
            ),
        }
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
