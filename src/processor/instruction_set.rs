use std::collections::HashMap;

use crate::interfaces::Bus as _;
use crate::processor::instruction::{
    AddressingMode, Instruction, InstructionKind, MiscInstructionKind, Opcode,
};
use crate::processor::internal_cpu::InternalCpu;
use crate::processor::status_register::{StatusRegister, StatusRegisterFlag};
use crate::types::SharedBus;
use crate::utils;

use AddressingMode::*;
use InstructionKind::*;
use MiscInstructionKind::*;
use StatusRegisterFlag::*;

pub struct InstructionSet {
    instruction_set: HashMap<Opcode, Instruction>,
}

impl InstructionSet {
    #[rustfmt::skip]
    pub fn new_legal_opcode_set() -> Self {
        let mut instruction_set = HashMap::new();

        let instructions = [
            // Transfer instructions
            Instruction {
                name: "LDA",
                opcode: 0xA9,
                instruction: InternalExecOnMemoryData(lda),
                addressing_mode: Immediate,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "LDA",
                opcode: 0xA5,
                instruction: InternalExecOnMemoryData(lda),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "LDA",
                opcode: 0xB5,
                instruction: InternalExecOnMemoryData(lda),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            Instruction {
                name: "LDA",
                opcode: 0xAD,
                instruction: InternalExecOnMemoryData(lda),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "LDA",
                opcode: 0xBD,
                instruction: InternalExecOnMemoryData(lda),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "LDA",
                opcode: 0xB9,
                instruction: InternalExecOnMemoryData(lda),
                addressing_mode: AbsoluteY,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "LDA",
                opcode: 0xA1,
                instruction: InternalExecOnMemoryData(lda),
                addressing_mode: IndirectX,
                bytes: 2,
                cycles: 6,
            },
            Instruction {
                name: "LDA",
                opcode: 0xB1,
                instruction: InternalExecOnMemoryData(lda),
                addressing_mode: IndirectY,
                bytes: 2,
                cycles: 5,
            },

            Instruction {
                name: "LDX",
                opcode: 0xA2,
                instruction: InternalExecOnMemoryData(ldx),
                addressing_mode: Immediate,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "LDX",
                opcode: 0xA6,
                instruction: InternalExecOnMemoryData(ldx),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "LDX",
                opcode: 0xB6,
                instruction: InternalExecOnMemoryData(ldx),
                addressing_mode: ZeroPageY,
                bytes: 2,
                cycles: 4,
            },
            Instruction {
                name: "LDX",
                opcode: 0xAE,
                instruction: InternalExecOnMemoryData(ldx),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "LDX",
                opcode: 0xBE,
                instruction: InternalExecOnMemoryData(ldx),
                addressing_mode: AbsoluteY,
                bytes: 3,
                cycles: 4,
            },

            Instruction {
                name: "LDY",
                opcode: 0xA0,
                instruction: InternalExecOnMemoryData(ldy),
                addressing_mode: Immediate,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "LDY",
                opcode: 0xA4,
                instruction: InternalExecOnMemoryData(ldy),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "LDY",
                opcode: 0xB4,
                instruction: InternalExecOnMemoryData(ldy),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            Instruction {
                name: "LDY",
                opcode: 0xAC,
                instruction: InternalExecOnMemoryData(ldy),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "LDY",
                opcode: 0xBC,
                instruction: InternalExecOnMemoryData(ldy),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 4,
            },

            Instruction {
                name: "STA",
                opcode: 0x85,
                instruction: StoreOp(sta),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "STA",
                opcode: 0x95,
                instruction: StoreOp(sta),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            Instruction {
                name: "STA",
                opcode: 0x8D,
                instruction: StoreOp(sta),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "STA",
                opcode: 0x9D,
                instruction: StoreOp(sta),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 5,
            },
            Instruction {
                name: "STA",
                opcode: 0x99,
                instruction: StoreOp(sta),
                addressing_mode: AbsoluteY,
                bytes: 3,
                cycles: 5,
            },
            Instruction {
                name: "STA",
                opcode: 0x81,
                instruction: StoreOp(sta),
                addressing_mode: IndirectX,
                bytes: 2,
                cycles: 6,
            },
            Instruction {
                name: "STA",
                opcode: 0x91,
                instruction: StoreOp(sta),
                addressing_mode: IndirectY,
                bytes: 2,
                cycles: 6,
            },

            Instruction {
                name: "STX",
                opcode: 0x86,
                instruction: StoreOp(stx),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "STX",
                opcode: 0x96,
                instruction: StoreOp(stx),
                addressing_mode: ZeroPageY,
                bytes: 2,
                cycles: 4,
            },
            Instruction {
                name: "STX",
                opcode: 0x8E,
                instruction: StoreOp(stx),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },

            Instruction {
                name: "STY",
                opcode: 0x84,
                instruction: StoreOp(sty),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "STY",
                opcode: 0x94,
                instruction: StoreOp(sty),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            Instruction {
                name: "STY",
                opcode: 0x8C,
                instruction: StoreOp(sty),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },

            Instruction {
                name: "TAX",
                opcode: 0xAA,
                instruction: SingleByte(tax),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },

            Instruction {
                name: "TAY",
                opcode: 0xA8,
                instruction: SingleByte(tay),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },

            Instruction {
                name: "TSX",
                opcode: 0xBA,
                instruction: SingleByte(tsx),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },

            Instruction {
                name: "TXA",
                opcode: 0x8A,
                instruction: SingleByte(txa),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },

            Instruction {
                name: "TXS",
                opcode: 0x9A,
                instruction: SingleByte(txs),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },

            Instruction {
                name: "TYA",
                opcode: 0x98,
                instruction: SingleByte(tya),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },

            // // Stack instructions
            Instruction {
                name: "PHA",
                opcode: 0x48,
                instruction: Misc(Push(pha)),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 3,
            },

            Instruction {
                name: "PHP",
                opcode: 0x08,
                instruction: Misc(Push(php)),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 3,
            },

            Instruction {
                name: "PLA",
                opcode: 0x68,
                instruction: Misc(Pull(pla)),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 4,
            },

            Instruction {
                name: "PLP",
                opcode: 0x28,
                instruction: Misc(Pull(plp)),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 4,
            },

            // Decrements and increments
            Instruction {
                name: "DEC",
                opcode: 0xC6,
                instruction: ReadModifyWrite(dec),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 5,
            },
            Instruction {
                name: "DEC",
                opcode: 0xD6,
                instruction: ReadModifyWrite(dec),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 6,
            },
            Instruction {
                name: "DEC",
                opcode: 0xCE,
                instruction: ReadModifyWrite(dec),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 6,
            },
            Instruction {
                name: "DEC",
                opcode: 0xDE,
                instruction: ReadModifyWrite(dec),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 7,
            },

            Instruction {
                name: "DEX",
                opcode: 0xCA,
                instruction: SingleByte(dex),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },

            Instruction {
                name: "DEY",
                opcode: 0x88,
                instruction: SingleByte(dey),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },

            Instruction {
                name: "INC",
                opcode: 0xE6,
                instruction: ReadModifyWrite(inc),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 5,
            },
            Instruction {
                name: "INC",
                opcode: 0xF6,
                instruction: ReadModifyWrite(inc),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 6,
            },
            Instruction {
                name: "INC",
                opcode: 0xEE,
                instruction: ReadModifyWrite(inc),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 6,
            },
            Instruction {
                name: "INC",
                opcode: 0xFE,
                instruction: ReadModifyWrite(inc),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 7,
            },

            Instruction {
                name: "INX",
                opcode: 0xE8,
                instruction: SingleByte(inx),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },

            Instruction {
                name: "INY",
                opcode: 0xC8,
                instruction: SingleByte(iny),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },

            // Arithmetic operations
            Instruction {
                name: "ADC",
                opcode: 0x69,
                instruction: InternalExecOnMemoryData(adc),
                addressing_mode: Immediate,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "ADC",
                opcode: 0x65,
                instruction: InternalExecOnMemoryData(adc),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "ADC",
                opcode: 0x75,
                instruction: InternalExecOnMemoryData(adc),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            Instruction {
                name: "ADC",
                opcode: 0x6D,
                instruction: InternalExecOnMemoryData(adc),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "ADC",
                opcode: 0x7D,
                instruction: InternalExecOnMemoryData(adc),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "ADC",
                opcode: 0x79,
                instruction: InternalExecOnMemoryData(adc),
                addressing_mode: AbsoluteY,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "ADC",
                opcode: 0x61,
                instruction: InternalExecOnMemoryData(adc),
                addressing_mode: IndirectX,
                bytes: 2,
                cycles: 6,
            },
            Instruction {
                name: "ADC",
                opcode: 0x71,
                instruction: InternalExecOnMemoryData(adc),
                addressing_mode: IndirectY,
                bytes: 2,
                cycles: 5,
            },

            Instruction {
                name: "SBC",
                opcode: 0xE9,
                instruction: InternalExecOnMemoryData(sbc),
                addressing_mode: Immediate,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "SBC",
                opcode: 0xE5,
                instruction: InternalExecOnMemoryData(sbc),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "SBC",
                opcode: 0xF5,
                instruction: InternalExecOnMemoryData(sbc),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            Instruction {
                name: "SBC",
                opcode: 0xED,
                instruction: InternalExecOnMemoryData(sbc),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "SBC",
                opcode: 0xFD,
                instruction: InternalExecOnMemoryData(sbc),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "SBC",
                opcode: 0xF9,
                instruction: InternalExecOnMemoryData(sbc),
                addressing_mode: AbsoluteY,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "SBC",
                opcode: 0xE1,
                instruction: InternalExecOnMemoryData(sbc),
                addressing_mode: IndirectX,
                bytes: 2,
                cycles: 6,
            },
            Instruction {
                name: "SBC",
                opcode: 0xF1,
                instruction: InternalExecOnMemoryData(sbc),
                addressing_mode: IndirectY,
                bytes: 2,
                cycles: 5,
            },

            // Logical operations
            Instruction {
                name: "AND",
                opcode: 0x29,
                instruction: InternalExecOnMemoryData(and),
                addressing_mode: Immediate,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "AND",
                opcode: 0x25,
                instruction: InternalExecOnMemoryData(and),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "AND",
                opcode: 0x35,
                instruction: InternalExecOnMemoryData(and),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            Instruction {
                name: "AND",
                opcode: 0x2D,
                instruction: InternalExecOnMemoryData(and),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "AND",
                opcode: 0x3D,
                instruction: InternalExecOnMemoryData(and),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "AND",
                opcode: 0x39,
                instruction: InternalExecOnMemoryData(and),
                addressing_mode: AbsoluteY,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "AND",
                opcode: 0x21,
                instruction: InternalExecOnMemoryData(and),
                addressing_mode: IndirectX,
                bytes: 2,
                cycles: 6,
            },
            Instruction {
                name: "AND",
                opcode: 0x31,
                instruction: InternalExecOnMemoryData(and),
                addressing_mode: IndirectY,
                bytes: 2,
                cycles: 5,
            },

            Instruction {
                name: "EOR",
                opcode: 0x49,
                instruction: InternalExecOnMemoryData(eor),
                addressing_mode: Immediate,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "EOR",
                opcode: 0x45,
                instruction: InternalExecOnMemoryData(eor),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "EOR",
                opcode: 0x55,
                instruction: InternalExecOnMemoryData(eor),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            Instruction {
                name: "EOR",
                opcode: 0x4D,
                instruction: InternalExecOnMemoryData(eor),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "EOR",
                opcode: 0x5D,
                instruction: InternalExecOnMemoryData(eor),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "EOR",
                opcode: 0x59,
                instruction: InternalExecOnMemoryData(eor),
                addressing_mode: AbsoluteY,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "EOR",
                opcode: 0x41,
                instruction: InternalExecOnMemoryData(eor),
                addressing_mode: IndirectX,
                bytes: 2,
                cycles: 6,
            },
            Instruction {
                name: "EOR",
                opcode: 0x51,
                instruction: InternalExecOnMemoryData(eor),
                addressing_mode: IndirectY,
                bytes: 2,
                cycles: 5,
            },

            Instruction {
                name: "ORA",
                opcode: 0x09,
                instruction: InternalExecOnMemoryData(ora),
                addressing_mode: Immediate,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "ORA",
                opcode: 0x05,
                instruction: InternalExecOnMemoryData(ora),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "ORA",
                opcode: 0x15,
                instruction: InternalExecOnMemoryData(ora),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            Instruction {
                name: "ORA",
                opcode: 0x0D,
                instruction: InternalExecOnMemoryData(ora),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "ORA",
                opcode: 0x1D,
                instruction: InternalExecOnMemoryData(ora),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "ORA",
                opcode: 0x19,
                instruction: InternalExecOnMemoryData(ora),
                addressing_mode: AbsoluteY,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "ORA",
                opcode: 0x01,
                instruction: InternalExecOnMemoryData(ora),
                addressing_mode: IndirectX,
                bytes: 2,
                cycles: 6,
            },
            Instruction {
                name: "ORA",
                opcode: 0x11,
                instruction: InternalExecOnMemoryData(ora),
                addressing_mode: IndirectY,
                bytes: 2,
                cycles: 5,
            },

            // Shift and rotation instructions
            Instruction {
                name: "ASL",
                opcode: 0x0A,
                instruction: SingleByte(asl_acc),
                addressing_mode: Accumulator,
                bytes: 1,
                cycles: 2,
            },
            Instruction {
                name: "ASL",
                opcode: 0x06,
                instruction: ReadModifyWrite(asl),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 5,
            },
            Instruction {
                name: "ASL",
                opcode: 0x16,
                instruction: ReadModifyWrite(asl),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 6,
            },
            Instruction {
                name: "ASL",
                opcode: 0x0E,
                instruction: ReadModifyWrite(asl),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 6,
            },
            Instruction {
                name: "ASL",
                opcode: 0x1E,
                instruction: ReadModifyWrite(asl),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 7,
            },

            Instruction {
                name: "LSR",
                opcode: 0x4A,
                instruction: SingleByte(lsr_acc),
                addressing_mode: Accumulator,
                bytes: 1,
                cycles: 2,
            },
            Instruction {
                name: "LSR",
                opcode: 0x46,
                instruction: ReadModifyWrite(lsr),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 5,
            },
            Instruction {
                name: "LSR",
                opcode: 0x56,
                instruction: ReadModifyWrite(lsr),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 6,
            },
            Instruction {
                name: "LSR",
                opcode: 0x4E,
                instruction: ReadModifyWrite(lsr),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 6,
            },
            Instruction {
                name: "LSR",
                opcode: 0x5E,
                instruction: ReadModifyWrite(lsr),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 7,
            },

            Instruction {
                name: "ROL",
                opcode: 0x2A,
                instruction: SingleByte(rol_acc),
                addressing_mode: Accumulator,
                bytes: 1,
                cycles: 2,
            },
            Instruction {
                name: "ROL",
                opcode: 0x26,
                instruction: ReadModifyWrite(rol),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 5,
            },
            Instruction {
                name: "ROL",
                opcode: 0x36,
                instruction: ReadModifyWrite(rol),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 6,
            },
            Instruction {
                name: "ROL",
                opcode: 0x2E,
                instruction: ReadModifyWrite(rol),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 6,
            },
            Instruction {
                name: "ROL",
                opcode: 0x3E,
                instruction: ReadModifyWrite(rol),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 7,
            },

            Instruction {
                name: "ROR",
                opcode: 0x6A,
                instruction: SingleByte(ror_acc),
                addressing_mode: Accumulator,
                bytes: 1,
                cycles: 2,
            },
            Instruction {
                name: "ROR",
                opcode: 0x66,
                instruction: ReadModifyWrite(ror),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 5,
            },
            Instruction {
                name: "ROR",
                opcode: 0x76,
                instruction: ReadModifyWrite(ror),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 6,
            },
            Instruction {
                name: "ROR",
                opcode: 0x6E,
                instruction: ReadModifyWrite(ror),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 6,
            },
            Instruction {
                name: "ROR",
                opcode: 0x7E,
                instruction: ReadModifyWrite(ror),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 7,
            },

            // Flag instructions
            Instruction {
                name: "CLC",
                opcode: 0x18,
                instruction: SingleByte(clc),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },
            Instruction {
                name: "CLD",
                opcode: 0xD8,
                instruction: SingleByte(cld),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },
            Instruction {
                name: "CLI",
                opcode: 0x58,
                instruction: SingleByte(cli),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },
            Instruction {
                name: "CLV",
                opcode: 0xB8,
                instruction: SingleByte(clv),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },
            Instruction {
                name: "SEC",
                opcode: 0x38,
                instruction: SingleByte(sec),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },
            Instruction {
                name: "SED",
                opcode: 0xF8,
                instruction: SingleByte(sed),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },
            Instruction {
                name: "SEI",
                opcode: 0x78,
                instruction: SingleByte(sei),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },

            // Comparaisons
            Instruction {
                name: "CMP",
                opcode: 0xC9,
                instruction: InternalExecOnMemoryData(cmp),
                addressing_mode: Immediate,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "CMP",
                opcode: 0xC5,
                instruction: InternalExecOnMemoryData(cmp),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "CMP",
                opcode: 0xD5,
                instruction: InternalExecOnMemoryData(cmp),
                addressing_mode: ZeroPageX,
                bytes: 2,
                cycles: 4,
            },
            Instruction {
                name: "CMP",
                opcode: 0xCD,
                instruction: InternalExecOnMemoryData(cmp),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "CMP",
                opcode: 0xDD,
                instruction: InternalExecOnMemoryData(cmp),
                addressing_mode: AbsoluteX,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "CMP",
                opcode: 0xD9,
                instruction: InternalExecOnMemoryData(cmp),
                addressing_mode: AbsoluteY,
                bytes: 3,
                cycles: 4,
            },
            Instruction {
                name: "CMP",
                opcode: 0xC1,
                instruction: InternalExecOnMemoryData(cmp),
                addressing_mode: IndirectX,
                bytes: 2,
                cycles: 6,
            },
            Instruction {
                name: "CMP",
                opcode: 0xD1,
                instruction: InternalExecOnMemoryData(cmp),
                addressing_mode: IndirectY,
                bytes: 2,
                cycles: 5,
            },

            Instruction {
                name: "CPX",
                opcode: 0xE0,
                instruction: InternalExecOnMemoryData(cpx),
                addressing_mode: Immediate,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "CPX",
                opcode: 0xE4,
                instruction: InternalExecOnMemoryData(cpx),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "CPX",
                opcode: 0xEC,
                instruction: InternalExecOnMemoryData(cpx),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },

            Instruction {
                name: "CPY",
                opcode: 0xC0,
                instruction: InternalExecOnMemoryData(cpy),
                addressing_mode: Immediate,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "CPY",
                opcode: 0xC4,
                instruction: InternalExecOnMemoryData(cpy),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "CPY",
                opcode: 0xCC,
                instruction: InternalExecOnMemoryData(cpy),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },

            // Conditional branch instructions
            Instruction {
                name: "BCC",
                opcode: 0x90,
                instruction: Misc(Branch(bcc)),
                addressing_mode: Relative,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "BCS",
                opcode: 0xB0,
                instruction: Misc(Branch(bcs)),
                addressing_mode: Relative,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "BEQ",
                opcode: 0xF0,
                instruction: Misc(Branch(beq)),
                addressing_mode: Relative,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "BMI",
                opcode: 0x30,
                instruction: Misc(Branch(bmi)),
                addressing_mode: Relative,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "BNE",
                opcode: 0xD0,
                instruction: Misc(Branch(bne)),
                addressing_mode: Relative,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "BPL",
                opcode: 0x10,
                instruction: Misc(Branch(bpl)),
                addressing_mode: Relative,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "BVC",
                opcode: 0x50,
                instruction: Misc(Branch(bvc)),
                addressing_mode: Relative,
                bytes: 2,
                cycles: 2,
            },
            Instruction {
                name: "BVS",
                opcode: 0x70,
                instruction: Misc(Branch(bvs)),
                addressing_mode: Relative,
                bytes: 2,
                cycles: 2,
            },

            // Jumps and subroutines
            Instruction {
                name: "JMP",
                opcode: 0x4C,
                instruction: Misc(Jump(jmp)),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 3,
            },
            Instruction {
                name: "JMP",
                opcode: 0x6C,
                instruction: Misc(Jump(jmp)),
                addressing_mode: Indirect,
                bytes: 3,
                cycles: 5,
            },

            Instruction {
                name: "JSR",
                opcode: 0x20,
                instruction: Misc(Call(jsr)),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 6,
            },

            Instruction {
                name: "RTI",
                opcode: 0x40,
                instruction: Misc(ReturnFromInterrupt(rti)),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 6,
            },

            // Interrupts
            Instruction {
                name: "BRK",
                opcode: 0x00,
                instruction: Misc(HardwareInterrupt(brk)),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 7,
            },

            Instruction {
                name: "RTS",
                opcode: 0x60,
                instruction: Misc(Return(rts)),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 6,
            },

            // Other
            Instruction {
                name: "BIT",
                opcode: 0x24,
                instruction: InternalExecOnMemoryData(bit),
                addressing_mode: ZeroPage,
                bytes: 2,
                cycles: 3,
            },
            Instruction {
                name: "BIT",
                opcode: 0x2C,
                instruction: InternalExecOnMemoryData(bit),
                addressing_mode: Absolute,
                bytes: 3,
                cycles: 4,
            },

            Instruction {
                name: "NOP",
                opcode: 0xEA,
                instruction: SingleByte(nop),
                addressing_mode: Implied,
                bytes: 1,
                cycles: 2,
            },
        ];

        for instruction in instructions {
            instruction_set.insert(instruction.opcode, instruction);
        }

        Self { instruction_set }
    }

    pub fn lookup(&self, opcode: Opcode) -> Option<Instruction> {
        self.instruction_set.get(&opcode).cloned()
    }
}

// Instruction Set
// ---------------

// Transfer instructions

/// LDA - Load Accumulator with Memory
///
/// Operation:
/// M -> A
///
/// Status Register
/// N Z C I D V
/// + + - - - -
pub fn lda(cpu: &mut InternalCpu, operand: u8) {
    cpu.acc = operand;
    cpu.sr.auto_set(Negative, cpu.acc);
    cpu.sr.auto_set(Zero, cpu.acc);
}

/// LDX - Load Index X with Memory
///
/// Operation:
/// M -> X
///
/// Status Register
/// N Z C I D V
/// + + - - - -
pub fn ldx(cpu: &mut InternalCpu, operand: u8) {
    cpu.x_reg = operand;
    cpu.sr.auto_set(Negative, cpu.x_reg);
    cpu.sr.auto_set(Zero, cpu.x_reg);
}

/// LDY - Load Index Y with Memory
///
/// Operation:
/// M -> Y
///
/// Status Register
/// N Z C I D V
/// + + - - - -
pub fn ldy(cpu: &mut InternalCpu, operand: u8) {
    cpu.y_reg = operand;
    cpu.sr.auto_set(Negative, cpu.y_reg);
    cpu.sr.auto_set(Zero, cpu.y_reg);
}

/// STA - Store Accumulator in Memory
///
/// Operation:
/// A -> M
///
/// Status Register
/// N Z C I D V
/// - - - - - -
pub fn sta(cpu: &mut InternalCpu) -> u8 {
    cpu.acc
}

/// STX - Store Index X in Memory
///
/// Operation:
/// X -> M
///
/// Status Register
/// N Z C I D V
/// - - - - - -
pub fn stx(cpu: &mut InternalCpu) -> u8 {
    cpu.x_reg
}

/// STY - Store Index Y in Memory
/// Operation:
/// Y -> M
///
/// Status Register
/// N Z C I D V
/// - - - - - -
pub fn sty(cpu: &mut InternalCpu) -> u8 {
    cpu.y_reg
}

/// TAX - Transfer Accumulator to Index X
///
/// Operation:
/// A -> X
///
/// Status Register:
/// N Z C I D V
/// + + - - - -
pub fn tax(cpu: &mut InternalCpu) {
    cpu.x_reg = cpu.acc;
    cpu.sr.auto_set(Negative, cpu.x_reg);
    cpu.sr.auto_set(Zero, cpu.x_reg);
}

/// TAY - Transfer Accumulator to Index Y
///
/// Operation:
/// A -> Y
///
/// Status Register:
/// N Z C I D V
/// + + - - - -
pub fn tay(cpu: &mut InternalCpu) {
    cpu.y_reg = cpu.acc;
    cpu.sr.auto_set(Negative, cpu.y_reg);
    cpu.sr.auto_set(Zero, cpu.y_reg);
}

/// TSX - Transfer Stack Pointer to Index X
///
/// Operation:
/// SP -> X
///
/// Status Register:
/// N Z C I D V
/// + + - - - -
pub fn tsx(cpu: &mut InternalCpu) {
    cpu.x_reg = cpu.sp;
    cpu.sr.auto_set(Negative, cpu.x_reg);
    cpu.sr.auto_set(Zero, cpu.x_reg);
}

/// TXA - Transfer Index X to Accumulator
///
/// Operation:
/// X -> A
///
/// Status Register:
/// N Z C I D V
/// + + - - - -
pub fn txa(cpu: &mut InternalCpu) {
    cpu.acc = cpu.x_reg;
    cpu.sr.auto_set(Negative, cpu.acc);
    cpu.sr.auto_set(Zero, cpu.acc);
}

/// TXS - Transfer Index X to Stack Pointer
///
/// Operation:
/// X -> SP
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn txs(cpu: &mut InternalCpu) {
    cpu.sp = cpu.x_reg;
}

/// TYA - Transfer Index Y to Accumulator
///
/// Operation:
/// Y -> A
///
/// Status Register:
/// N Z C I D V
/// + + - - - -
pub fn tya(cpu: &mut InternalCpu) {
    cpu.acc = cpu.y_reg;
    cpu.sr.auto_set(Negative, cpu.acc);
    cpu.sr.auto_set(Zero, cpu.acc);
}

// Stack instructions

pub fn push(cpu: &mut InternalCpu, data: u8, memory: &SharedBus) {
    let address = 0x0100 + (cpu.sp as u16);
    println!("Push to SP 0x{:X} - 0x{:X}", cpu.sp, data);
    memory.borrow_mut().write(address, data);
    cpu.sp -= 1;
}

pub fn pull(cpu: &mut InternalCpu, memory: &SharedBus) -> u8 {
    cpu.sp += 1;
    let address = 0x0100 + (cpu.sp as u16);
    let data = memory.borrow().read(address);
    println!("Pull from SP 0x{:X} - 0x{:X}", cpu.sp, data);
    data
}

/// PHA - Push Accumulator on Stack
///
/// Operation:
/// push A
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn pha(cpu: &mut InternalCpu, memory: &SharedBus) {
    push(cpu, cpu.acc, memory);
}

/// PHP - Push Processor Status on Stack
///
/// The status register will be pushed with the break flag and bit
/// 5 set to 1.
///
/// Operation:
/// push SR
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn php(cpu: &mut InternalCpu, memory: &SharedBus) {
    let sr: u8 = cpu.sr.into();
    push(cpu, sr | (1 << Break as u8) | (1 << 5), memory);
}

/// PLA - Pull Accumulator from Stack
///
/// Operation:
/// pull A
///
/// Status Register
/// N Z C I D V
/// + + - - - -
pub fn pla(cpu: &mut InternalCpu, memory: &SharedBus) {
    cpu.acc = pull(cpu, memory);
    cpu.sr.auto_set(Negative, cpu.acc);
    cpu.sr.auto_set(Zero, cpu.acc);
}

/// PLP - Pull Processor Status from Stack
///
/// The status register will be pulled with the break flag and bit
/// 5 ignored.
///
/// Operation:
/// pull A
///
/// Status Register
/// N Z C I D V
/// + + - - - -
pub fn plp(cpu: &mut InternalCpu, memory: &SharedBus) {
    let mut sr = StatusRegister::from(pull(cpu, memory));
    sr.set_value(Break, cpu.sr.get(Break));
    // XXX bit 5 is ignored, as NES don't use it
    cpu.sr = sr
}

// Decrements and increments

/// DEC - Decrment Memory by One
///
/// Operation:
/// M - 1 -> M
///
/// Status Register
/// N Z C I D V
/// + + - - - -
pub fn dec(cpu: &mut InternalCpu, operand: u8) -> u8 {
    let (res, _) = operand.overflowing_sub(1);
    cpu.sr.auto_set(Negative, res);
    cpu.sr.auto_set(Zero, res);
    res
}

/// DEX - Decrment Index X by One
///
/// Operation:
/// X - 1 -> X
///
/// Status Register
/// N Z C I D V
/// + + - - - -
pub fn dex(cpu: &mut InternalCpu) {
    let (res, _) = cpu.x_reg.overflowing_sub(1);
    cpu.x_reg = res;
    cpu.sr.auto_set(Negative, cpu.x_reg);
    cpu.sr.auto_set(Zero, cpu.x_reg);
}

/// DEY - Decrment Index Y by One
///
/// Operation:
/// Y - 1 -> Y
///
/// Status Register
/// N Z C I D V
/// + + - - - -
pub fn dey(cpu: &mut InternalCpu) {
    let (res, _) = cpu.y_reg.overflowing_sub(1);
    cpu.y_reg = res;
    cpu.sr.auto_set(Negative, cpu.y_reg);
    cpu.sr.auto_set(Zero, cpu.y_reg);
}

/// INC - Increment Memory by One
///
/// Operation:
/// M + 1 -> M
///
/// Status Register
/// N Z C I D V
/// + + - - - -
pub fn inc(cpu: &mut InternalCpu, operand: u8) -> u8 {
    let (res, _) = operand.overflowing_add(1);
    cpu.sr.auto_set(Negative, res);
    cpu.sr.auto_set(Zero, res);
    res
}

/// INX - Incrment Index X by One
///
/// Operation:
/// X + 1 -> X
///
/// Status Register
/// N Z C I D V
/// + + - - - -
pub fn inx(cpu: &mut InternalCpu) {
    let (res, _) = cpu.x_reg.overflowing_add(1);
    cpu.x_reg = res;
    cpu.sr.auto_set(Negative, cpu.x_reg);
    cpu.sr.auto_set(Zero, cpu.x_reg);
}

/// INY - Incrment Index Y by One
///
/// Operation:
/// Y + 1 -> Y
///
/// Status Register
/// N Z C I D V
/// + + - - - -
pub fn iny(cpu: &mut InternalCpu) {
    let (res, _) = cpu.y_reg.overflowing_add(1);
    cpu.y_reg = res;
    cpu.sr.auto_set(Negative, cpu.y_reg);
    cpu.sr.auto_set(Zero, cpu.y_reg);
}

// Arithmetic operations

/// ADC - Add Memory to Accumulator with Carry
///
/// Operation:
/// A + M + C -> A, C
///
/// Status Register:
/// N Z C I D V
/// + + + - - +
pub fn adc(cpu: &mut InternalCpu, operand: u8) {
    let carry = if cpu.sr.get(Carry) { 1 } else { 0 };
    let res = cpu.acc as u16 + operand as u16 + carry;
    let carry = (res & (1 << 8)) != 0;
    let res = res as u8;
    let overflow = utils::bv(cpu.acc, 7) == utils::bv(operand, 7)
        && utils::bv(operand, 7) != utils::bv(res, 7);

    cpu.acc = res;
    cpu.sr.auto_set(Negative, cpu.acc);
    cpu.sr.auto_set(Zero, cpu.acc);
    cpu.sr.set_value(Carry, carry);
    cpu.sr.set_value(Overflow, overflow);
}

/// SBC - Substract Memory from Accumulator with Borrow
///
/// Operation:
/// A - M - (1 - C) -> A
///
/// Status Register:
/// N Z C I D V
/// + + + - - +
pub fn sbc(cpu: &mut InternalCpu, operand: u8) {
    adc(cpu, !operand);
    // let carry = if cpu.sr.get(Carry) { 1 } else { 0 };

    // let res = cpu.acc as u16 + (operand ^ 0xFF) as u16 + carry;
    // let overflow = utils::bv(cpu.acc, 7) == utils::bv(operand, 7)
    //     && utils::bv(operand, 7) != utils::bv(res as u8, 7);
    // let carry = if overflow {
    //     false
    // } else {
    //     true
    // };

    // let res = res as u8;

    // cpu.acc = res;
    // cpu.sr.auto_set(Negative, cpu.acc);
    // cpu.sr.auto_set(Zero, cpu.acc);
    // cpu.sr.set_value(Carry, carry);
    // cpu.sr.set_value(Overflow, overflow);
}

// Logic operations

/// AND - AND Memory with Accumulator
///
/// Operation:
/// A AND M -> A
///
/// Status Register:
/// N Z C I D V
/// + + - - - -
pub fn and(cpu: &mut InternalCpu, operand: u8) {
    cpu.acc &= operand;
    cpu.sr.auto_set(Negative, cpu.acc);
    cpu.sr.auto_set(Zero, cpu.acc);
}

/// EOR - Exclusive-OR Memory with Accumulator
///
/// Operation:
/// A EOR M -> A
///
/// Status Register:
/// N Z C I D V
/// + + - - - -
pub fn eor(cpu: &mut InternalCpu, operand: u8) {
    cpu.acc ^= operand;
    cpu.sr.auto_set(Negative, cpu.acc);
    cpu.sr.auto_set(Zero, cpu.acc);
}

/// ORA - OR Memory with Accumulator
///
/// Operation:
/// A OR M -> A
///
/// Status Register:
/// N Z C I D V
/// + + - - - -
pub fn ora(cpu: &mut InternalCpu, operand: u8) {
    cpu.acc |= operand;
    cpu.sr.auto_set(Negative, cpu.acc);
    cpu.sr.auto_set(Zero, cpu.acc);
}

// Shift & Rotate instructions

/// ASL - Shift Left One Bit (Memory or Accumulator)
///
/// Operation:
/// C <- [76543210] <- 0
///
/// Status Register:
/// N Z C I D V
/// + + + - - -
pub fn asl_acc(cpu: &mut InternalCpu) {
    cpu.acc = asl(cpu, cpu.acc);
}
pub fn asl(cpu: &mut InternalCpu, operand: u8) -> u8 {
    let result = operand << 1;
    let carry = utils::bv(operand, 7) != 0;
    cpu.sr.auto_set(Negative, result);
    cpu.sr.auto_set(Zero, result);
    cpu.sr.set_value(Carry, carry);
    result
}

/// LSR - Shift One Bit Right (Memory or Accumulator)
///
/// Operation:
/// 0 -> [76543210] -> C
///
/// Status Register:
/// N Z C I D V
/// 0 + + - - -
pub fn lsr_acc(cpu: &mut InternalCpu) {
    cpu.acc = lsr(cpu, cpu.acc);
}

pub fn lsr(cpu: &mut InternalCpu, operand: u8) -> u8 {
    let result = operand >> 1;
    let carry = utils::bv(operand, 0) != 0;
    cpu.sr.clear(Negative);
    cpu.sr.auto_set(Zero, result);
    cpu.sr.set_value(Carry, carry);
    result
}

/// ROL - Rotate One Bit Left (Memory or Accumulator)
///
/// Operation:
/// C <- [76543210] <- C
///
/// Status Register:
/// N Z C I D V
/// + + + - - -
pub fn rol_acc(cpu: &mut InternalCpu) {
    cpu.acc = rol(cpu, cpu.acc);
}

pub fn rol(cpu: &mut InternalCpu, operand: u8) -> u8 {
    let new_carry = utils::bv(operand, 7) != 0;
    let curr_carry = if cpu.sr.get(Carry) { 1 } else { 0 };
    let result = operand << 1 | curr_carry;
    cpu.sr.auto_set(Negative, result);
    cpu.sr.auto_set(Zero, result);
    cpu.sr.set_value(Carry, new_carry);
    result
}

/// ROR - Rotate One Bit Right (Memory or Accumulator)
///
/// Operation:
/// C -> [76543210] -> C
///
/// Status Register:
/// N Z C I D V
/// + + + - - -
pub fn ror_acc(cpu: &mut InternalCpu) {
    cpu.acc = ror(cpu, cpu.acc);
}

pub fn ror(cpu: &mut InternalCpu, operand: u8) -> u8 {
    let new_carry = utils::bv(operand, 0) != 0;
    let curr_carry = if cpu.sr.get(Carry) { 1 } else { 0 };
    let result = operand >> 1 | (curr_carry << 7);
    cpu.sr.auto_set(Negative, result);
    cpu.sr.auto_set(Zero, result);
    cpu.sr.set_value(Carry, new_carry);
    result
}

// Flag instructions

/// CLC - Clear Carry Flag
///
/// Operation:
/// 0 -> C
///
/// Status Register:
/// N Z C I D V
/// - - 0 - - -
pub fn clc(cpu: &mut InternalCpu) {
    cpu.sr.clear(Carry);
}

/// CLD - Clear Decimal Mode
///
/// Operation:
/// 0 -> D
///
/// Status Register:
/// N Z C I D V
/// - - - - 0 -
pub fn cld(cpu: &mut InternalCpu) {
    cpu.sr.clear(Decimal);
}

/// CLI - Clear Interrupt Disable Bit
///
/// Operation:
/// 0 -> I
///
/// Status Register:
/// N Z C I D V
/// - - - 0 - -
pub fn cli(cpu: &mut InternalCpu) {
    cpu.sr.clear(InterruptDisable);
}

/// CLV - Clear Overflow Flag
///
/// Operation:
/// 0 -> V
///
/// Status Register:
/// N Z C I D V
/// - - - - - 0
pub fn clv(cpu: &mut InternalCpu) {
    cpu.sr.clear(Overflow);
}

/// SEC - Set Carry Flag
///
/// Operation:
/// 1 -> C
///
/// Status Register:
/// N Z C I D V
/// - - 1 - - -
pub fn sec(cpu: &mut InternalCpu) {
    cpu.sr.set(Carry);
}

/// SED - Set Decimal Flag
///
/// Operation:
/// 1 -> D
///
/// Status Register:
/// N Z C I D V
/// - - - - 1 -
pub fn sed(cpu: &mut InternalCpu) {
    cpu.sr.set(Decimal);
}

/// SEI - Set Interrupt Disable Status
///
/// Operation:
/// 1 -> I
///
/// Status Register:
/// N Z C I D V
/// - - - 1 - -
pub fn sei(cpu: &mut InternalCpu) {
    cpu.sr.set(InterruptDisable);
}

// Comparaisons

pub fn generic_cmp(cpu: &mut InternalCpu, a: u8, b: u8) {
    let (res, _) = a.overflowing_sub(b);
    cpu.sr.auto_set(Negative, res);
    cpu.sr.auto_set(Zero, res);
    cpu.sr.set_value(Carry, a >= b);
}

/// CMP - Compare Memory with Accumulator
///
/// Operation:
/// A - M
///
/// Status Register:
/// N Z C I D V
/// + + + - - -
pub fn cmp(cpu: &mut InternalCpu, operand: u8) {
    generic_cmp(cpu, cpu.acc, operand);
}

/// CPX - Compare Memory and Index X
///
/// Operation:
/// X - M
///
/// Status Register:
/// N Z C I D V
/// + + + - - -
pub fn cpx(cpu: &mut InternalCpu, operand: u8) {
    generic_cmp(cpu, cpu.x_reg, operand);
}

/// CPY - Compare Memory and Index Y
///
/// Operation:
/// Y - M
///
/// Status Register:
/// N Z C I D V
/// + + + - - -
pub fn cpy(cpu: &mut InternalCpu, operand: u8) {
    generic_cmp(cpu, cpu.y_reg, operand);
}

// Conditional branch

pub fn branch(cpu: &mut InternalCpu, condition: bool, offset: u8) {
    if condition {
        // TODO add +1 if page changes
        let (pc, _) = cpu.pc.overflowing_add_signed(offset as i8 as i16);
        cpu.pc = pc;
    }
}

/// BCC - Branch on Carry Clear
///
/// Operation:
/// branch on C = 0
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn bcc(cpu: &mut InternalCpu, offset: u8) {
    branch(cpu, !cpu.sr.get(Carry), offset);
}

/// BCS - Branch on Carry Set
///
/// Operation:
/// branch on C = 1
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn bcs(cpu: &mut InternalCpu, offset: u8) {
    branch(cpu, cpu.sr.get(Carry), offset);
}

/// BEQ - Branch on Result Zero
///
/// Operation:
/// branch on Z = 1
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn beq(cpu: &mut InternalCpu, offset: u8) {
    branch(cpu, cpu.sr.get(Zero), offset);
}

/// BMI - Branch on Result Minus
///
/// Operation:
/// branch on N = 1
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn bmi(cpu: &mut InternalCpu, offset: u8) {
    branch(cpu, cpu.sr.get(Negative), offset);
}

/// BNE - Branch on Result not Zero
///
/// Operation:
/// branch on Z = 0
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn bne(cpu: &mut InternalCpu, offset: u8) {
    branch(cpu, !cpu.sr.get(Zero), offset);
}

/// BPL - Branch on Result Plus
///
/// Operation:
/// branch on N = 0
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn bpl(cpu: &mut InternalCpu, offset: u8) {
    branch(cpu, !cpu.sr.get(Negative), offset);
}

/// BVC - Branch on Overflow Clear
///
/// Operation:
/// branch on V = 0
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn bvc(cpu: &mut InternalCpu, offset: u8) {
    branch(cpu, !cpu.sr.get(Overflow), offset);
}

/// BVS - Branch on Overflow Set
///
/// Operation:
/// branch on V = 1
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn bvs(cpu: &mut InternalCpu, offset: u8) {
    branch(cpu, cpu.sr.get(Overflow), offset);
}

// Jumps and subroutines

/// JMP - Jump to New Location
///
/// Operation:
/// (PC+1) -> PCL
/// (PC+2) -> PCH
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn jmp(cpu: &mut InternalCpu, address: u16) {
    cpu.pc = address;
}

/// JSR - Jump to New Location Saving Return Address
///
/// Operation:
/// push (PC+2)
/// (PC+1) -> PCL
/// (PC+2) -> PCH
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn jsr(cpu: &mut InternalCpu, address: u16, memory: &SharedBus) {
    let pc = cpu.pc + 2;
    let pch = (pc >> 8) as u8;
    let pcl = (pc & 0x00FF) as u8;
    push(cpu, pch, memory);
    push(cpu, pcl, memory);
    cpu.pc = address;
}

/// RTS - Return from subroutine
///
/// Operation:
/// pull PC, PC+1 -> PC
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn rts(cpu: &mut InternalCpu, memory: &SharedBus) {
    let pcl = pull(cpu, memory) as u16;
    let pch = pull(cpu, memory) as u16;
    cpu.pc = ((pch << 8) | pcl) + 1;
}

// Interrupts

/// BRK - Force Break
///
/// BRK initiates a software interrupt similar to a hardware
/// interrupt (IRQ). The return address pushed to the stack is
/// PC+2, providing an extra byte of spacing for a break mark
/// (identifying a reason for the break.)
///
/// The status register will be pushed to the stack with the break
/// flag set to 1. However, when retrieved during RTI or by a PLP
/// instruction, the break flag will be ignored.
///
/// The interrupt disable flag is not set automatically
///
/// Operation:
/// interrupt, push PC+2, push SR
///
/// Status Register:
/// N Z C I D V
/// - - - 1 - -
pub fn brk(cpu: &mut InternalCpu, memory: &SharedBus) {
    let return_address = cpu.pc + 2;
    let pch = (return_address >> 8) as u8;
    let pcl = (return_address & 0x00FF) as u8;
    push(cpu, pch, memory);
    push(cpu, pcl, memory);
    let current_sr: u8 = cpu.sr.into();
    let sr: u8 = current_sr | (1 << Break as u8);
    push(cpu, sr, memory);
    let adl = memory.borrow().read(0xFFFE) as u16;
    let adh = memory.borrow().read(0xFFFF) as u16;
    cpu.pc = (adh << 8) | adl;
    cpu.sr.set(InterruptDisable);
}

/// RTI - Return from Interrupt
///
/// The status register is pulled with the break flag and bit 5
/// ignored. Then PC is pulled from stack.
///
/// Operation:
/// pull SR, pull PC
///
/// Status Register:
///  N Z C I D V
///  from stack
pub fn rti(cpu: &mut InternalCpu, memory: &SharedBus) {
    let mut stack_sr = pull(cpu, memory);
    stack_sr &= !(1 << Break as u8);
    cpu.sr = StatusRegister::from(stack_sr);
    let pcl = pull(cpu, memory) as u16;
    let pch = pull(cpu, memory) as u16;
    cpu.pc = (pch << 8) | pcl;
}

// Other

/// BIT - Test Bits in Memory with Accumulator
///
/// bits 7 and 6 of operand are transfered to bit 7 and 6 of SR
/// (N,V); the zero-flag is set to the result of operand AND
/// accumulator.
///
/// Operation:
/// A AND M, M7 -> N, M6 -> V
///
/// Status Register:
///  N Z C I D V
/// M7 + - - - M6
pub fn bit(cpu: &mut InternalCpu, operand: u8) {
    cpu.sr.set_value(Negative, utils::bv(operand, 7) != 0);
    cpu.sr.set_value(Overflow, utils::bv(operand, 6) != 0);
    cpu.sr.auto_set(Zero, cpu.acc & operand);
}

/// NOP - No Operation
///
/// Operation:
/// ---
///
/// Status Register:
/// N Z C I D V
/// - - - - - -
pub fn nop(_: &mut InternalCpu) {}
