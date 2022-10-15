use crate::instructions::opcodes::decode;
use crate::instructions::{
    execute, get_ops_map, get_sequences_map, ExecutionStatus, MicroInstruction,
};
use crate::pinout::Pinout;
use crate::registers::{IndexRegister, RegisterFile, SelectedRegister8};
use std::slice::Iter;

pub struct Cpu {
    regs: RegisterFile,
    pins: Pinout,
    current_sequence: Option<Iter<'static, MicroInstruction>>,
    current_op: Option<Iter<'static, MicroInstruction>>,
    data_destination: Option<SelectedRegister8>,
    index_register: Option<IndexRegister>,
    instr_ready: bool,
    running_op: bool,
}

pub enum YieldStatus {
    ClockFinished,
    WaitingMemory,
}

impl Cpu {
    pub fn read_data_pins(&self) -> u8 {
        self.pins.get_data()
    }

    pub fn read_address_pins(&self) -> u16 {
        self.pins.get_address()
    }

    pub fn set_data_pins(&mut self, value: u8) {
        self.pins.set_data_input(value);
        let data_destination = self.data_destination.expect("no data destination set");
        self.regs.set_selected_register8(data_destination, value);
        if data_destination == SelectedRegister8::IR {
            self.instr_ready = true;
        }
    }

    pub fn run_op(&mut self) -> ExecutionStatus {
        let mut run_status = ExecutionStatus::Continue;

        if let Some(op) = &mut self.current_op {
            if let Some(&micro_instr) = op.next() {
                run_status = execute(
                    micro_instr,
                    self.index_register,
                    &mut self.regs,
                    &mut self.pins,
                );
            } else {
                self.running_op = false;
                self.current_op = None;
            }
        } else {
            self.running_op = false;
            self.current_op = None;
        }

        run_status
    }

    pub fn run_sequence(&mut self) -> ExecutionStatus {
        let run_status;
        let sequence = &mut self.current_sequence.as_mut().unwrap();
        if let Some(&micro_instr) = sequence.next() {
            run_status = execute(
                micro_instr,
                self.index_register,
                &mut self.regs,
                &mut self.pins,
            );
        } else {
            self.current_sequence = None;
            run_status = ExecutionStatus::Continue;
        }

        run_status
    }

    pub fn run(&mut self) -> YieldStatus {
        if self.instr_ready == true {
            self.decode_instr();
        }

        let mut run_status = ExecutionStatus::Continue;

        while run_status == ExecutionStatus::Continue {
            if self.running_op {
                run_status = self.run_op();
            } else if self.current_sequence.is_some() {
                run_status = self.run_sequence();
            } else {
                run_status = execute(
                    MicroInstruction::Fetch,
                    None,
                    &mut self.regs,
                    &mut self.pins,
                );
            }

            match run_status {
                ExecutionStatus::YieldClock => {}
                ExecutionStatus::Continue => {}
                ExecutionStatus::RunOp => {
                    self.running_op = true;
                    run_status = ExecutionStatus::Continue;
                }
                ExecutionStatus::WaitMemory { dst } => {
                    self.data_destination = dst;
                    return YieldStatus::WaitingMemory;
                }
                ExecutionStatus::RunOpAndFinish => {}
                ExecutionStatus::FinishInstruction => {}
            };
        }

        YieldStatus::ClockFinished
    }

    fn decode_instr(&mut self) {
        let opcode = self.regs.ir.get_u8();
        let decoded_intr = decode(opcode);

        self.current_sequence = get_sequences_map()
            .get(&decoded_intr.sequence)
            .map_or(None, |v| Some(v.iter()));

        self.current_op = get_ops_map()
            .get(&decoded_intr.operation)
            .map_or(None, |v| Some(v.iter()));

        self.index_register = decoded_intr.index;

        self.instr_ready = false;
    }
}
