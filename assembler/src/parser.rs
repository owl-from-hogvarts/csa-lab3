
//! Constructs representation of a program with help of 
//! utility structures from `crate::source_code` module.

use std::collections::HashMap;

mod address;
pub(crate) mod errors;
pub(crate) mod token;
mod argument;
mod compiler_directive;
mod command;

use errors::*;

use crate::source_code::command::SourceCodeCommand;
use crate::source_code::CompilerDirective;
use crate::source_code::Index;
use crate::source_code::Label;
use crate::source_code::SourceCodeItem;

use self::token::Token;
use self::token::TokenStream;

#[derive(Default)]
pub struct ParsedProgram {
    pub labels: HashMap<Label, Index>,
    pub items: Vec<SourceCodeItem>,
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
        Ok(())
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

    Ok(())
}
