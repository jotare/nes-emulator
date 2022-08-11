#[cfg(test)]
mod tests;

use super::bus::Bus;
use std::{collections::HashMap, rc::Rc};

/// MOS 6502 processor emulator.
///
/// CPU abstraction is connected to a `Memory` to perform read and
/// write operations on it.
///
/// This implementation uses the legal opcode instruction set. Illegal
/// instructions are not implemented.
pub struct Cpu {
    acc: u8,   // Accumulator
    x_reg: u8, // X register
    y_reg: u8, // Y register
    sp: u8,    // Stack Pointer
    pc: u16,   // Program Counter
    sr: u8,    // Status Register
    bus: Rc<dyn Bus>,
    instruction_set: HashMap<u8, Instruction>,
}

// Instruction addressing modes
enum AddressingMode {
    Implied,     // Implied Addressing
    Accum,       // Accumulator Addressing
    Immediate,   // Immediate Addressing
    Absolute,    // Absoulute Addressing
    ZeroPage,    // Zero Page Addressing
    AbsX,        // Absoulute Indexed Addressing (X)
    AbsY,        // Absoulute Indexed Addressing (Y)
    ZpgX,        // Zero Page Indexed Addressing (X)
    ZpgY,        // Zero Page Indexed Addressing (Y)
    Relative,    // Relative Addressing
    IndX,        // Zero Page Indexed Indirect Addressing (X)
    IndY,        // Zero Page Indexed Indirect Addressing (Y)
    AbsIndirect, // Absolute Indirect Addressing (Jump instructions only)
}
use AddressingMode::*;

#[derive(Clone)]
pub enum ExecutableInstruction {
    // transfer
    Load(fn(&mut Cpu, u8)),
    Transfer(fn(&mut Cpu)),
    // decrement
    Decrement(fn(&mut Cpu)),
    // increment
    Increment(fn(&mut Cpu)),
    // arithmetic
    Arithmetic
    // logical
    Logical(fn(&mut Cpu, u8)),
    // flag instructions
    Flag(fn(&mut Cpu)),
}
use ExecutableInstruction::*;

pub struct FetchedInstruction {
    instruction: ExecutableInstruction,
    cycles: u8,
    address: u16,
    data: Option<u8>,
}

/// An `Instruction` represents a single MOS 6502 instruction. It has
/// a name, an addressing mode, number of bytes and a function pointer
/// to execute it's corresponding CPU operation.
pub struct Instruction {
    name: &'static str,
    instruction: ExecutableInstruction,
    addressing: AddressingMode,
    cycles: u8,
}

macro_rules! zero_arg_instruction {
    ($name:expr, $instruction_type:expr, Cpu::$fun:ident, $addr_mode:expr, $cycles:expr) => {
        Instruction {
            name: $name,
            instruction: $instruction_type(|cpu| Cpu::$fun(cpu)),
            addressing: $addr_mode,
            cycles: $cycles,
        }
    };
}

macro_rules! instruction {
    ($name:expr, Transfer, Cpu::$fun:ident, $addr_mode:expr, $cycles:expr) => {
        zero_arg_instruction!($name, Transfer, Cpu::$fun, $addr_mode, $cycles)
    };
    ($name:expr, Decrement, Cpu::$fun:ident, $addr_mode:expr, $cycles:expr) => {
        zero_arg_instruction!($name, Decrement, Cpu::$fun, $addr_mode, $cycles)
    };
    ($name:expr, Increment, Cpu::$fun:ident, $addr_mode:expr, $cycles:expr) => {
        zero_arg_instruction!($name, Increment, Cpu::$fun, $addr_mode, $cycles)
    };
    ($name:expr, Flag, Cpu::$fun:ident, $addr_mode:expr, $cycles:expr) => {
        zero_arg_instruction!($name, Transfer, Cpu::$fun, $addr_mode, $cycles)
    };
    ($name:expr, $instruction_type:expr, Cpu::$fun:ident, $addr_mode:expr, $cycles:expr) => {
        Instruction {
            name: $name,
            instruction: $instruction_type(|cpu, operand| Cpu::$fun(cpu, operand)),
            addressing: $addr_mode,
            cycles: $cycles,
        }
    };
}

pub fn legal_opcode_instruction_set() -> HashMap<u8, Instruction> {
    let mut instruction_set = HashMap::new();

    // Transfer instructions
    instruction_set.insert(0xA9, instruction!("LDA", Load, Cpu::lda, Immediate, 2));
    instruction_set.insert(0xA2, instruction!("LDX", Load, Cpu::lda, Immediate, 2));
    instruction_set.insert(0xA0, instruction!("LDY", Load, Cpu::lda, Immediate, 2));

    instruction_set.insert(0xAA, instruction!("TAX", Transfer, Cpu::tax, Implied, 2));
    instruction_set.insert(0xA8, instruction!("TAY", Transfer, Cpu::tay, Implied, 2));
    instruction_set.insert(0xBA, instruction!("TSX", Transfer, Cpu::tsx, Implied, 2));
    instruction_set.insert(0x8A, instruction!("TXA", Transfer, Cpu::txa, Implied, 2));
    instruction_set.insert(0x9A, instruction!("TXS", Transfer, Cpu::txs, Implied, 2));
    instruction_set.insert(0x98, instruction!("TYA", Transfer, Cpu::tya, Implied, 2));

    // Decrements and increments
    // instruction_set.insert(0xCA, instruction!("DEC", Decrement, Cpu::dec, Immediate, 2));
    instruction_set.insert(0xCA, instruction!("DEX", Decrement, Cpu::dex, Immediate, 2));
    instruction_set.insert(0x88, instruction!("DEY", Decrement, Cpu::dey, Immediate, 2));
    // instruction_set.insert(0xCA, instruction!("INC", Increment, Cpu::inc, Immediate, 2));
    instruction_set.insert(0xCA, instruction!("INX", Increment, Cpu::inx, Immediate, 2));
    instruction_set.insert(0x88, instruction!("INY", Increment, Cpu::iny, Immediate, 2));

    // Logical operations
    instruction_set.insert(0x29, instruction!("AND", Logical, Cpu::and, Immediate, 2));
    instruction_set.insert(0x49, instruction!("EOR", Logical, Cpu::eor, Immediate, 2));
    instruction_set.insert(0x09, instruction!("ORA", Logical, Cpu::ora, Immediate, 2));

    // Flag instructions
    instruction_set.insert(0x18, instruction!("CLC", Flag, Cpu::clc, Implied, 2));
    instruction_set.insert(0xD8, instruction!("CLD", Flag, Cpu::cld, Implied, 2));
    instruction_set.insert(0x58, instruction!("CLI", Flag, Cpu::cli, Implied, 2));
    instruction_set.insert(0xB8, instruction!("CLV", Flag, Cpu::clv, Implied, 2));
    instruction_set.insert(0x38, instruction!("SEC", Flag, Cpu::sec, Implied, 2));
    instruction_set.insert(0xF8, instruction!("SED", Flag, Cpu::sed, Implied, 2));
    instruction_set.insert(0x78, instruction!("SEI", Flag, Cpu::sei, Implied, 2));

    instruction_set
}

#[derive(Debug, Clone, Copy)]
pub enum StatusRegisterFlag {
    Carry = 1 << 0,
    Zero = 1 << 1,
    InterruptDisable = 1 << 2,
    Decimal = 1 << 3,           // unused in the NES
    Break = 1 << 4,
    // bit 5 is unused and is always 1
    Overflow = 1 << 6,
    Negative = 1 << 7,
}
use StatusRegisterFlag::*;

impl Cpu {
    /// Create a new CPU and connect it to a `Memory`.
    pub fn new(bus: Rc<dyn Bus>) -> Self {
        Self {
            acc: 0,
            x_reg: 0,
            y_reg: 0,
            sp: 0,
            pc: 0,
            sr: 0,
            bus,
            instruction_set: legal_opcode_instruction_set(),
        }
    }

    /// Fetch the instruction pointed by the program counter from
    /// memory and execute it atomically.
    pub fn execute(&mut self) {
        let instruction = self.fetch();
        match instruction.instruction {
            // transfer
            Load(fun) => fun(self, instruction.data.unwrap()),
            Transfer(fun) => fun(self),
            // decrements and increments
            Decrement(fun) => fun(self),
            Increment(fun) => fun(self),
            // logical
            Logical(fun) => fun(self, instruction.data.unwrap()),
            // flag
            Flag(fun) => fun(self),
        }
        self.pc += instruction.cycles as u16;
    }

    fn memory_read(&self, address: u16) -> u8 {
        self.bus.read(address)
    }

    fn memory_write(&self, address: u16, data: u8) {
        self.bus.write(address, data);
    }

    // Fetch the instruction pointer by the PC
    fn fetch(&mut self) -> FetchedInstruction {
        let opcode = self.memory_read(self.pc);
        let instruction = self
            .instruction_set
            .get(&opcode)
            .unwrap_or_else(|| panic!("Invalid instruction '0x{:x}'", opcode));

        let (addr, data) = match instruction.addressing {
            Implied => {
                let addr = self.pc + 1;
                let data = None;
                (addr, data)
            }
            Immediate => {
                let addr = self.pc + 1;
                let data = self.memory_read(self.pc + 1);
                (addr, Some(data))
            }
            ZeroPage => {
                // Effective address is 00, ADL
                let adl = self.memory_read(self.pc + 1) as u16;
                let addr = adl;
                let data = self.memory_read(addr);
                (addr, Some(data))
            }
            Absolute => {
                // Effective address is ADH, ADL
                let adl = self.memory_read(self.pc + 1) as u16;
                let adh = (self.memory_read(self.pc + 2) as u16) << 8;
                let addr = adh | adl;
                let data = self.memory_read(addr);
                (addr, Some(data))
            }
            _ => (self.pc, None),
        };

        FetchedInstruction {
            instruction: instruction.instruction.clone(),
            cycles: instruction.cycles,
            address: addr,
            data,
        }
    }

    // Status Register

    fn get_flag(&self, flag: StatusRegisterFlag) -> bool {
        (self.sr & (flag as u8)) > 0
    }

    fn set_flag(&mut self, flag: StatusRegisterFlag, enable: bool) {
        if enable {
            self.sr |= flag as u8;
        } else {
            self.sr &= !(flag as u8);
        }
    }

    fn auto_set_flag(&mut self, flag: StatusRegisterFlag, value: u8) {
        match flag {
            Carry => {}
            Zero => self.set_flag(Zero, value == 0),
            InterruptDisable => {}
            Decimal => {}
            Break => {}
            Overflow => {}
            Negative => self.set_flag(Negative, (value & 1 << 7) > 0),
        }
    }
}

// Instruction Set
impl Cpu {

    // Transfer instructions

    /// LDA - Load Accumulator with Memory
    ///
    /// Operation:
    /// M -> A
    ///
    /// Status Register
    /// N Z C I D V
    /// + + - - - -
    fn lda(&mut self, operand: u8) {
        self.acc = operand;

        self.auto_set_flag(Negative, self.acc);
        self.auto_set_flag(Zero, self.acc);
    }

    /// Load Index X with Memory
    ///
    /// Operation:
    /// M -> X
    ///
    /// Status Register
    /// N Z C I D V
    /// + + - - - -
    fn ldx(&mut self, operand: u8) {
        self.x_reg = operand;

        self.auto_set_flag(Negative, self.x_reg);
        self.auto_set_flag(Zero, self.x_reg);
    }

    /// Load Index Y with Memory
    /// Operation:
    /// M -> Y
    ///
    /// Status Register
    /// N Z C I D V
    /// + + - - - -
    fn ldy(&mut self, operand: u8) {
        self.y_reg = operand;

        self.auto_set_flag(Negative, self.y_reg);
        self.auto_set_flag(Zero, self.y_reg);
    }

    /// TAX - Transfer Accumulator to Index X
    ///
    /// Operation:
    /// A -> X
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + - - - -
    fn tax(&mut self) {
        self.x_reg = self.acc;

        self.auto_set_flag(Negative, self.x_reg);
        self.auto_set_flag(Zero, self.x_reg);
    }

    /// TAY - Transfer Accumulator to Index Y
    ///
    /// Operation:
    /// A -> Y
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + - - - -
    fn tay(&mut self) {
        self.y_reg = self.acc;

        self.auto_set_flag(Negative, self.y_reg);
        self.auto_set_flag(Zero, self.y_reg);
    }

    /// TSX - Transfer Stack Pointer to Index X
    ///
    /// Operation:
    /// SP -> X
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + - - - -
    fn tsx(&mut self) {
        self.x_reg = self.sp;

        self.auto_set_flag(Negative, self.x_reg);
        self.auto_set_flag(Zero, self.x_reg);
    }

    /// TXA - Transfer Index X to Accumulator
    ///
    /// Operation:
    /// X -> A
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + - - - -
    fn txa(&mut self) {
        self.acc = self.x_reg;

        self.auto_set_flag(Negative, self.acc);
        self.auto_set_flag(Zero, self.acc);
    }

    /// TXS - Transfer Index X to Stack Pointer
    ///
    /// Operation:
    /// X -> SP
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + - - - -
    fn txs(&mut self) {
        self.sp = self.x_reg;

        self.auto_set_flag(Negative, self.sp);
        self.auto_set_flag(Zero, self.sp);
    }

    /// TYA - Transfer Index Y to Accumulator
    ///
    /// Operation:
    /// Y -> A
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + - - - -
    fn tya(&mut self) {
        self.acc = self.y_reg;

        self.auto_set_flag(Negative, self.acc);
        self.auto_set_flag(Zero, self.acc);
    }

    // Decrements and increments

    /// DEX - Decrment Index X by One
    ///
    /// Operation:
    /// X + 1 -> X
    ///
    /// Status Register
    /// N Z C I D V
    /// + + - - - -
    fn dex(&mut self) {
        let (res, _) = self.x_reg.overflowing_sub(1);
        self.x_reg = res;

        self.auto_set_flag(Negative, self.x_reg);
        self.auto_set_flag(Zero, self.x_reg);
    }

    /// DEY - Decrment Index Y by One
    ///
    /// Operation:
    /// Y + 1 -> Y
    ///
    /// Status Register
    /// N Z C I D V
    /// + + - - - -
    fn dey(&mut self) {
        let (res, _) = self.y_reg.overflowing_sub(1);
        self.y_reg = res;

        self.auto_set_flag(Negative, self.y_reg);
        self.auto_set_flag(Zero, self.y_reg);
    }

    /// INX - Incrment Index X by One
    ///
    /// Operation:
    /// X + 1 -> X
    ///
    /// Status Register
    /// N Z C I D V
    /// + + - - - -
    fn inx(&mut self) {
        let (res, _) = self.x_reg.overflowing_add(1);
        self.x_reg = res;

        self.auto_set_flag(Negative, self.x_reg);
        self.auto_set_flag(Zero, self.x_reg);
    }

    /// INY - Incrment Index Y by One
    ///
    /// Operation:
    /// Y + 1 -> Y
    ///
    /// Status Register
    /// N Z C I D V
    /// + + - - - -
    fn iny(&mut self) {
        let (res, _) = self.y_reg.overflowing_add(1);
        self.y_reg = res;

        self.auto_set_flag(Negative, self.y_reg);
        self.auto_set_flag(Zero, self.y_reg);
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
    fn and(&mut self, operand: u8) {
        self.acc &= operand;

        self.auto_set_flag(Negative, self.acc);
        self.auto_set_flag(Zero, self.acc);
    }

    /// EOR - Exclusive-OR Memory with Accumulator
    ///
    /// Operation:
    /// A EOR M -> A
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + - - - -
    fn eor(&mut self, operand: u8) {
        self.acc ^= operand;

        self.auto_set_flag(Negative, self.acc);
        self.auto_set_flag(Zero, self.acc);
    }

    /// ORA - OR Memory with Accumulator
    ///
    /// Operation:
    /// A OR M -> A
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + - - - -
    fn ora(&mut self, operand: u8) {
        self.acc |= operand;

        self.auto_set_flag(Negative, self.acc);
        self.auto_set_flag(Zero, self.acc);
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
    fn clc(&mut self) {
        self.set_flag(Carry, false);
    }

    /// CLD - Clear Decimal Mode
    ///
    /// Operation:
    /// 0 -> D
    ///
    /// Status Register:
    /// N Z C I D V
    /// - - - - 0 -
    fn cld(&mut self) {
        self.set_flag(Decimal, false);
    }

    /// CLI - Clear Interrupt Disable Bit 
    ///
    /// Operation:
    /// 0 -> I
    ///
    /// Status Register:
    /// N Z C I D V
    /// - - - 0 - -
    fn cli(&mut self) {
        self.set_flag(InterruptDisable, false);
    }

    /// CLV - Clear Overflow Flag 
    ///
    /// Operation:
    /// 0 -> V
    ///
    /// Status Register:
    /// N Z C I D V
    /// - - - - - 0
    fn clv(&mut self) {
        self.set_flag(Overflow, false);
    }

    /// SEC - Set Carry Flag
    ///
    /// Operation:
    /// 1 -> C
    ///
    /// Status Register:
    /// N Z C I D V
    /// - - 1 - - -
    fn sec(&mut self) {
        self.set_flag(Carry, true);
    }

    /// SED - Set Decimal Flag
    ///
    /// Operation:
    /// 1 -> D
    ///
    /// Status Register:
    /// N Z C I D V
    /// - - - - 1 -
    fn sed(&mut self) {
        self.set_flag(Decimal, true);
    }

    /// SEI - Set Interrupt Disable Status 
    ///
    /// Operation:
    /// 1 -> I
    ///
    /// Status Register:
    /// N Z C I D V
    /// - - - 1 - -
    fn sei(&mut self) {
        self.set_flag(InterruptDisable, true);
    }

}
