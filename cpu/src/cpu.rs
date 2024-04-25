use std::collections::HashSet;

use isa::{MemoryItem, Opcode, OperandType};

use self::{
    control_unit::{Microinstruction, Signal},
    data_path::{ALU_Config, Registers, ALU},
    io_controller::IOController,
    shared::Status,
};

mod control_unit;
mod data_path;
mod io_controller;
mod shared;

type MicrocodeStorage = Vec<Microinstruction>;
type MicroInstructionCounter = usize;
type TRegisterValue = u32;

pub struct CPU {
    io_controller: IOController,
    registers: Registers,
    status: Status,
    memory: Vec<MemoryItem>,
    microcode: MicrocodeStorage,
    microcode_program_counter: MicroInstructionCounter,
}

impl CPU {
    pub fn start(mut self) {
        loop {
            // rise
            let micro_instruction = self.microcode[self.microcode_program_counter];
            if micro_instruction.get(&Signal::HALT).is_some() {
                break;
            }

            let is_io = micro_instruction.get(&Signal::IO).is_some();
            let is_io_write = micro_instruction.get(&Signal::WRITE_IO).is_some();

            if is_io {
                let device_address = u32::from(self.registers.data) as u8;
                if is_io_write {
                    self.io_controller
                        .write(device_address, self.registers.accumulator as u8);
                } else {
                    // no sign extension happens
                    self.registers.accumulator = self.io_controller.read(device_address) as u32;
                }
            }

            if micro_instruction.get(&Signal::WRITE_MEM).is_some() {
                self.memory[self.registers.address as usize] = self.registers.data;
            }

            let left = if micro_instruction.get(&Signal::ZERO_LEFT).is_some() {
                0
            } else {
                if micro_instruction.get(&Signal::SELECT_PC).is_some() {
                    self.registers.program_counter
                } else {
                    self.registers.accumulator
                }
            };

            let right = if micro_instruction.get(&Signal::ZERO_RIGHT).is_some() {
                0
            } else {
                if micro_instruction.get(&Signal::SELECT_CMD_OPERAND).is_some() {
                    self.registers.command.operand.operand as TRegisterValue
                } else {
                    self.registers.data.unwrap_data()
                }
            };

            let alu_config = ALU_Config {
                left,
                right,
                AND: micro_instruction.get(&Signal::AND).is_some(),
                NOT_LEFT: micro_instruction.get(&Signal::NOT_LEFT).is_some(),
                NOT_RIGHT: micro_instruction.get(&Signal::NOT_RIGHT).is_some(),
                INC: micro_instruction.get(&Signal::INC).is_some(),
            };

            let alu_output = ALU(alu_config);

            // fall
            if micro_instruction.get(&Signal::WRITE_STATUS).is_some() {
                self.status = Status {
                    zero: alu_output.zero,
                    carry: alu_output.carry,
                };
            }

            if micro_instruction.get(&Signal::WRITE_ACCUMULATOR).is_some() {
                self.registers.accumulator = alu_output.value;
            }

            if micro_instruction
                .get(&Signal::WRITE_PROGRAM_COUNTER)
                .is_some()
            {
                self.registers.program_counter = alu_output.value;
            }

            let invert_flags = micro_instruction
                .get(&Signal::WRITE_PROGRAM_COUNTER_CLEAR)
                .is_some();

            // Z invert write
            // 0 0      0
            // 1 0      1
            // 1 1      0
            // 0 1      1
            if micro_instruction
                .get(&Signal::WRITE_PROGRAM_COUNTER_Z)
                .is_some()
            {
                if self.status.zero != invert_flags {
                    self.registers.program_counter = alu_output.value;
                }
            }

            if micro_instruction
                .get(&Signal::WRITE_PROGRAM_COUNTER_C)
                .is_some()
            {
                if self.status.carry != invert_flags {
                    self.registers.program_counter = alu_output.value;
                }
            }

            if micro_instruction.get(&Signal::WRITE_COMMAND).is_some() {
                if let MemoryItem::Command(command) = self.registers.data {
                    self.registers.command = command;
                } else {
                    panic!("Tryed to write binary data into command register!")
                }
            }

            if micro_instruction
                .get(&Signal::WRITE_DATA_OR_ADDRESS)
                .is_some()
            {
                if micro_instruction.get(&Signal::WRITE_ADDRESS).is_some() {
                    self.registers.address = alu_output.value;
                } else {
                    self.registers.data = if micro_instruction.get(&Signal::SELECT_MEM).is_some() {
                        self.memory[self.registers.address as usize]
                    } else {
                        MemoryItem::Data(alu_output.value)
                    }
                }
            }

            let mc_0 = micro_instruction.get(&Signal::SELECT_MC_0).is_some() as u8;
            let mc_1 = (micro_instruction.get(&Signal::SELECT_MC_1).is_some() as u8) << 1;

            let mc = mc_0 | mc_1;
            self.microcode_program_counter = match mc {
                0b00 => self.microcode_program_counter + 1,
                0b01 => 0,
                0b10 => Self::operand_type_to_mc(self.registers.command.operand.operand_type),
                0b11 => Self::opcode_to_mc(self.registers.command.opcode),
            }
        }
    }

    fn opcode_to_mc(opcode: Opcode) -> MicroInstructionCounter {}
    fn operand_type_to_mc(operand: OperandType) -> MicroInstructionCounter {}
}
