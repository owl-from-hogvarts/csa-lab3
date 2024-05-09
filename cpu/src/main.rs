use std::{
    env,
    error::Error,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

use cli_utils::{check_empty_arguments, ConfigurationError};
use io_controller::{IOController, SimpleInputOutput};
use isa::CompiledProgram;
use memory::Memory;

use crate::cpu::Cpu;

mod cpu;
mod io_controller;
mod memory;

fn main() {
    match start() {
        Ok(_) => (),
        Err(err) => eprintln!("Error: {err}"),
    }
}

fn start() -> Result<(), Box<dyn Error>> {
    let log_path = Path::new("cpu.log");
    let log_output = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(log_path)?;
    let log_config = simplelog::ConfigBuilder::new()
        .set_time_level(log::LevelFilter::Debug)
        .build();

    simplelog::WriteLogger::init(log::LevelFilter::Trace, log_config, log_output)?;

    let delimiter = "=".repeat(80);

    log::info!("{}", delimiter);
    log::info!("{}", delimiter);
    log::info!("{}", delimiter);

    let config = parse_cli_args()?;
    let program: CompiledProgram = serde_json::from_reader(File::open(config.program_path)?)?;
    let output = fs::read_to_string(config.io_device_input_path)?;

    let memory = Memory::burn(program);
    let io_controller = IOController::new().connect(0, Box::new(SimpleInputOutput::new(output)));

    let cpu = Cpu::new(memory, io_controller);
    cpu.start();

    Ok(())
}

struct Config {
    program_path: PathBuf,
    io_device_input_path: PathBuf,
}

// custom parsing, because parsing of file paths is required
// custom error handling logic is easier to implement in that way
fn parse_cli_args() -> Result<Config, ConfigurationError> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    check_empty_arguments(&args)?;

    args.reverse();

    let program_path = args
        .pop()
        .ok_or(ConfigurationError::ArgumentNotFound {
            argument_name: "program path".into(),
        })?
        .into();

    let io_device_input_path = args
        .pop()
        .ok_or(ConfigurationError::ArgumentNotFound {
            argument_name: "io device input".into(),
        })?
        .into();

    Ok(Config {
        program_path,
        io_device_input_path,
    })
}
