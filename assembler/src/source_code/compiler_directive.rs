use isa::RawAddress;

use super::Label;

#[derive(Clone)]
pub enum CompilerDirective {
    Data(Vec<u32>),
    Pointer(Label),
    SetAddress(RawAddress),
}

