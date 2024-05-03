use std::collections::HashMap;

use isa::Opcode;
use isa::Operand;
use isa::OperandType;
use isa::RawAddress;
use isa::RawOperand;
use isa::RawPort;
use isa::{CompiledCommand, CompiledSection};
use isa::{CompiledProgram, MemoryItem};

use num::ToPrimitive;

mod address;
mod errors;
mod token;

use errors::*;

use crate::parser::address::AddressingMode;
use crate::parser::address::Reference;

use self::address::AddressWithMode;
use self::token::Token;
use self::token::TokenStream;

type Label = String;
type Index = usize;

type ResolvedLabels = HashMap<Label, RawAddress>;

// argument notion is related to source code
// while operand is all about compiled representation

#[derive(Clone)]
enum Argument {
    None,
    Port(RawPort),
    Immediate(RawOperand),
    Address(AddressWithMode),
}

impl Argument {
    fn parse_none(stream: &mut TokenStream) -> Result<Argument, ParsingError> {
        stream.next_end_of_input()?;
        Ok(Argument::None)
    }

    fn parse_port(stream: &mut TokenStream) -> Result<Argument, ParsingError> {
        let port_number = stream.next_number()?;

        Ok(Argument::Port(
            port_number
                .to_u8()
                .ok_or(ParsingError::CouldNotParseArgument)?,
        ))
    }

    fn parse_immediate(stream: &mut TokenStream) -> Result<Argument, ParsingError> {
        Ok(Argument::Immediate(stream.next_number()?))
    }

    fn parse_address(stream: &mut TokenStream) -> Result<Argument, ParsingError> {
        let address: AddressWithMode = stream.try_into()?;

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
                        &Reference::RawAddress(address) => address,
                        Reference::Label(label) => labels.get(label).copied().ok_or(
                            CompilationError::LabelDoesNotExists {
                                label: label.clone(),
                            },
                        )?,
                    };
                let operand = match address.mode {
                    AddressingMode::Absolute => actual_address,
                    AddressingMode::Relative | AddressingMode::Indirect => {
                        // relative addressing mode is relative to Program Counter.
                        // Program counter always points to the next command
                        actual_address.overflowing_sub(current_address + 1).0
                    }
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
    argument_type: fn(&mut TokenStream) -> Result<Argument, ParsingError>,
}

#[derive(Clone)]
enum CompilerDirective {
    Data(Vec<u32>),
    Pointer(Label),
    SetAddress(RawAddress),
}

impl CompilerDirective {
    fn from_token_stream(stream: &mut TokenStream) -> Result<Option<Self>, ParsingError> {
        if let Token::Word(command) = stream.peek(1)? {
            match command.to_uppercase().as_str() {
                "WORD" => {
                    // advance stream on match only
                    stream.next_word()?;
                    let mut data = Vec::new();

                    if let Ok(label) = stream.next_word() {
                        return Ok(Some(Self::Pointer(label)));
                    }

                    while let Ok(number) = stream.next_long_number() {
                        data.push(number as u32);
                    }

                    // ensure that no unparsed input left
                    stream.next_end_of_input()?;

                    return Ok(Some(Self::Data(data)));
                }
                "ORG" => {
                    stream.next_word()?;
                    let address = stream.next_number()?;

                    return Ok(Some(Self::SetAddress(address)));
                }
                _ => return Ok(None),
            };
        }

        Ok(None)
    }
}

#[derive(Clone)]
enum SourceCodeItem {
    Command(SourceCodeCommand),
    CompilerDirective(CompilerDirective),
}

impl SourceCodeItem {
    /// returns size of an item in cells
    ///
    /// currently each cell is 4 bytes long that is u32
    pub fn size(&self) -> RawAddress {
        match self {
            SourceCodeItem::Command(_) => 1,
            SourceCodeItem::CompilerDirective(directive) => match directive {
                CompilerDirective::Data(data) => data
                    .len()
                    .try_into()
                    .expect("Too big data item! It won't fit into cpu's memory"),
                CompilerDirective::SetAddress(_) => 0,
                CompilerDirective::Pointer(_) => 1,
            },
        }
    }
}

#[derive(Clone)]
struct SourceCodeCommand {
    metadata: &'static SourceCommandMetadata,
    argument: Argument,
}

impl SourceCodeCommand {
    fn from_token_stream(stream: &mut TokenStream) -> Result<Self, ParsingError> {
        let opcode = stream.next_word()?;

        let metadata = Self::get_metadata_by_opcode(opcode.as_str())?;
        let argument = (metadata.argument_type)(stream)?;

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
            "SHIFT_LEFT" => Ok(&SourceCommandMetadata {
                opcode: Opcode::SHIFT_LEFT,
                argument_type: Argument::parse_none,
            }),
            "SHIFT_RIGHT" => Ok(&SourceCommandMetadata {
                opcode: Opcode::SHIFT_RIGHT,
                argument_type: Argument::parse_none,
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

struct AddressIterator<T> {
    items: T,
    next_address: RawAddress,
    next_index_to_consume: usize,
}

impl<'a> AddressIterator<std::slice::Iter<'a, SourceCodeItem>> {
    fn new(items: &'a Vec<SourceCodeItem>) -> Self {
        Self {
            items: items.iter(),
            next_address: 0,
            next_index_to_consume: 0,
        }
    }

    /// Consumes n elements from the very start of original iterator
    fn nth_form_start(&mut self, n: usize) -> Option<<Self as Iterator>::Item> {
        assert!(n >= self.next_index_to_consume, "can't go backward!");
        let advance_by = n - self.next_index_to_consume;

        self.nth(advance_by)
    }
}

impl<'a, T: Iterator<Item = &'a SourceCodeItem>> Iterator for AddressIterator<T> {
    type Item = (RawAddress, &'a SourceCodeItem);

    fn next(&mut self) -> Option<Self::Item> {
        // item -> address
        let item = self.items.next()?;
        let mut current_address = self.next_address;
        self.next_index_to_consume += 1;

        self.next_address = if let &SourceCodeItem::CompilerDirective(
            CompilerDirective::SetAddress(address),
        ) = item
        {
            // this directive is zero sized
            // so it sort of lies at the same memory address
            // as next element
            current_address = address;
            current_address
        } else {
            self.next_address + item.size()
        };

        Some((current_address, item))
    }
}

#[derive(Default)]
pub struct ParsedProgram {
    labels: HashMap<Label, Index>,
    items: Vec<SourceCodeItem>,
}

impl ParsedProgram {
    fn has_label(&self, label: &Label) -> bool {
        self.labels.contains_key(label)
    }

    fn insert_label(&mut self, label: Label) -> Result<(), ParsingError> {
        if self.has_label(&label) {
            return Err(ParsingError::MultipleDefinitions(label));
        }

        self.labels.insert(label, self.items.len());
        return Ok(());
    }

    fn addresses(items: &Vec<SourceCodeItem>) -> AddressIterator<std::slice::Iter<SourceCodeItem>> {
        AddressIterator::new(items)
    }

    pub fn compile(self) -> Result<CompiledProgram, CompilationError> {
        // RESOLVE LABELS

        let mut labels: Vec<(Label, Index)> = self.labels.into_iter().collect();
        // sorting is required to NOT go back in indexes which labels points to.
        labels.sort_by_key(|&(_, index)| index);

        let mut resolved_labels: ResolvedLabels = ResolvedLabels::new();
        let mut addresses = ParsedProgram::addresses(&self.items);

        for (label, label_index) in labels {
            let (resolved_address, _) = addresses
                .nth_form_start(label_index)
                // this should be impossible, since labels always points to items
                // and items are neither removed nor inserted after parsing
                .expect("labels points OUT of program!");
            resolved_labels.insert(label, resolved_address);
        }

        // compile and distribute commands among sections
        let mut sections: Vec<CompiledSection> = Vec::new();
        // base section
        sections.push(CompiledSection::with_address(0));

        // create sections from ORG commands
        // distribute items among sections
        for (current_address, item) in ParsedProgram::addresses(&self.items) {
            let current_section = sections
                .last_mut()
                .expect("At least default section must be present");

            match item {
                SourceCodeItem::CompilerDirective(directive) => match directive {
                    CompilerDirective::SetAddress(address) => {
                        sections.push(CompiledSection::with_address(*address));
                        continue;
                    }
                    CompilerDirective::Data(data) => data
                        .into_iter()
                        .map(|&byte| MemoryItem::Data(byte as u32))
                        .for_each(|item| current_section.items.push(item)),
                    CompilerDirective::Pointer(label) => {
                        current_section.items.push(MemoryItem::Data(
                            resolved_labels
                                .get(label)
                                .map(|&address| address as u32)
                                .ok_or(CompilationError::LabelDoesNotExists {
                                    label: label.clone(),
                                })?,
                        ));
                    }
                },
                SourceCodeItem::Command(command) => {
                    let memory_item =
                        MemoryItem::Command(command.compile(&resolved_labels, current_address)?);
                    current_section.items.push(memory_item);
                }
            };
        }

        Ok(CompiledProgram { sections })
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

    let mut program = ParsedProgram::default();

    for (line_index, line) in lines.enumerate() {
        let line_number = line_index + 1;
        process_line(line, &mut program)
            .map_err(|error| ParsingErrorOnLine { error, line_number })?;
    }

    Ok(program)
}

// this function is required to easily convert parsing error
// into parsing error on line
fn process_line(line: &str, program: &mut ParsedProgram) -> Result<(), ParsingError> {
    let mut tokens: TokenStream = line.parse()?;

    // line: statement "\n" | "\n"
    // statement: label | command | labeled_statement
    // labeled_statement: label command
    // command: word ???
    // label: word ":"

    // parse label
    const LABEL_DELIMITER: char = ':';
    if let Ok(&Token::SpecialSymbol(LABEL_DELIMITER)) = tokens.peek(2) {
        let label = tokens.next_word()?;
        // consume label delimiter
        tokens
            .next_special_symbol(LABEL_DELIMITER)
            .expect("already checked with peek");
        program.insert_label(label.clone())?;
    }

    // just label or empty line
    if tokens.next_end_of_input().is_ok() {
        return Ok(());
    }

    let compiler_directive = CompilerDirective::from_token_stream(&mut tokens)?;
    if let Some(directive) = compiler_directive {
        program
            .items
            .push(SourceCodeItem::CompilerDirective(directive));
        return Ok(());
    }

    let command = SourceCodeCommand::from_token_stream(&mut tokens)?;
    program.items.push(SourceCodeItem::Command(command));

    return Ok(());
}
