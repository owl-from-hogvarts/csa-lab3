use std::collections::HashMap;

use isa::{Opcode, Operand, OperandType, RawOperand, RawPort};
use once_cell::sync::Lazy;
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

static WORD_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\w+").unwrap());
static LABEL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(format!(r"^{}:", *WORD_REGEX).as_str()).unwrap());

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

enum ActualAddress {
    RawAddress(RawAddress),
    Label(Label),
}

struct Address {
    mode: AddressingMode,
    address: ActualAddress,
}

impl Address {
    fn parse_mode(input: &str) -> Result<(AddressingMode, &str), AddressError> {
        if input.starts_with("!") {
            return Ok((AddressingMode::Absolute, &input[1..]));
        }

        let start_parenthese = input.starts_with("(");
        let ends_parenthese = input.ends_with(")");

        if start_parenthese != ends_parenthese {
            return Err(AddressError::SyntaxError("Because single paranthese present assumes Relative mode\nNo matching parenthese was found!"));
        }

        if start_parenthese && ends_parenthese {
            return Ok((AddressingMode::Indirect, &input[1..input.len() - 2]));
        }

        return Ok((AddressingMode::Relative, input));
    }

    fn parse_address(address: &str) -> Result<ActualAddress, AddressError> {
        let address = address.trim();
        if address.starts_with(|value: char| ('0'..'9').contains(&value)) {
            // probaly number
            let (prefix, value) = address.split_at(2);
            let address = match prefix {
                "0x" => RawAddress::from_str_radix(value, 16),
                "0b" => RawAddress::from_str_radix(value, 2),
                _ => RawAddress::from_str_radix(address, 10),
            }
            .unwrap();

            return Ok(ActualAddress::RawAddress(address));
        };

        let Some(label) = WORD_REGEX.find(address) else {
            return Err(AddressError::CouldNotParseArgument);
        };

        Ok(ActualAddress::Label(label.as_str().to_owned()))
    }
}

enum AddressError {
    NoAddressProvided,
    SyntaxError(&'static str),
    CouldNotParseArgument,
}

impl TryFrom<&str> for Address {
    type Error = AddressError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (mode, address) = Address::parse_mode(value)?;
        let address = Address::parse_address(address)?;

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

struct Program {
    labels: ResolvedLabels,
}

enum ResolutionError {
    LabelDoesNotExists { label: Label },
    UnknownCommand(String),
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

struct SourceCodeCommand {
    opcode: String,
    argument: Argument,
}

impl SourceCodeCommand {
    fn compile(&self, program: &Program) -> Result<CompiledCommand, ResolutionError> {
        use ArgumentType::*;
        let metadata = match self.opcode.as_ref() {
            "IN" => SourceCommandMetadata {
                opcode: Opcode::IN,
                argument_type: Port,
            },
            "OUT" => SourceCommandMetadata {
                opcode: Opcode::OUT,
                argument_type: Port,
            },
            "LOAD" => SourceCommandMetadata {
                opcode: Opcode::LOAD,
                argument_type: Address,
            },
            "STORE" => SourceCommandMetadata {
                opcode: Opcode::STORE,
                argument_type: Address,
            },
            "ADD" => SourceCommandMetadata {
                opcode: Opcode::ADD,
                argument_type: Address,
            },
            "INC" => SourceCommandMetadata {
                opcode: Opcode::INC,
                argument_type: None,
            },
            "AND" => SourceCommandMetadata {
                opcode: Opcode::AND,
                argument_type: Address,
            },
            "ANDI" => SourceCommandMetadata {
                opcode: Opcode::AND,
                argument_type: Immidiate,
            },
            "CMP" => SourceCommandMetadata {
                opcode: Opcode::CMP,
                argument_type: Address,
            },
            "JZC" => SourceCommandMetadata {
                opcode: Opcode::JZC,
                argument_type: Address,
            },
            "JZS" => SourceCommandMetadata {
                opcode: Opcode::JZS,
                argument_type: Address,
            },
            "JZ" => SourceCommandMetadata {
                opcode: Opcode::JZS,
                argument_type: Address,
            },
            "JCC" => SourceCommandMetadata {
                opcode: Opcode::JCC,
                argument_type: Address,
            },
            "JCS" => SourceCommandMetadata {
                opcode: Opcode::JCS,
                argument_type: Address,
            },
            "JC" => SourceCommandMetadata {
                opcode: Opcode::JCS,
                argument_type: Address,
            },
            "JUMP" => SourceCommandMetadata {
                opcode: Opcode::JUMP,
                argument_type: Address,
            },
            "NOP" => SourceCommandMetadata {
                opcode: Opcode::NOP,
                argument_type: None,
            },
            "HALT" => SourceCommandMetadata {
                opcode: Opcode::HALT,
                argument_type: None,
            },
            _ => return Err(ResolutionError::UnknownCommand(self.opcode.clone())),
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

    let command_regex = Regex::new(r"\w^").unwrap();
    for line in lines {
        let mut start = 0;
        if let Some(label) = LABEL_REGEX.find(line) {
            labels.insert(label.as_str().to_owned(), commands.len());
            start = label.start();
        }

        let Some(command) = command_regex.find_at(&line[start..], start) else {
            continue;
        };
        start = command.end();
    }
}
