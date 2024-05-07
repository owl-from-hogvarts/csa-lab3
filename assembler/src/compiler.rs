

//! Does the compilation. Resolves labels to actual addresses.
//! Resolves references to labels. Generates sections from
//! assembler directives

use std::collections::HashMap;

use isa::CompiledProgram;
use isa::CompiledSection;
use isa::MemoryItem;
use isa::RawAddress;

use crate::parser::ParsedProgram;
use crate::source_code::{CompilerDirective, Index, Label, SourceCodeItem};

use self::errors::CompilationError;

mod command;
mod errors;

type ResolvedLabels = HashMap<Label, RawAddress>;

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

impl ParsedProgram {
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
