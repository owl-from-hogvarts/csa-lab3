use std::{borrow::Borrow, collections::HashMap};

use isa::{Opcode, Operand, OperandType, RawOperand, RawPort};
use regex::Regex;

type Label = String;
type Index = usize;
type Labels = HashMap<Label, Index>;
type RawAddress = u16;
type ResolvedIndex = HashMap<Index, RawAddress>;
type ResolvedLabels = HashMap<Label, RawAddress>;

// argument notions is related to source code
// while operand is all about compiled representation

// parse: SourceCodeCommands(argument_str), label -> indexes
// resolve labels: label -> index -> address
// each command compiles interprets it's argument on it's own

#[derive(Clone, Copy)]
enum AddressingMode {
    Absolute, // !number, !label
    Relative, // number, label
    Indirect, // (number), (label)
}

impl From<AddressingMode> for OperandType {
    fn from(value: AddressingMode) -> Self {
        use isa::OperandType::*;
        match value {
            AddressingMode::Absolute => Absolute,
            AddressingMode::Relative => Relative,
            AddressingMode::Indirect => Indirect,
        }
    }
}

enum AddressingModeParseError {}

impl TryFrom<&str> for AddressingMode {
    type Error = AddressingModeParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // absolute
        if value.starts_with("!") {
            return Ok(Self::Absolute);
        }

        // indirect
        if value.starts_with("(") && value.ends_with(")") {
            return Ok(Self::Indirect);
        }

        // relative
        return Ok(Self::Relative);
    }
}

enum ActualAddress {
    RawAddress(RawAddress),
    Label(Label),
}

enum ActualAddressError {}

impl TryFrom<&str> for ActualAddress {
    type Error = ActualAddressError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut value = value;
        // crutch, don't have enought time
        // for real parsing
        if value.starts_with("!") {
            value = &value[1..];
        } else if value.starts_with("(") {
            value = &value[1..value.len() - 2];
        }

        if let Ok(value) = value.parse::<RawAddress>() {
            return Ok(Self::RawAddress(value));
        }

        Ok(Self::Label(value.to_string()))
    }
}

struct Address {
    mode: AddressingMode,
    address: ActualAddress,
}

enum AddressError {
    AddressingModeError(AddressingModeParseError),
    ActualAddressError(ActualAddressError),
}

impl From<AddressingModeParseError> for AddressError {
    fn from(value: AddressingModeParseError) -> Self {
        Self::AddressingModeError(value)
    }
}

impl From<ActualAddressError> for AddressError {
    fn from(value: ActualAddressError) -> Self {
        Self::ActualAddressError(value)
    }
}

impl TryFrom<&str> for Address {
    type Error = AddressError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mode: AddressingMode = value.try_into()?;
        let address: ActualAddress = value.try_into()?;

        Ok(Self { mode, address })
    }
}

#[derive(PartialEq)]
enum ArgumentType {
    None,
    Port,
    Immidiate,
    Address,
}

impl From<&Argument> for ArgumentType {
    fn from(value: &Argument) -> Self {
        match value {
            Argument::None => ArgumentType::None,
            Argument::Port(_) => ArgumentType::Port,
            Argument::Immidiate(_) => ArgumentType::Immidiate,
            Argument::Address(_) => ArgumentType::Address,
        }
    }
}

enum Argument {
    None,
    Port(RawPort),
    Immidiate(RawOperand),
    Address(Address),
}

impl Argument {
    fn to_operand(&self, program: &Program) -> Result<Operand, ResolutionError> {
        use isa::OperandType::*;
        Ok(match self {
            Argument::None => Operand {
                operand: 0,
                operand_type: None,
            },
            &Argument::Port(port) => Operand {
                operand: port as u16,
                operand_type: Immidiate,
            },
            &Argument::Immidiate(value) => Operand {
                operand: value,
                operand_type: Immidiate,
            },
            Argument::Address(address) => {
                let operand = match &address.address {
                    &ActualAddress::RawAddress(address) => address,
                    ActualAddress::Label(label) => program.resolve(label)?,
                };
                let operand_type: OperandType = address.mode.into();

                Operand {
                    operand,
                    operand_type,
                }
            }
        })
    }
}

struct SourceCodeCommand {
    opcode: String,
    argument: Argument,
}

struct Program {
    labels: ResolvedLabels,
}

enum ResolutionError {
    LabelDoesNotExists { label: Label },
}

impl Program {
    fn resolve(&self, label: &str) -> Result<u16, ResolutionError> {
        self.labels
            .get(label)
            .copied()
            .ok_or(ResolutionError::LabelDoesNotExists {
                label: label.to_string(),
            })
    }
}

struct SourceCommandMetadata {
    opcode: Opcode,
    argument_type: ArgumentType,
}

impl SourceCodeCommand {
    fn compile(&self, program: &Program) -> Result<CompiledCommand, ResolutionError> {
        let metadata = match self.opcode.as_ref() {
            "AND" => SourceCommandMetadata {
                opcode: Opcode::AND,
                argument_type: ArgumentType::Address,
            },
            "ANDI" => SourceCommandMetadata {
                opcode: Opcode::AND,
                argument_type: ArgumentType::Immidiate,
            },
        };

        if metadata.argument_type != (&self.argument).into() {
            // error here
        }

        Ok(CompiledCommand {
            opcode: metadata.opcode,
            operand: self.argument.to_operand(program)?,
        })
    }
}

struct CompiledCommand {
    opcode: Opcode,
    operand: Operand,
}

fn parse_asm(input: &str) {
    let lines = input.lines();
    // remove comments
    let lines = lines
        .map(|line| {
            if let Some(comment_start) = line.find("//") {
                return &line[..comment_start];
            }

            line
        })
        .map(|line| line.trim());

    let mut labels: Labels = Labels::new();
    let commands: Vec<SourceCodeCommand> = Vec::new();

    let label_regex = Regex::new(r"^\w+:").unwrap();
    let command_regex = Regex::new(r"\w^").unwrap();
    for line in lines {
        let mut start = 0;
        if let Some(label) = label_regex.find(line) {
            labels.insert(label.as_str().to_owned(), commands.len());
            start = label.start();
        }

        let Some(command) = command_regex.find_at(&line[start..], start) else {
            continue;
        };
        start = command.end();
    }
}
