use std::fmt::{Debug, Display};

use isa::{MemoryItem, Opcode, OperandType, RawAddress};

use crate::{io_controller::IOController, memory::Memory};

use self::{
    control_unit::{Microinstruction, Signal},
    data_path::{ALU_Config, Registers, ALU},
    status::Status,
};

mod control_unit;
mod data_path;
mod status;

type MicrocodeStorage = Vec<Microinstruction>;
type MicroInstructionCounter = usize;
type TRegisterValue = u32;

pub struct Cpu {
    io_controller: IOController,
    registers: Registers,
    status: Status,
    memory: Memory,
    microcode: MicrocodeStorage,
    microcode_program_counter: MicroInstructionCounter,
}

impl Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CPU")
            .field("io_controller", &self.io_controller)
            .field("registers", &self.registers)
            .field("status", &self.status)
            .field("microcode_program_counter", &self.microcode_program_counter)
            .finish()
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Registers:")?;
        writeln!(f, "{}", self.registers)?;
        writeln!(f, "Status: {}", self.status)?;
        writeln!(f, "MC Counter: {}", self.microcode_program_counter)
    }
}

impl Cpu {
    pub fn new(memory: Memory, io_controller: IOController) -> Self {
        Self {
            io_controller,
            registers: Registers::default(),
            status: Status {
                // all registers reset to zeroes
                zero: true,
                carry: false,
            },
            memory,
            microcode: control_unit::get_microcode(),
            microcode_program_counter: 0,
        }
    }

    pub fn start(mut self) {
        let mut instructions_executed = 0;
        let mut micro_instructions_executed = 0;
        loop {
            // rise
            let micro_instruction = self.microcode[self.microcode_program_counter].clone();
            if micro_instruction.contains(&Signal::HALT) {
                break;
            }

            let is_io = micro_instruction.contains(&Signal::IO);
            let is_io_write = micro_instruction.contains(&Signal::WRITE_IO);
            // lazy to defer access to data
            // otherwise may access instruction by accident
            let device_address = || u32::from(self.registers.data) as u8;

            if is_io && is_io_write {
                self.io_controller
                    .write(device_address(), self.registers.accumulator as u8);
            }

            if micro_instruction.contains(&Signal::WRITE_MEM) {
                self.memory[self.registers.address] = self.registers.data;
            }

            let left = if micro_instruction.contains(&Signal::ZERO_LEFT) {
                0
            } else if micro_instruction.contains(&Signal::SELECT_PC) {
                // no sign extension happens
                self.registers.program_counter as u32
            } else {
                self.registers.accumulator
            };

            let right_0 = micro_instruction.contains(&Signal::SELECT_RIGHT_DATA) as u8;
            let right_1 =
                (micro_instruction.contains(&Signal::SELECT_RIGHT_CMD_OPERAND) as u8) << 1;
            let right = right_1 | right_0;
            let right = match right {
                0b00 => 0,
                0b01 => self.registers.data.unwrap_data(),
                0b10 => self.registers.command.operand.operand as u32,
                0b11 => self.registers.address as u32,
                _ => unreachable!(),
            };

            let alu_config = ALU_Config {
                left,
                right,
                AND: micro_instruction.contains(&Signal::AND),
                NOT_LEFT: micro_instruction.contains(&Signal::NOT_LEFT),
                NOT_RIGHT: micro_instruction.contains(&Signal::NOT_RIGHT),
                INC: micro_instruction.contains(&Signal::INC),
                SHIFT: micro_instruction.contains(&Signal::SHIFT),
                SHIFT_LEFT: micro_instruction.contains(&Signal::SHIFT_LEFT),
            };

            let alu_output = ALU(alu_config);

            // fall
            if micro_instruction.contains(&Signal::WRITE_STATUS) {
                self.status = Status {
                    zero: alu_output.zero,
                    carry: alu_output.carry,
                };
            }

            if micro_instruction.contains(&Signal::WRITE_ACCUMULATOR) {
                if is_io {
                    // no sign extension happens
                    self.registers.accumulator = self.io_controller.read(device_address()) as u32;
                } else {
                    self.registers.accumulator = alu_output.value;
                }
            }

            if micro_instruction.contains(&Signal::WRITE_PROGRAM_COUNTER) {
                self.registers.program_counter = alu_output.value as RawAddress;
            }

            let invert_flags = micro_instruction.contains(&Signal::WRITE_PROGRAM_COUNTER_CLEAR);

            // Z invert write
            // 0 0      0
            // 1 0      1
            // 1 1      0
            // 0 1      1
            if micro_instruction.contains(&Signal::WRITE_PROGRAM_COUNTER_Z)
                && self.status.zero != invert_flags
            {
                self.registers.program_counter = alu_output.value as RawAddress;
            }

            if micro_instruction.contains(&Signal::WRITE_PROGRAM_COUNTER_C)
                && self.status.carry != invert_flags
            {
                self.registers.program_counter = alu_output.value as RawAddress;
            }

            if micro_instruction.contains(&Signal::WRITE_COMMAND) {
                if let MemoryItem::Command(command) = self.registers.data {
                    self.registers.command = command;
                } else {
                    panic!("Tried to write binary data into command register!")
                }
            }

            let select_memory = micro_instruction.contains(&Signal::SELECT_MEM);
            if micro_instruction.contains(&Signal::WRITE_DATA) {
                self.registers.data = if select_memory {
                    self.memory[self.registers.address]
                } else {
                    MemoryItem::Data(alu_output.value)
                }
            }

            if micro_instruction.contains(&Signal::WRITE_ADDRESS) && !select_memory {
                self.registers.address = alu_output.value as RawAddress;
            }

            log::info!("{}", self);

            let mc_0 = micro_instruction.contains(&Signal::SELECT_MC_0) as u8;
            let mc_1 = (micro_instruction.contains(&Signal::SELECT_MC_1) as u8) << 1;

            let mc = mc_0 | mc_1;
            self.microcode_program_counter = match mc {
                0b00 => self.microcode_program_counter + 1,
                0b01 => 0,
                0b10 => Self::operand_type_to_mc(self.registers.command.operand.operand_type),
                0b11 => Self::opcode_to_mc(self.registers.command.opcode),
                _ => unreachable!(),
            };

            if self.microcode_program_counter == 0 {
                instructions_executed += 1;
            }

            micro_instructions_executed += 1;
        }

        log::info!(
            "Instructions: {}; MC: {}",
            instructions_executed,
            micro_instructions_executed
        );
    }

    fn opcode_to_mc(opcode: Opcode) -> MicroInstructionCounter {
        // security mechanism is required here
        // each table entry should have bitmask of allowed argument types
        // if argument is not allowed, then processor should throw
        // security exception
        // after all, this is cratch too.
        // proper way would be to introduce command formats.
        // This is too complicated for the lab, so leaving it as is
        match opcode {
            Opcode::IN => 13,
            Opcode::OUT => 14,
            Opcode::LOAD => 15,
            Opcode::STORE => 16,
            Opcode::ADD => 18,
            Opcode::INC => 19,
            Opcode::AND => 20,
            Opcode::CMP => 21,
            Opcode::SHIFT_LEFT => 22,
            Opcode::SHIFT_RIGHT => 23,
            Opcode::JZC => 24,
            Opcode::JZS => 25,
            Opcode::JCC => 26,
            Opcode::JCS => 27,
            Opcode::JUMP => 28,
            // just fetch next instruction
            Opcode::NOP => 0,
            Opcode::HALT => 29,
        }
    }
    fn operand_type_to_mc(operand: OperandType) -> MicroInstructionCounter {
        match operand {
            OperandType::None => 3,
            OperandType::Indirect => 9,
            OperandType::Absolute => 5,
            OperandType::Relative => 7,
            OperandType::Immediate => 4,
        }
    }
}
