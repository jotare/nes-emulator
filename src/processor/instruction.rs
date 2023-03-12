use crate::processor::internal_cpu::InternalCpu;
use crate::types::SharedBus;

pub type Opcode = u8;

#[derive(Clone)]
pub struct Instruction {
    pub opcode: Opcode,
    pub name: &'static str,
    pub instruction: InstructionKind,
    pub addressing_mode: AddressingMode,
    pub bytes: u8,
    pub cycles: u8,
}

#[derive(Clone)]
pub enum InstructionKind {
    SingleByte(fn(&mut InternalCpu)),
    InternalExecOnMemoryData(fn(&mut InternalCpu, u8)),
    StoreOp(fn(&mut InternalCpu) -> u8),
    ReadModifyWrite(fn(&mut InternalCpu, u8) -> u8),
    Misc(MiscInstructionKind),
}

#[derive(Clone)]
pub enum MiscInstructionKind {
    Push(fn(&mut InternalCpu, &SharedBus)),
    Pull(fn(&mut InternalCpu, &SharedBus)),
    Jump(fn(&mut InternalCpu, u16)),
    Branch(fn(&mut InternalCpu, u8)),
    Call(fn(&mut InternalCpu, u16, &SharedBus)),
    Return(fn(&mut InternalCpu, &SharedBus)),
    HardwareInterrupt(fn(&mut InternalCpu, &SharedBus)),
    ReturnFromInterrupt(fn(&mut InternalCpu, &SharedBus)),
}

#[derive(Clone, Copy, Debug)]
pub enum AddressingMode {
    Implied,     // Implied Addressing
    Accumulator, // Accumulator Addressing
    Immediate,   // Immediate Addressing
    Absolute,    // Absoulute Addressing
    ZeroPage,    // Zero Page Addressing
    AbsoluteX,   // Absoulute Indexed Addressing (X)
    AbsoluteY,   // Absoulute Indexed Addressing (Y)
    ZeroPageX,   // Zero Page Indexed Addressing (X)
    ZeroPageY,   // Zero Page Indexed Addressing (Y)
    IndirectX,   // Zero Page Indexed Indirect Addressing (X)
    IndirectY,   // Zero Page Indexed Indirect Addressing (Y)
    Relative,    // Relative Addressing (branch operations)
    Indirect,    // Indirect Addressing (jump operations)
}
