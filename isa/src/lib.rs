pub type RawOperand = u16;
pub type RawPort = u8;

pub struct Operand {
    pub operand: RawOperand,
    pub operand_type: OperandType,
}

// address
// label, (label), !label, same with number

pub enum Opcode {
    IN,  // port
    OUT, // port

    LOAD,  // address
    STORE, // address

    ADD, // address
    INC, // none
    AND, // address
    CMP, // address

    JZC,  // address
    JZS,  // address
    JCS,  // address
    JCC,  // address
    JUMP, // address

    NOP,  // none
    HALT, // none
}

pub enum OperandType {
    None,
    Indirect,
    Absolute,
    Relative,
    Immidiate,
}
