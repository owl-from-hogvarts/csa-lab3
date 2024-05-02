use std::collections::HashSet;

use super::MicrocodeStorage;

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
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
    WRITE_DATA,
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

    // Commutator
    SHIFT_LEFT,

    // left alu input multiplexing
    // zero left has higher priority
    // (if both zero and PC are selected, zero will be outputted)
    // by default AC is selected
    ZERO_LEFT,
    SELECT_PC,

    // RIGHT MULTIPLEXOR
    // SELECT_RIGHT_CMD_OPERAND | SELECT_RIGHT_DATA | OUTPUT
    // -------------------------| ----------------- | ----------
    // 0                        | 0                 | 0
    // 0                        | 1                 | DATA
    // 1                        | 0                 | CMD_OPERAND
    // 1                        | 1                 | ADDRESS
    SELECT_RIGHT_CMD_OPERAND,
    SELECT_RIGHT_DATA,

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
        // pc -> addr
        /* 0 */
        mc![SELECT_PC, WRITE_ADDRESS],
        // pc += 1; mem[addr] -> data
        /* 1 */
        mc![
            SELECT_PC,
            INC,
            WRITE_PROGRAM_COUNTER,
            SELECT_MEM,
            WRITE_DATA
        ],
        /* 2 */
        mc![WRITE_COMMAND, SELECT_MC_1],
        // ----

        // operand fetch
        // none
        /* 3 */
        mc![SELECT_MC_0, SELECT_MC_1],
        // immediate
        /* 4 */
        mc![
            SELECT_RIGHT_CMD_OPERAND,
            ZERO_LEFT,
            WRITE_DATA,
            // this is cratch. see notes on "jump" microcode
            WRITE_ADDRESS,
            SELECT_MC_0,
            SELECT_MC_1
        ],
        // absolute
        /* 5 */
        mc![SELECT_RIGHT_CMD_OPERAND, ZERO_LEFT, WRITE_ADDRESS],
        /* 6 */
        mc![SELECT_MEM, WRITE_DATA, SELECT_MC_0, SELECT_MC_1],
        // ----

        // relative
        /* 7 */
        mc![SELECT_PC, SELECT_RIGHT_CMD_OPERAND, WRITE_ADDRESS],
        /* 8 */
        mc![SELECT_MEM, WRITE_DATA, SELECT_MC_0, SELECT_MC_1],
        // ----

        // indirect relative
        /* 9 */
        mc![SELECT_PC, SELECT_RIGHT_CMD_OPERAND, WRITE_ADDRESS],
        /* 10 */
        mc![SELECT_MEM, WRITE_DATA],
        /* 11 */
        mc![ZERO_LEFT, SELECT_RIGHT_DATA, WRITE_ADDRESS],
        /* 12 */
        mc![SELECT_MEM, WRITE_DATA, SELECT_MC_0, SELECT_MC_1],
        // ----

        // execution
        // at this moment operand is stored in
        // data register

        // io
        // IN
        /* 13 */
        mc![IO, WRITE_ACCUMULATOR, SELECT_MC_0],
        // OUT
        /* 14 */
        mc![IO, WRITE_IO, SELECT_MC_0],
        // ----

        // memory
        // LOAD
        /* 15 */
        mc![ZERO_LEFT, SELECT_RIGHT_DATA, WRITE_ACCUMULATOR, SELECT_MC_0],
        // STORE
        /* 16 */
        mc![WRITE_DATA],
        /* 17 */
        mc![WRITE_MEM, SELECT_MC_0],
        // ----

        // operations
        // ADD
        /* 18 */
        mc![
            SELECT_RIGHT_DATA,
            WRITE_ACCUMULATOR,
            WRITE_STATUS,
            SELECT_MC_0
        ],
        // INC
        /* 19 */
        mc![INC, WRITE_ACCUMULATOR, WRITE_STATUS, SELECT_MC_0],
        // AND
        /* 20 */
        mc![
            AND,
            SELECT_RIGHT_DATA,
            WRITE_ACCUMULATOR,
            WRITE_STATUS,
            SELECT_MC_0
        ],
        // CMP
        /* 21 */
        mc![SELECT_RIGHT_DATA, NOT_RIGHT, INC, WRITE_STATUS, SELECT_MC_0],
        // SHIFT_LEFT
        /* 22 */
        mc![SHIFT_LEFT, WRITE_ACCUMULATOR, SELECT_MC_0],
        // ----

        // jumps
        // jumps only support absolute, relative, indirect operand types
        // therefore if compiler occasionally produces jump command
        // with operand type of Immediate
        // undefined behaviour occurs!
        // Then it's possible vulnerability

        // solutions: for one variant see cpu.rs

        // cratch is implemented: too short on time
        // cratch: for "immediate" operand type, override address
        // with immediate value

        // JZC
        /* 23 */
        mc![
            ZERO_LEFT,
            SELECT_RIGHT_CMD_OPERAND,
            SELECT_RIGHT_DATA,
            WRITE_PROGRAM_COUNTER_Z,
            WRITE_PROGRAM_COUNTER_CLEAR,
            SELECT_MC_0
        ],
        // JZS
        /* 24 */
        mc![
            ZERO_LEFT,
            SELECT_RIGHT_CMD_OPERAND,
            SELECT_RIGHT_DATA,
            WRITE_PROGRAM_COUNTER_Z,
            SELECT_MC_0
        ],
        // JCC
        /* 25 */
        mc![
            ZERO_LEFT,
            SELECT_RIGHT_CMD_OPERAND,
            SELECT_RIGHT_DATA,
            WRITE_PROGRAM_COUNTER_C,
            WRITE_PROGRAM_COUNTER_CLEAR,
            SELECT_MC_0
        ],
        // JCS
        /* 26 */
        mc![
            ZERO_LEFT,
            SELECT_RIGHT_CMD_OPERAND,
            SELECT_RIGHT_DATA,
            WRITE_PROGRAM_COUNTER_C,
            SELECT_MC_0
        ],
        // JUMP
        /* 27 */
        mc![
            ZERO_LEFT,
            SELECT_RIGHT_CMD_OPERAND,
            SELECT_RIGHT_DATA,
            WRITE_PROGRAM_COUNTER,
            SELECT_MC_0
        ],
        // NOP
        // well do nothing

        // HALT
        /* 28 */
        mc![HALT, SELECT_MC_0],
    ]
}
