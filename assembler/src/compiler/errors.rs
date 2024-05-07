use std::{error::Error, fmt::Display};

use crate::source_code::Label;

#[derive(Debug)]
pub enum CompilationError {
    LabelDoesNotExists { label: Label },
}

impl Error for CompilationError {}

impl Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationError::LabelDoesNotExists { label } => {
                writeln!(f, "Label {label} does not exist!")
            }
        }
    }
}
