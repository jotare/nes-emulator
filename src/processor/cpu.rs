#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::rc::Rc;

use super::bus::Bus;
use super::utils::bv;

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
#[derive(Clone, Debug)]
enum AddressingMode {
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
}
use AddressingMode::*;

#[derive(Clone)]
pub enum InstructionKind {
    SingleByte(fn(&mut Cpu)),
    InternalExecOnMemoryData(fn(&mut Cpu, u8)),
    StoreOp(fn(&mut Cpu) -> u8),
    ReadModifyWrite(fn(&mut Cpu, u8) -> u8),
    Misc,
}
use InstructionKind::*;

/// An `Instruction` represents a single MOS 6502 instruction. It has
/// a name, an addressing mode, number of bytes and a function pointer
/// to execute it's corresponding CPU operation.
pub struct Instruction {
    name: &'static str,
    instruction: InstructionKind,
    addressing: AddressingMode,
    cycles: u8,
}

macro_rules! instruction {
    ($name:expr, SingleByte, Cpu::$fun:ident, $addr_mode:expr, $cycles:expr) => {
        Instruction {
            name: $name,
            instruction: SingleByte(|cpu| Cpu::$fun(cpu)),
            addressing: $addr_mode,
            cycles: $cycles,
        }
    };
    ($name:expr, InternalExecOnMemoryData, Cpu::$fun:ident, $addr_mode:expr, $cycles:expr) => {
        Instruction {
            name: $name,
            instruction: InternalExecOnMemoryData(|cpu, operand| Cpu::$fun(cpu, operand)),
            addressing: $addr_mode,
            cycles: $cycles,
        }
    };
    ($name:expr, StoreOp, Cpu::$fun:ident, $addr_mode:expr, $cycles:expr) => {
        Instruction {
            name: $name,
            instruction: StoreOp(|cpu| Cpu::$fun(cpu)),
            addressing: $addr_mode,
            cycles: $cycles,
        }
    };
    ($name:expr, ReadModifyWrite, Cpu::$fun:ident, $addr_mode:expr, $cycles:expr) => {
        Instruction {
            name: $name,
            instruction: ReadModifyWrite(|cpu, operand| Cpu::$fun(cpu, operand)),
            addressing: $addr_mode,
            cycles: $cycles,
        }
    };
    ($name:expr, Misc, Cpu::$fun:ident, $addr_mode:expr, $cycles:expr) => {
        Instruction {
            name: $name,
            instruction: Misc,
            addressing: $addr_mode,
            cycles: $cycles,
        }
    };
}

#[rustfmt::skip]
pub fn legal_opcode_instruction_set() -> HashMap<u8, Instruction> {
    let mut instruction_set = HashMap::new();

    // Transfer instructions
    instruction_set.insert(0xA9, instruction!("LDA", InternalExecOnMemoryData, Cpu::lda, Immediate, 2));
    instruction_set.insert(0xA2, instruction!("LDX", InternalExecOnMemoryData, Cpu::ldx, Immediate, 2));
    instruction_set.insert(0xA0, instruction!("LDY", InternalExecOnMemoryData, Cpu::ldy, Immediate, 2));

    instruction_set.insert(0x85, instruction!("STA", StoreOp, Cpu::sta, ZeroPage, 2));
    instruction_set.insert(0x86, instruction!("STX", StoreOp, Cpu::stx, ZeroPage, 2));
    instruction_set.insert(0x84, instruction!("STY", StoreOp, Cpu::sty, ZeroPage, 2));

    instruction_set.insert(0xAA, instruction!("TAX", SingleByte, Cpu::tax, Implied, 2));
    instruction_set.insert(0xA8, instruction!("TAY", SingleByte, Cpu::tay, Implied, 2));
    instruction_set.insert(0xBA, instruction!("TSX", SingleByte, Cpu::tsx, Implied, 2));
    instruction_set.insert(0x8A, instruction!("TXA", SingleByte, Cpu::txa, Implied, 2));
    instruction_set.insert(0x9A, instruction!("TXS", SingleByte, Cpu::txs, Implied, 2));
    instruction_set.insert(0x98, instruction!("TYA", SingleByte, Cpu::tya, Implied, 2));

    // Decrements and increments
    instruction_set.insert(0xC6, instruction!("DEC", ReadModifyWrite, Cpu::dec, ZeroPage, 2));
    instruction_set.insert(0xCA, instruction!("DEX", SingleByte, Cpu::dex, Immediate, 2));
    instruction_set.insert(0x88, instruction!("DEY", SingleByte, Cpu::dey, Immediate, 2));
    instruction_set.insert(0xE6, instruction!("INC", ReadModifyWrite, Cpu::inc, ZeroPage, 2));
    instruction_set.insert(0xCA, instruction!("INX", SingleByte, Cpu::inx, Immediate, 2));
    instruction_set.insert(0x88, instruction!("INY", SingleByte, Cpu::iny, Immediate, 2));

    // Arithmetic operations
    instruction_set.insert(0x69, instruction!("ADC", InternalExecOnMemoryData, Cpu::adc, Immediate, 2));
    instruction_set.insert(0xE9, instruction!("SBC", InternalExecOnMemoryData, Cpu::sbc, Immediate, 2));

    // Logical operations
    instruction_set.insert(0x29, instruction!("AND", InternalExecOnMemoryData, Cpu::and, Immediate, 2));
    instruction_set.insert(0x49, instruction!("EOR", InternalExecOnMemoryData, Cpu::eor, Immediate, 2));
    instruction_set.insert(0x09, instruction!("ORA", InternalExecOnMemoryData, Cpu::ora, Immediate, 2));

    // Shift and rotation instructions
    instruction_set.insert(0x0A, instruction!("ASL", SingleByte, Cpu::asl, Accumulator, 2));
    instruction_set.insert(0x4A, instruction!("LSR", SingleByte, Cpu::lsr, Accumulator, 2));
    instruction_set.insert(0x2A, instruction!("ROL", SingleByte, Cpu::rol, Accumulator, 2));
    instruction_set.insert(0x6A, instruction!("ROR", SingleByte, Cpu::ror, Accumulator, 2));

    // Flag instructions
    instruction_set.insert(0x18, instruction!("CLC", SingleByte, Cpu::clc, Implied, 2));
    instruction_set.insert(0xD8, instruction!("CLD", SingleByte, Cpu::cld, Implied, 2));
    instruction_set.insert(0x58, instruction!("CLI", SingleByte, Cpu::cli, Implied, 2));
    instruction_set.insert(0xB8, instruction!("CLV", SingleByte, Cpu::clv, Implied, 2));
    instruction_set.insert(0x38, instruction!("SEC", SingleByte, Cpu::sec, Implied, 2));
    instruction_set.insert(0xF8, instruction!("SED", SingleByte, Cpu::sed, Implied, 2));
    instruction_set.insert(0x78, instruction!("SEI", SingleByte, Cpu::sei, Implied, 2));

    // Comparaisons
    instruction_set.insert(0xC9, instruction!("CMP", InternalExecOnMemoryData, Cpu::cmp, Immediate, 2));
    instruction_set.insert(0xE0, instruction!("CPX", InternalExecOnMemoryData, Cpu::cpx, Immediate, 2));
    instruction_set.insert(0xC0, instruction!("CPY", InternalExecOnMemoryData, Cpu::cpy, Immediate, 2));
    // Conditional branch instructions
    instruction_set.insert(0x90, instruction!("BCC", Misc, Cpu::bcc, Relative, 2));
    instruction_set.insert(0xB0, instruction!("BCS", Misc, Cpu::bcs, Relative, 2));
    instruction_set.insert(0xF0, instruction!("BEQ", Misc, Cpu::beq, Relative, 2));
    instruction_set.insert(0x30, instruction!("BMI", Misc, Cpu::bmi, Relative, 2));
    instruction_set.insert(0xD0, instruction!("BNE", Misc, Cpu::bne, Relative, 2));
    instruction_set.insert(0x10, instruction!("BPL", Misc, Cpu::bpl, Relative, 2));
    instruction_set.insert(0x50, instruction!("BVC", Misc, Cpu::bvc, Relative, 2));
    instruction_set.insert(0x70, instruction!("BVS", Misc, Cpu::bvs, Relative, 2));


    // Other
    instruction_set.insert(0xEA, instruction!("NOP", SingleByte, Cpu::nop, Implied, 2));

    instruction_set
}

#[derive(Debug, Clone, Copy)]
pub enum StatusRegisterFlag {
    Carry = 1 << 0,
    Zero = 1 << 1,
    InterruptDisable = 1 << 2,
    Decimal = 1 << 3, // unused in the NES
    Break = 1 << 4,
    // bit 5 is unused and is always 1
    Overflow = 1 << 6,
    Negative = 1 << 7,
}
use StatusRegisterFlag::*;

impl Cpu {
    /// Create a new CPU and connect it to a `Memory`.
    pub fn new(bus: Rc<dyn Bus>) -> Self {
        let mut new = Self {
            acc: 0,
            x_reg: 0,
            y_reg: 0,
            sp: 0xFF, // 256 byte stack between 0x0100 - 0x01FF. Stack Pointer 0x00 - 0xFF
            pc: 0,
            sr: 0,
            bus,
            instruction_set: legal_opcode_instruction_set(),
        };
        new.reset();
        new
    }

    pub fn reset(&mut self) {
        self.acc = 0;
        self.x_reg = 0;
        self.y_reg = 0;
        self.sp = 0xFF;
        self.pc = 0;
        self.sr = 0;
    }

    /// Fetch the instruction pointed by the program counter from
    /// memory and execute it atomically.
    pub fn execute(&mut self) {
        let opcode = self.memory_read(self.pc);
        let instruction = self
            .instruction_set
            .get(&opcode)
            .unwrap_or_else(|| panic!("Invalid instruction '0x{:x}'", opcode));

        let addressing = instruction.addressing.clone();
        let cycles = instruction.cycles;
        let instruction = instruction.instruction.clone();

        match instruction {
            SingleByte(fun) => {
                // both address and data will be discarted
                fun(self);
            }
            InternalExecOnMemoryData(fun) => {
                let (_, data) = self.load(addressing);
                fun(self, data);
            }
            StoreOp(fun) => {
                let data = fun(self);
                self.store(data, addressing);
            }
            ReadModifyWrite(fun) => {
                let (_, data) = self.load(addressing.clone());
                let result = fun(self, data);
                self.store(result, addressing);
            }
            Misc => {}
        }

        self.pc += cycles as u16;
    }

    fn memory_read(&self, address: u16) -> u8 {
        self.bus.read(address)
    }

    fn memory_write(&self, address: u16, data: u8) {
        self.bus.write(address, data);
    }

    fn load(&mut self, addr_mode: AddressingMode) -> (u16, u8) {
        let opcode = self.memory_read(self.pc);
        let (addr, data) = match addr_mode {
            Implied => {
                let addr = self.pc + 1;
                let data = opcode; // discarted
                (addr, data)
            }
            Accumulator => {
                let addr = self.pc + 1;
                let data = self.acc;
                (addr, data)
            }
            Immediate => {
                let addr = self.pc + 1;
                let data = self.memory_read(self.pc + 1);
                (addr, data)
            }
            ZeroPage => {
                // Effective address is 00, ADL
                let adl = self.memory_read(self.pc + 1) as u16;
                let addr = adl;
                let data = self.memory_read(addr);
                (addr, data)
            }
            Absolute => {
                // Effective address is ADH, ADL
                let adl = self.memory_read(self.pc + 1) as u16;
                let adh = (self.memory_read(self.pc + 2) as u16) << 8;
                let addr = adh | adl;
                let data = self.memory_read(addr);
                (addr, data)
            }
            IndirectX => {
                // page zero base address
                let bal = self.memory_read(self.pc + 1) as u16;
                let adl = self.memory_read(bal + (self.x_reg as u16)) as u16;
                let adh = self.memory_read(bal + (self.x_reg as u16) + 1) as u16;
                let addr = (adh << 8) | adl;
                let data = self.memory_read(addr);
                (addr, data)
            }
            AbsoluteX => {
                let bal = self.memory_read(self.pc + 1) as u16;
                let bah = self.memory_read(self.pc + 2) as u16;
                let addr = ((bah << 8) | bal) + self.x_reg as u16;
                let data = self.memory_read(addr);
                (addr, data)
            }
            AbsoluteY => {
                let bal = self.memory_read(self.pc + 1) as u16;
                let bah = self.memory_read(self.pc + 2) as u16;
                let addr = ((bah << 8) | bal) + self.y_reg as u16;
                let data = self.memory_read(addr);
                (addr, data)
            }
            ZeroPageX => {
                let bal = self.memory_read(self.pc + 1) as u16;
                let addr = bal + self.x_reg as u16;
                let data = self.memory_read(addr);
                (addr, data)
            }
            ZeroPageY => {
                let bal = self.memory_read(self.pc + 1) as u16;
                let addr = bal + self.y_reg as u16;
                let data = self.memory_read(addr);
                (addr, data)
            }
            IndirectY => {
                let ial = self.memory_read(self.pc + 1) as u16;
                let bal = self.memory_read(ial) as u16;
                let bah = self.memory_read(ial + 1) as u16;
                let addr = ((bah << 8) | bal) + self.y_reg as u16;
                let data = self.memory_read(addr);
                (addr, data)
            }
        };
        (addr, data)
    }

    fn store(&mut self, data: u8, addr_mode: AddressingMode) {
        let addr = match addr_mode {
            ZeroPage => {
                self.memory_read(self.pc + 1) as u16
            }
            Absolute => {
                let adl = self.memory_read(self.pc + 1) as u16;
                let adh = (self.memory_read(self.pc + 2) as u16) << 8;
                adh | adl
            }
            IndirectX => {
                let bal = self.memory_read(self.pc + 1) as u16;
                let adl = self.memory_read(bal + (self.x_reg as u16)) as u16;
                let adh = self.memory_read(bal + (self.x_reg as u16) + 1) as u16;
                (adh << 8) | adl
            }
            AbsoluteX => {
                todo!();
            }
            AbsoluteY => {
                todo!();
            }
            ZeroPageX => {
                let bal = self.memory_read(self.pc + 1) as u16;
                (bal + (self.x_reg as u16)) & 0x00FF
            }
            ZeroPageY => {
                let bal = self.memory_read(self.pc + 1) as u16;
                (bal + (self.y_reg as u16)) & 0x00FF
            }
            IndirectY => {
                todo!();
            }
            _ => {
                panic!("Invalid store addressing mode: {:?}", addr_mode);
            }
        };
        self.memory_write(addr, data);
    }

    // Status Register

    fn flag(&self, flag: StatusRegisterFlag) -> bool {
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
            Zero => self.set_flag(Zero, value == 0),
            Negative => self.set_flag(Negative, bv(value, 7) != 0),
            _ => panic!("Auto set flag {:?} not implemented", flag),
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

    /// LDX - Load Index X with Memory
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

    /// LDY - Load Index Y with Memory
    ///
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

    /// STA - Store Accumulator in Memory
    ///
    /// Operation:
    /// A -> M
    ///
    /// Status Register
    /// N Z C I D V
    /// - - - - - -
    fn sta(&mut self) -> u8 {
        self.acc
    }

    /// STX - Store Index X in Memory
    ///
    /// Operation:
    /// X -> M
    ///
    /// Status Register
    /// N Z C I D V
    /// - - - - - -
    fn stx(&mut self) -> u8 {
        self.x_reg
    }

    /// STY - Store Index Y in Memory
    /// Operation:
    /// Y -> M
    ///
    /// Status Register
    /// N Z C I D V
    /// - - - - - -
    fn sty(&mut self) -> u8 {
        self.y_reg
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

    /// DEC - Decrment Memory by One
    ///
    /// Operation:
    /// M - 1 -> M
    ///
    /// Status Register
    /// N Z C I D V
    /// + + - - - -
    fn dec(&mut self, operand: u8) -> u8 {
        let (res, _) = operand.overflowing_sub(1);

        self.auto_set_flag(Negative, res);
        self.auto_set_flag(Zero, res);

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
    fn dex(&mut self) {
        let (res, _) = self.x_reg.overflowing_sub(1);
        self.x_reg = res;

        self.auto_set_flag(Negative, self.x_reg);
        self.auto_set_flag(Zero, self.x_reg);
    }

    /// DEY - Decrment Index Y by One
    ///
    /// Operation:
    /// Y - 1 -> Y
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

    /// INC - Increment Memory by One
    ///
    /// Operation:
    /// M + 1 -> M
    ///
    /// Status Register
    /// N Z C I D V
    /// + + - - - -
    fn inc(&mut self, operand: u8) -> u8 {
        let (res, _) = operand.overflowing_add(1);

        self.auto_set_flag(Negative, res);
        self.auto_set_flag(Zero, res);

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

    // Arithmetic operations

    /// ADC - Add Memory to Accumulator with Carry
    ///
    /// Operation:
    /// A + M + C -> A, C
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + + - - +
    fn adc(&mut self, operand: u8) {
        let carry = if self.flag(Carry) { 1 } else { 0 };

        let res = self.acc as u16 + operand as u16 + carry;
        let carry = (res & (1 << 8)) != 0;
        let res = res as u8;
        let overflow = bv(self.acc, 7) == bv(operand, 7) && bv(operand, 7) != bv(res, 7);

        self.acc = res;
        self.auto_set_flag(Negative, self.acc);
        self.auto_set_flag(Zero, self.acc);
        self.set_flag(Carry, carry);
        self.set_flag(Overflow, overflow);
    }

    /// SBC - Substract Memory from Accumulator with Borrow
    ///
    /// Operation:
    /// A - M - ^C -> A
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + + - - +
    fn sbc(&mut self, operand: u8) {
        let carry = if self.flag(Carry) { 1 } else { 0 };
        let (res, carry) = self.acc.overflowing_sub(operand + carry);
        let overflow = bv(self.acc, 7) == bv(operand, 7) && bv(operand, 7) != bv(res, 7);

        self.acc = res;
        self.auto_set_flag(Negative, self.acc);
        self.auto_set_flag(Zero, self.acc);
        self.set_flag(Carry, carry);
        self.set_flag(Overflow, overflow);
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

    // Shift & Rotate instructions

    /// ASL - Shift Left One Bit (Memory or Accumulator)
    ///
    /// Operation:
    /// C <- [76543210] <- 0
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + + - - -
    fn asl(&mut self) {
        let carry = bv(self.acc, 7) != 0;
        self.acc <<= 1;
        self.auto_set_flag(Negative, self.acc);
        self.auto_set_flag(Zero, self.acc);
        self.set_flag(Carry, carry);
    }

    /// LSR - Shift One Bit Right (Memory or Accumulator)
    ///
    /// Operation:
    /// 0 -> [76543210] -> C
    ///
    /// Status Register:
    /// N Z C I D V
    /// 0 + + - - -
    fn lsr(&mut self) {
        let carry = bv(self.acc, 0) != 0;
        self.acc >>= 1;
        self.auto_set_flag(Negative, self.acc);
        self.auto_set_flag(Zero, self.acc);
        self.set_flag(Carry, carry);
    }

    /// ROL - Rotate One Bit Left (Memory or Accumulator)
    ///
    /// Operation:
    /// C <- [76543210] <- C
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + + - - -
    fn rol(&mut self) {
        let new_carry = bv(self.acc, 7) != 0;
        let curr_carry = if self.flag(Carry) { 1 } else { 0 };
        self.acc = self.acc << 1 | curr_carry;
        self.auto_set_flag(Negative, self.acc);
        self.auto_set_flag(Zero, self.acc);
        self.set_flag(Carry, new_carry);
    }

    /// ROR - Rotate One Bit Right (Memory or Accumulator)
    ///
    /// Operation:
    /// C -> [76543210] -> C
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + + - - -
    fn ror(&mut self) {
        let new_carry = bv(self.acc, 0) != 0;
        let curr_carry = if self.flag(Carry) { 1 } else { 0 };
        self.acc = self.acc >> 1 | (curr_carry << 7);
        self.auto_set_flag(Negative, self.acc);
        self.auto_set_flag(Zero, self.acc);
        self.set_flag(Carry, new_carry);
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

    // Comparaisons

    fn generic_cmp(&mut self, a: u8, b: u8) {
        let (res, _) = a.overflowing_sub(b);
        self.auto_set_flag(Negative, res);
        self.auto_set_flag(Zero, res);
        self.set_flag(Carry, a >= b);
    }

    /// CMP - Compare Memory with Accumulator
    ///
    /// Operation:
    /// A - M
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + + - - -
    fn cmp(&mut self, operand: u8) {
        self.generic_cmp(self.acc, operand);
    }

    /// CPX - Compare Memory and Index X
    ///
    /// Operation:
    /// X - M
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + + - - -
    fn cpx(&mut self, operand: u8) {
        self.generic_cmp(self.x_reg, operand);
    }

    /// CPY - Compare Memory and Index Y
    ///
    /// Operation:
    /// Y - M
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + + - - -
    fn cpy(&mut self, operand: u8) {
        self.generic_cmp(self.y_reg, operand);
    }

    // Conditional branch

    fn branch(&mut self, condition: bool, offset: u8) {
        if condition {
            let carry = if self.flag(Carry) { 1 } else { 0 };
            self.pc += (offset as u16) + carry;
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
    fn bcc(&mut self, offset: u8) {
        self.branch(!self.flag(Carry), offset);
    }

    /// BCS - Branch on Carry Set
    ///
    /// Operation:
    /// branch on C = 1
    ///
    /// Status Register:
    /// N Z C I D V
    /// - - - - - -
    fn bcs(&mut self, offset: u8) {
        self.branch(self.flag(Carry), offset);
    }

    /// BEQ - Branch on Result Zero
    ///
    /// Operation:
    /// branch on Z = 1
    ///
    /// Status Register:
    /// N Z C I D V
    /// - - - - - -
    fn beq(&mut self, offset: u8) {
        self.branch(self.flag(Zero), offset);
    }

    /// BMI - Branch on Result Minus
    ///
    /// Operation:
    /// branch on N = 1
    ///
    /// Status Register:
    /// N Z C I D V
    /// - - - - - -
    fn bmi(&mut self, offset: u8) {
        self.branch(self.flag(Negative), offset);
    }

    /// BNE - Branch on Result not Zero
    ///
    /// Operation:
    /// branch on Z = 0
    ///
    /// Status Register:
    /// N Z C I D V
    /// - - - - - -
    fn bne(&mut self, offset: u8) {
        self.branch(!self.flag(Zero), offset);
    }

    /// BPL - Branch on Result Plus
    ///
    /// Operation:
    /// branch on N = 0
    ///
    /// Status Register:
    /// N Z C I D V
    /// - - - - - -
    fn bpl(&mut self, offset: u8) {
        self.branch(!self.flag(Negative), offset);
    }

    /// BVC - Branch on Overflow Clear
    ///
    /// Operation:
    /// branch on V = 0
    ///
    /// Status Register:
    /// N Z C I D V
    /// - - - - - -
    fn bvc(&mut self, offset: u8) {
        self.branch(!self.flag(Overflow), offset);
    }

    /// BVS - Branch on Overflow Set
    ///
    /// Operation:
    /// branch on V = 1
    ///
    /// Status Register:
    /// N Z C I D V
    /// - - - - - -
    fn bvs(&mut self, offset: u8) {
        self.branch(self.flag(Overflow), offset);
    }

    // Other

    /// NOP - No Operation
    ///
    /// Operation:
    /// ---
    ///
    /// Status Register:
    /// N Z C I D V
    /// - - - - - -
    fn nop(&mut self) {}
}
