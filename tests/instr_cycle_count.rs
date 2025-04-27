#[cfg(feature = "disassembly")]
use cpu6502::instr::destruct_sequence;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct OpcodeInfo {
    opcode: u8,
    mnemonics: Vec<String>,
    #[serde(rename = "addressingMode")]
    addressing_mode: Option<String>,
    cycles: Option<u8>,
    #[serde(rename = "pageBoundaryCycle")]
    page_boundary_cycle: Option<bool>,
    illegal: bool,
}

#[test]
#[cfg(feature = "disassembly")]
fn test() {
    use std::fs;

    let json_text = fs::read_to_string("testdata/all_6502.json")
        .expect("Could not open testdata/all_6502.json");

    let opcodes: Vec<OpcodeInfo> = serde_json::from_str(&json_text).expect("Could not parse json");

    let opcode_data = opcodes.into_iter().filter(|e| e.illegal == false).map(|e| {
        (
            e.opcode,
            e.cycles.unwrap_or(1),
            e.page_boundary_cycle.unwrap_or(false),
        )
    });

    for (opcode, mut expected_cycle_count, page_boundary) in opcode_data {
        if page_boundary {
            expected_cycle_count += 1;
        }

        let decoded = cpu6502::decode(opcode);
        let (sequence, operation, _) = destruct_sequence(decoded);
        let sequence = cpu6502::sequence_for_mode(sequence);

        let cycles = sequence
            .into_iter()
            .filter(|&&e| {
                matches!(
                    e,
                    cpu6502::MicroInstruction::YieldClock
                        | cpu6502::MicroInstruction::FinishInstruction
                )
            })
            .count();
        let mut cycles = (cycles + 1) as u8; // +1 for fetch
        if matches!(
            operation,
            cpu6502::InstructionOp::BranchPlus
                | cpu6502::InstructionOp::BranchMinus
                | cpu6502::InstructionOp::BranchOverflowClear
                | cpu6502::InstructionOp::BranchOverflowSet
                | cpu6502::InstructionOp::BranchCarryClear
                | cpu6502::InstructionOp::BranchCarrySet
                | cpu6502::InstructionOp::BranchNotEqual
                | cpu6502::InstructionOp::BranchEqual
        ) {
            cycles -= 1;
        }

        assert_eq!(cycles, expected_cycle_count, "opcode {:#04X}", opcode);
    }
}
