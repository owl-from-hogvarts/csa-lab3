use cli_utils::{check_empty_arguments, ConfigurationError};
use std::{
    env,
    error::Error,
    fs::{self, OpenOptions},
    path::PathBuf,
};

use parser::parse_asm;

mod parser;
mod compiler;

fn main() {
    match start() {
        Ok(_) => (),
        Err(err) => eprintln!("Error: {err}"),
    }
}

fn start() -> Result<(), Box<dyn Error>> {
    let config = parse_cli_args()?;

    let input_srting = fs::read_to_string(config.input_file)?;

    let parsed_program = parse_asm(input_srting)?;

    let compiled = parsed_program.compile()?;

    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(config.output_file)?;

    Ok(serde_json::to_writer(output_file, &compiled)?)
}

struct Config {
    input_file: PathBuf,
    output_file: PathBuf,
}

/// accepts two positional args:
/// input output
fn parse_cli_args() -> Result<Config, ConfigurationError> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    check_empty_arguments(&args)?;
    args.reverse();

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
