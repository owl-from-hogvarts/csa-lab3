use std::collections::HashSet;

use super::MicrocodeStorage;

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Hash)]
pub enum Signal {
    // enables io
    IO,
    // by default io works in input mode
    WRITE_IO,
    // memory
    SELECT_MEM, // select input for data register
    WRITE_MEM,  // by default read from memory
    // latches
    WRITE_ACCUMULATOR,
    // this is to prevent data races when
    // address and data are latched in single
    // microinstruction
    // by default write to data
    WRITE_DATA_OR_ADDRESS,
    WRITE_ADDRESS, // select write to address
    WRITE_STATUS,
    WRITE_COMMAND,
    // program counter controls
    WRITE_PROGRAM_COUNTER, // just write
    // if both flags specified, either of flags set
    // triggers write that is OR is applied
    // to write signals
    WRITE_PROGRAM_COUNTER_Z,     // write if Z flag *set*
    WRITE_PROGRAM_COUNTER_C,     // write if C flag *set*
    WRITE_PROGRAM_COUNTER_CLEAR, // write if specified flag is *clear*
    // instead of *set*

    // ALU
    // by default addition is made
    AND,
    NOT_LEFT,
    NOT_RIGHT,
    // left + right + 1
    INC,
    // left alu input multiplexing
    // zero left has higher priority
    // (if both zero and PC are selected, zero will be outputted)
    // by default AC is selected
    ZERO_LEFT,
    SELECT_PC,

    // right alu input multiplexer
    // zero right has higher priority
    // by default data is selected
    ZERO_RIGHT,
    SELECT_CMD_OPERAND,

    // CONTROL UNIT
    // by default current_address + 1 happens
    // SELECT_MC_1   SELECT_MC_0   OUTPUT
    // 0             0             +1
    // 0             1             0
    // 1             0             ARG_TYPE
    // 1             1             OPCODE
    SELECT_MC_1,
    SELECT_MC_0,

    // processor control
    HALT,
}

pub type Microinstruction = HashSet<Signal>;

macro_rules! mc {
    ($($signals:ident),+) => {
        Microinstruction::from([$($signals),+])
    };
}

pub fn get_microcode() -> MicrocodeStorage {
    use Signal::*;
    vec![
        // instruction fetch
        // pc > addr
        mc![SELECT_PC, ZERO_RIGHT, WRITE_DATA_OR_ADDRESS, WRITE_ADDRESS],
        // pc += 1; mem[addr] > data
        mc![
            SELECT_PC,
            ZERO_RIGHT,
            INC,
            WRITE_PROGRAM_COUNTER,
            SELECT_MEM,
            WRITE_DATA_OR_ADDRESS
        ],
        mc![WRITE_COMMAND],
        mc![SELECT_MC_1],
        // operand fetch
        // immediate
        mc![SELECT_CMD_OPERAND, ZERO_LEFT, WRITE_DATA_OR_ADDRESS],
        mc![SELECT_MC_0, SELECT_MC_1],
        // absolute
        mc![
            SELECT_CMD_OPERAND,
            ZERO_LEFT,
            WRITE_DATA_OR_ADDRESS,
            WRITE_ADDRESS
        ],
        mc![SELECT_MEM, WRITE_DATA_OR_ADDRESS],
        mc![SELECT_MC_0, SELECT_MC_1],
        // relative
        mc![
            SELECT_PC,
            SELECT_CMD_OPERAND,
            WRITE_DATA_OR_ADDRESS,
            WRITE_ADDRESS
        ],
        mc![SELECT_MEM, WRITE_DATA_OR_ADDRESS],
        mc![SELECT_MC_0, SELECT_MC_1],
        // indirect relative
        mc![
            SELECT_PC,
            SELECT_CMD_OPERAND,
            WRITE_DATA_OR_ADDRESS,
            WRITE_ADDRESS
        ],
        mc![SELECT_MEM, WRITE_DATA_OR_ADDRESS],
        mc![ZERO_LEFT, WRITE_DATA_OR_ADDRESS, WRITE_ADDRESS],
        mc![SELECT_MEM, WRITE_DATA_OR_ADDRESS],
        mc![SELECT_MC_0, SELECT_MC_1],
        // execution
        // at this moment operand is stored in
        // data register
        // IN
        mc![IO, WRITE_ACCUMULATOR],
        // OUT
        mc![IO, WRITE_IO],
        // LOAD
        mc![ZERO_LEFT, WRITE_ACCUMULATOR],
        // STORE
        mc![ZERO_RIGHT, WRITE_DATA_OR_ADDRESS],
        mc![WRITE_MEM],
        // ADD
        mc![WRITE_ACCUMULATOR, WRITE_STATUS],
        // INC
        mc![ZERO_RIGHT, INC, WRITE_ACCUMULATOR, WRITE_STATUS],
        // AND
        mc![AND, WRITE_ACCUMULATOR, WRITE_STATUS],
        // CMP
        mc![NOT_RIGHT, INC, WRITE_STATUS],
        // JZC
        mc![
            SELECT_PC,
            SELECT_CMD_OPERAND,
            WRITE_PROGRAM_COUNTER_Z,
            WRITE_PROGRAM_COUNTER_CLEAR
        ],
        // JZS
        mc![WRITE_PROGRAM_COUNTER_Z],
        // JCC
        mc![WRITE_PROGRAM_COUNTER_C],
    ]
}
