use serde_json::to_writer;
use std::{
    env,
    fmt::Display,
    fs::{self, OpenOptions},
    path::PathBuf,
};

use parser::parse_asm;

mod parser;

fn main() {
    let config = match parse_cli_args() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {err}");
            return;
        }
    };

    let input_srting = match fs::read_to_string(config.input_file) {
        Ok(input) => input,
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

    let parsed_program = match parse_asm(input_srting) {
        Ok(program) => program,
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

    let compiled = match parsed_program.compile() {
        Ok(compiled_program) => compiled_program,
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };

    let output_file = match OpenOptions::new()
        .create(true)
        .write(true)
        .open(config.output_file)
    {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    };
    match to_writer(output_file, &compiled) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
    }
}

struct Config {
    input_file: PathBuf,
    output_file: PathBuf,
}

/// accepts two positional args:
/// input output
fn parse_cli_args() -> Result<Config, ConfigurationError> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    args.reverse();
    for (index, arg) in args.iter().enumerate() {
        if arg.len() < 1 {
            return Err(ConfigurationError::EmptyArgument(index));
        }
    }

    let input_file: PathBuf = args
        .pop()
        .ok_or(ConfigurationError::ArgumentNotFound {
            argument_name: "input".into(),
        })?
        .into();

    let input_file_name = input_file
        .file_stem()
        .ok_or(ConfigurationError::NotAFile(input_file.clone()))?
        .to_str()
        .ok_or(ConfigurationError::InvalidUnicode)?
        .to_owned();

    let output_file = args.pop().unwrap_or(input_file_name + ".json").into();

    Ok(Config {
        input_file,
        output_file,
    })
}

enum ConfigurationError {
    InvalidUnicode,
    NotAFile(PathBuf),
    ArgumentNotFound { argument_name: String },
    EmptyArgument(usize),
}

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
