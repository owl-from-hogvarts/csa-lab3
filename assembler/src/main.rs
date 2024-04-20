use std::{env, fmt::Display, fs, path::PathBuf};

mod commands;

fn main() {
    let config = match parse_cli_args() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };
    // read file
    let input_srting = fs::read_to_string(config.input_file);
    // split by line
    // remove empty lines
    // match every line

    // argument types:
    // number: 10, 0xf, 0b111
    // (number)
    // label, (label)
    // string (acceptable only for "word" special command)

    // output is json
    println!("Hello, world!");
}

struct Config {
    input_file: PathBuf,
    output_file: PathBuf,
}

const ARGS_COUNT: usize = 2;

/// accepts two positional args:
/// input output
fn parse_cli_args() -> Result<Config, ConfigurationError> {
    let mut args: Vec<String> = env::args().skip(1).collect();

    for (index, arg) in args.iter().enumerate() {
        if arg.len() < 1 {
            return Err(ConfigurationError::EmptyArgument(index));
        }
    }

    let output_file = args
        .pop()
        .ok_or(ConfigurationError::ArgumentNotFound {
            argument_name: "input".into(),
        })?
        .into();
    let input_file = args
        .pop()
        .ok_or(ConfigurationError::ArgumentNotFound {
            argument_name: "output".into(),
        })?
        .into();

    Ok(Config {
        input_file,
        output_file,
    })
}

enum ConfigurationError {
    ArgumentNotFound { argument_name: String },
    EmptyArgument(usize),
}

impl Display for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigurationError::ArgumentNotFound { argument_name } => {
                write!(f, "Argument expected but not provided: {argument_name}")
            }
            ConfigurationError::EmptyArgument(index) => {
                write!(f, "Argument at position {} is empty!", index + 1)
            }
        }
    }
}
