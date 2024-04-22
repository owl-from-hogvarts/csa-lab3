use std::{collections::HashMap, str::FromStr};

use isa::CompiledCommand;
use isa::CompiledProgram;
use isa::CompiledSection;
use isa::MemoryItem;
use isa::Opcode;
use isa::Operand;
use isa::OperandType;
use isa::RawAddress;
use isa::RawOperand;
use isa::RawPort;

use num::{Integer, Num};
use once_cell::sync::Lazy;
use regex::Regex;

mod errors;

use errors::*;

type Label = String;
type Index = usize;

type RawSections = HashMap<Index, RawAddress>;
type ResolvedLabels = HashMap<Label, RawAddress>;

// argument notion is related to source code
// while operand is all about compiled representation

// let's bring in notion of sections
// then, to resolve actual address of label we need to figure out
// to which section it does belong.
// after doing so, section start + relative offset to command gives
// actual command address.
// Word produces new section.
// labels -> address
// sections: index -> address
// when resolving label usage within command
// two components are required: command address and label address
// Label address is known at parsing time.
// As commands are considered to be a part of section,
// they obtain theirs address as (section start + (command_index - section_index))
// this

static WORD_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\w_]+").unwrap());
static LABEL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(format!(r"^{}?:", *WORD_REGEX).as_str()).unwrap());

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

#[derive(Clone)]
enum ActualAddress {
    RawAddress(RawAddress),
    Label(Label),
}

#[derive(Clone)]
struct Address {
    mode: AddressingMode,
    address: ActualAddress,
}

impl Address {
    fn parse_mode(input: &str) -> Result<(AddressingMode, &str), ParsingError> {
        if input.starts_with("!") {
            return Ok((AddressingMode::Absolute, &input[1..]));
        }

        let start_parentheses = input.starts_with("(");
        let ends_parentheses = input.ends_with(")");

        if start_parentheses != ends_parentheses {
            return Err(ParsingError::SyntaxError(
                r#"Because single parentheses found present assumes Relative mode.
                No matching parentheses was found!"#
                    .to_string(),
            ));
        }

        if start_parentheses && ends_parentheses {
            return Ok((AddressingMode::Indirect, &input[1..input.len() - 2]));
        }

        return Ok((AddressingMode::Relative, input));
    }

    fn parse_address(address: &str) -> Result<ActualAddress, ParsingError> {
        let address = address.trim();
        if address.starts_with(|value: char| ('0'..'9').contains(&value)) {
            // probably number
            let address = parse_number(address)?;

            return Ok(ActualAddress::RawAddress(address));
        };

        let Some(label) = WORD_REGEX.find(address) else {
            return Err(ParsingError::CouldNotParseArgument);
        };

        Ok(ActualAddress::Label(label.as_str().to_owned()))
    }
}

impl TryFrom<&str> for Address {
    type Error = ParsingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (mode, address) = Address::parse_mode(value)?;
        let address = Address::parse_address(address)?;

        Ok(Self { mode, address })
    }
}

#[derive(Clone)]
enum Argument {
    None,
    Port(RawPort),
    Immediate(RawOperand),
    Address(Address),
}

impl Argument {
    fn parse_none(_input: &str) -> Result<Argument, ParsingError> {
        Ok(Argument::None)
    }

    fn parse_port(input: &str) -> Result<Argument, ParsingError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(ParsingError::NoArgumentProvided);
        }

        Ok(Argument::Port(parse_number(input)?))
    }

    fn parse_immediate(input: &str) -> Result<Argument, ParsingError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(ParsingError::NoArgumentProvided);
        }

        Ok(Argument::Immediate(parse_number(input)?))
    }

    fn parse_address(input: &str) -> Result<Argument, ParsingError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(ParsingError::NoArgumentProvided);
        }

        let address: Address = input.try_into()?;

        Ok(Argument::Address(address))
    }

    fn to_operand(
        &self,
        labels: &ResolvedLabels,
        current_address: RawAddress,
    ) -> Result<Operand, CompilationError> {
        use isa::OperandType::*;
        Ok(match self {
            Argument::None => Operand {
                operand: 0,
                operand_type: None,
            },
            &Argument::Port(port) => Operand {
                operand: port as u16,
                operand_type: Immediate,
            },
            &Argument::Immediate(value) => Operand {
                operand: value,
                operand_type: Immediate,
            },
            Argument::Address(address) => {
                let actual_address =
                    match &address.address {
                        &ActualAddress::RawAddress(address) => address,
                        ActualAddress::Label(label) => labels.get(label).copied().ok_or(
                            CompilationError::LabelDoesNotExists {
                                label: label.clone(),
                            },
                        )?,
                    };
                let operand = match address.mode {
                    AddressingMode::Absolute => actual_address,
                    AddressingMode::Relative => actual_address.overflowing_sub(current_address).0,
                    AddressingMode::Indirect => actual_address - current_address,
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

struct SourceCommandMetadata {
    opcode: Opcode,
    argument_type: fn(&str) -> Result<Argument, ParsingError>,
}

#[derive(Clone)]
enum SourceCodeItem {
    Data(u32),
    Command(SourceCodeCommand),
}

#[derive(Clone)]
struct SourceCodeCommand {
    metadata: &'static SourceCommandMetadata,
    argument: Argument,
}

impl FromStr for SourceCodeCommand {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let opcode_match = WORD_REGEX
            .find(s)
            .expect("Non empty line with at least one word");

        let metadata = Self::get_metadata_by_opcode(opcode_match.as_str())?;
        let remainder = &s[opcode_match.end()..];
        let argument = (metadata.argument_type)(remainder)?;

        Ok(SourceCodeCommand { metadata, argument })
    }
}

impl SourceCodeCommand {
    fn get_metadata_by_opcode(
        opcode: &str,
    ) -> Result<&'static SourceCommandMetadata, ParsingError> {
        match opcode.to_uppercase().as_str() {
            "IN" => Ok(&SourceCommandMetadata {
                opcode: Opcode::IN,
                argument_type: Argument::parse_port,
            }),
            "OUT" => Ok(&SourceCommandMetadata {
                opcode: Opcode::OUT,
                argument_type: Argument::parse_port,
            }),
            "LOAD" => Ok(&SourceCommandMetadata {
                opcode: Opcode::LOAD,
                argument_type: Argument::parse_address,
            }),
            "STORE" => Ok(&SourceCommandMetadata {
                opcode: Opcode::STORE,
                argument_type: Argument::parse_address,
            }),
            "ADD" => Ok(&SourceCommandMetadata {
                opcode: Opcode::ADD,
                argument_type: Argument::parse_address,
            }),
            "INC" => Ok(&SourceCommandMetadata {
                opcode: Opcode::INC,
                argument_type: Argument::parse_none,
            }),
            "AND" => Ok(&SourceCommandMetadata {
                opcode: Opcode::AND,
                argument_type: Argument::parse_address,
            }),
            "ANDI" => Ok(&SourceCommandMetadata {
                opcode: Opcode::AND,
                argument_type: Argument::parse_immediate,
            }),
            "CMP" => Ok(&SourceCommandMetadata {
                opcode: Opcode::CMP,
                argument_type: Argument::parse_address,
            }),
            "JZC" => Ok(&SourceCommandMetadata {
                opcode: Opcode::JZC,
                argument_type: Argument::parse_address,
            }),
            "JZS" => Ok(&SourceCommandMetadata {
                opcode: Opcode::JZS,
                argument_type: Argument::parse_address,
            }),
            "JZ" => Ok(&SourceCommandMetadata {
                opcode: Opcode::JZS,
                argument_type: Argument::parse_address,
            }),
            "JCC" => Ok(&SourceCommandMetadata {
                opcode: Opcode::JCC,
                argument_type: Argument::parse_address,
            }),
            "JCS" => Ok(&SourceCommandMetadata {
                opcode: Opcode::JCS,
                argument_type: Argument::parse_address,
            }),
            "JC" => Ok(&SourceCommandMetadata {
                opcode: Opcode::JCS,
                argument_type: Argument::parse_address,
            }),
            "JUMP" => Ok(&SourceCommandMetadata {
                opcode: Opcode::JUMP,
                argument_type: Argument::parse_address,
            }),
            "NOP" => Ok(&SourceCommandMetadata {
                opcode: Opcode::NOP,
                argument_type: Argument::parse_none,
            }),
            "HALT" => Ok(&SourceCommandMetadata {
                opcode: Opcode::HALT,
                argument_type: Argument::parse_none,
            }),
            _ => return Err(ParsingError::UnknownCommand(opcode.to_owned())),
        }
    }

    fn compile(
        &self,
        labels: &ResolvedLabels,
        current_address: RawAddress,
    ) -> Result<CompiledCommand, CompilationError> {
        let Self { metadata, .. } = self;

        Ok(CompiledCommand {
            opcode: metadata.opcode,
            operand: self.argument.to_operand(labels, current_address)?,
        })
    }
}

#[derive(Clone, Copy)]
struct RawSection {
    start_index: Index,
    start_address: RawAddress,
}

impl From<(Index, RawAddress)> for RawSection {
    fn from(value: (Index, RawAddress)) -> Self {
        Self {
            start_index: value.0,
            start_address: value.1,
        }
    }
}

struct SourceCodeSection {
    items: Vec<SourceCodeItem>,
    start_address: RawAddress,
}

pub struct ParsedProgram {
    items: Vec<SourceCodeItem>,
    labels: ResolvedLabels,
    raw_sections: RawSections,
}

impl ParsedProgram {
    pub fn compile(self) -> Result<CompiledProgram, CompilationError> {
        let mut raw_sections: Vec<RawSection> =
            self.raw_sections.into_iter().map(|v| v.into()).collect();

        raw_sections.sort_unstable_by_key(|elem| elem.start_index);

        let items = self.items;

        let mut sections: Vec<SourceCodeSection> = Vec::new();
        sections.reserve(raw_sections.len());

        for section in raw_sections.windows(2) {
            let current = section[0];
            let next = section[1];
            let items = &items[current.start_index..next.start_index];

            let actual_size = items.len();
            if actual_size > (next.start_address - current.start_address).into() {
                return Err(CompilationError::SectionTooLarge {
                    address_span: (current.start_address..next.start_address),
                    actual_size,
                });
            }

            let items = Vec::from(items);
            sections.push(SourceCodeSection {
                items,
                start_address: current.start_address,
            })
        }

        let mut program = CompiledProgram {
            sections: Vec::new(),
        };
        program.sections.reserve(sections.len());

        for source_section in sections {
            let mut compiled_section = CompiledSection {
                start_address: source_section.start_address,
                items: Vec::new(),
            };
            compiled_section.items.reserve(source_section.items.len());

            for (offset, item) in source_section.items.iter().enumerate() {
                // +1 as PC for current command already points to next command
                let item_address = source_section.start_address + offset as u16 + 1;
                let memory_item = match item {
                    &SourceCodeItem::Data(data) => MemoryItem::Data(data),
                    SourceCodeItem::Command(command) => {
                        MemoryItem::Command(command.compile(&self.labels, item_address)?)
                    }
                };

                compiled_section.items.push(memory_item);
            }

            program.sections.push(compiled_section);
        }

        Ok(program)
    }
}

pub fn parse_asm(input: impl AsRef<str>) -> Result<ParsedProgram, ParsingErrorOnLine> {
    let input = input.as_ref();
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

    let mut labels: ResolvedLabels = ResolvedLabels::new();
    let mut items: Vec<SourceCodeItem> = Vec::new();
    let mut raw_sections = RawSections::new();
    // first command may lie on first address
    raw_sections.insert(0, 0x0);

    let mut current_address: RawAddress = 0x0;

    for (line_number, line) in lines.enumerate() {
        let maybe_label = LABEL_REGEX.find(line);

        let start = maybe_label.map_or(0, |label_match| label_match.end());
        let command_string = &line[start..].trim();

        let mut command_processed = false;
        if command_string.to_uppercase().starts_with("ORG") {
            let address = command_string[3..].trim();
            let address: RawAddress =
                parse_number(address).map_err(|err: <RawAddress as Num>::FromStrRadixErr| {
                    ParsingErrorOnLine {
                        error: err.into(),
                        line_number,
                    }
                })?;
            current_address = address;
            raw_sections.insert(items.len(), current_address);
            command_processed = true;
        }

        // order is important!
        // label points to the address past an ORG command
        // but to the start of a WORD
        if let Some(label) = maybe_label {
            let label = label.as_str()[..label.end() - 1].to_owned();
            if labels.contains_key(&label) {
                return Err(ParsingErrorOnLine {
                    error: ParsingError::MultipleDefinitions(label),
                    line_number,
                });
            }
            labels.insert(label, current_address);
            if command_processed {
                continue;
            }
        }

        if command_string.to_uppercase().starts_with("WORD") {
            let argument: u32 = match parse_number(command_string[4..].trim()) {
                Ok(argument) => argument,
                Err(err) => {
                    return Err(ParsingErrorOnLine {
                        error: err.into(),
                        line_number,
                    })
                }
            };

            current_address += 1;
            items.push(SourceCodeItem::Data(argument));
            continue;
        }

        if command_string.is_empty() {
            continue;
        }

        match SourceCodeCommand::from_str(command_string) {
            Ok(command) => items.push(SourceCodeItem::Command(command)),
            Err(error) => return Err(ParsingErrorOnLine { error, line_number }),
        }

        current_address += 1;
    }

    raw_sections.insert(items.len(), current_address);

    Ok(ParsedProgram {
        items,
        labels,
        raw_sections,
    })
}

fn parse_number<T: Integer>(input: &str) -> Result<T, <T as Num>::FromStrRadixErr> {
    let (prefix, value) = if input.len() > 2 {
        input.split_at(2)
    } else {
        ("", input)
    };

    match prefix {
        "0x" => T::from_str_radix(value, 16),
        "0b" => T::from_str_radix(value, 2),
        _ => T::from_str_radix(input, 10),
    }
}
