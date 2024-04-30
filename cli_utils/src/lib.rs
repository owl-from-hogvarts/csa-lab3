use std::{error::Error, fmt::Display, path::PathBuf};

pub fn check_empty_arguments(args: &Vec<String>) -> Result<(), ConfigurationError> {
    for (index, arg) in args.iter().enumerate() {
        if arg.len() < 1 {
            return Err(ConfigurationError::EmptyArgument(index));
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum ConfigurationError {
    InvalidUnicode,
    NotAFile(PathBuf),
    ArgumentNotFound { argument_name: String },
    EmptyArgument(usize),
}

impl Error for ConfigurationError {}

impl Display for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigurationError::ArgumentNotFound { argument_name } => {
                writeln!(f, "Argument expected but not provided: {argument_name}")
            }
            ConfigurationError::EmptyArgument(index) => {
                writeln!(f, "Argument at position {} is empty!", index + 1)
            }
            ConfigurationError::InvalidUnicode => {
                writeln!(f, "Only Unicode arguments are supported!")
            }
            ConfigurationError::NotAFile(path) => {
                writeln!(
                    f,
                    "{} seems to be NOT a file! Expected text file",
                    path.to_string_lossy()
                )
            }
        }
    }
}
