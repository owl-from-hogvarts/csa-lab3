
//! Structures which represent source code

pub mod command;
mod compiler_directive;

pub use compiler_directive::*;
use isa::RawAddress;

use self::command::SourceCodeCommand; 

pub type Label = String;
pub type Index = usize;


#[derive(Clone)]
pub enum SourceCodeItem {
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

