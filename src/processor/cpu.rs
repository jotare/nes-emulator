#![allow(dead_code, unused_variables)]

use std::collections::HashMap;

use crate::processor::memory::Memory;

/// MOS 6502 processor emulator.
///
/// CPU abstraction is connected to a `Memory` to perform read and
/// write operations on it.
///
/// This implementation uses the legal opcode instruction set. Illegal
/// instructions are not implemented.
pub struct Cpu<'a> {
    acc: u8,   // Accumulator
    x_reg: u8, // X register
    y_reg: u8, // Y register
    sp: u8,    // Stack Pointer
    pc: u16,   // Program Counter
    sr: u8,    // Status Register
    memory: &'a dyn Memory,
    instruction_set: HashMap<u8, Instruction>,
}

impl<'a> Cpu<'a> {
    /// Create a new CPU and connect it to a `Memory`.
    pub fn new(memory: &'a dyn Memory) -> Self {
        Self {
            acc: 0,
            x_reg: 0,
            y_reg: 0,
            sp: 0,
            pc: 0,
            sr: 0,
            memory,
            instruction_set: legal_opcode_instruction_set(),
        }
    }

    /// Fetch the instruction pointed by the program counter from
    /// memory and execute it atomically.
    pub fn execute(&mut self) {
        let instruction = self.fetch();
        match instruction.instruction {
            Logical(fun) => fun(self, instruction.data.unwrap()),
            Jump(fun) => fun(self, instruction.address),
        }
        self.pc += instruction.cycles as u16;
    }

    // Fetch the instruction pointer by the PC
    fn fetch(&mut self) -> FetchedInstruction {
        let opcode = self.memory.read(self.pc);
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
                let data = self.memory.read(self.pc + 1);
                (addr, Some(data))
            }
            ZeroPage => {
                // Effective address is 00, ADL
                let adl = self.memory.read(self.pc + 1) as u16;
                let addr = 0x00 << 8 | adl;
                let data = self.memory.read(addr);
                (addr, Some(data))
            }
            Absolute => {
                // Effective address is ADH, ADL
                let adl = self.memory.read(self.pc + 1) as u16;
                let adh = (self.memory.read(self.pc + 2) as u16) << 8;
                let addr = adh | adl;
                let data = self.memory.read(addr);
                (addr, Some(data))
            }
            _ => (self.pc, None),
        };

        FetchedInstruction {
            instruction: instruction.instruction.clone(),
            cycles: instruction.cycles,
            address: addr,
            data: data,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StatusRegisterFlag {
    Carry = 1 << 0,
    Zero = 1 << 1,
    DisableInterrupts = 1 << 2,
    // bit 3 is for Decimal Mode, not used in the NES
    Break = 1 << 4,
    // bit 5 is unused and is always 1
    Overflow = 1 << 6,
    Negative = 1 << 7,
}
use StatusRegisterFlag::*;

impl<'a> Cpu<'a> {
    // Return a status register flag.
    fn get_flag(&self, flag: StatusRegisterFlag) -> bool {
        (self.sr & (flag as u8)) > 0
    }

    // Set an specific status register flag.
    fn set_flag(&mut self, flag: StatusRegisterFlag, enable: bool) {
        if enable {
            self.sr |= flag as u8;
        } else {
            self.sr &= !(flag as u8);
        }
    }
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
    Logical(fn(&mut Cpu, u8)),
    Jump(fn(&mut Cpu, u16)),
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

macro_rules! instruction {
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

    // Logical operations
    instruction_set.insert(0x29, instruction!("AND", Logical, Cpu::and, Immediate, 2));
    instruction_set.insert(0x49, instruction!("EOR", Logical, Cpu::eor, Immediate, 2));
    instruction_set.insert(0x09, instruction!("ORA", Logical, Cpu::ora, Immediate, 2));

    instruction_set
}

impl<'a> Cpu<'a> {
    // Logical operations

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

        self.set_flag(Negative, (self.acc & 1 << 7) > 0);
        self.set_flag(Zero, self.acc == 0);
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

        self.set_flag(Negative, (self.acc & 1 << 7) > 0);
        self.set_flag(Zero, self.acc == 0);
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

        self.set_flag(Negative, (self.acc & 1 << 7) > 0);
        self.set_flag(Zero, self.acc == 0);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::processor::memory::Memory;
    use crate::processor::memory::Ram;

    use super::*;

    #[test]
    fn test_status_register() {
        let ram = Ram::new();
        let mut cpu = Cpu::new(&ram);
        let flags = vec![
            StatusRegisterFlag::Carry,
            StatusRegisterFlag::Zero,
            StatusRegisterFlag::DisableInterrupts,
            StatusRegisterFlag::Break,
            StatusRegisterFlag::Overflow,
            StatusRegisterFlag::Negative,
        ];

        for flag in flags {
            assert!(!cpu.get_flag(flag));
            cpu.set_flag(flag, true);
            assert!(cpu.get_flag(flag));
            cpu.set_flag(flag, false);
            assert!(!cpu.get_flag(flag));
        }
    }

    struct MockRam {
        memory: RefCell<Vec<u8>>,
    }

    impl MockRam {
        pub fn new() -> Self {
            Self {
                memory: RefCell::new(Vec::new()),
            }
        }

        /// Set mock memory to the `instruction`
        pub fn add_instruction(&self, instruction: Vec<u8>) {
            let mut memory = self.memory.borrow_mut();
            for byte in instruction {
                memory.push(byte);
            }
        }
    }

    impl Memory for MockRam {
        fn read(&self, address: u16) -> u8 {
            self.memory.borrow()[address as usize]
        }

        fn write(&mut self, address: u16, data: u8) {
            self.memory.borrow_mut()[address as usize] = data;
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_logical_instructions() {
        let mock_ram = MockRam::new();
        let mut cpu = Cpu::new(&mock_ram);
        cpu.acc = 0xAC;

        // A AND 0xFF = A = 0xAC
        mock_ram.add_instruction(vec![0x29, 0xFF]);
        cpu.execute();
        assert_eq!(cpu.acc, 0xAC);
        assert!(!cpu.get_flag(Zero));
        assert!(cpu.get_flag(Negative));

        // A AND 0x0F = 0x0C
        mock_ram.add_instruction(vec![0x29, 0x0F]);
        cpu.execute();
        assert_eq!(cpu.acc, 0x0C);
        assert!(!cpu.get_flag(Zero));
        assert!(!cpu.get_flag(Negative));

        // A AND 0x00 = 0x00
        mock_ram.add_instruction(vec![0x29, 0x00]);
        cpu.execute();
        assert_eq!(cpu.acc, 0x00);
        assert!(cpu.get_flag(Zero));
        assert!(!cpu.get_flag(Negative));

        // A ORA 0x00 = 0x00
        mock_ram.add_instruction(vec![0x09, 0x00]);
        cpu.execute();
        assert_eq!(cpu.acc, 0x00);
        assert!(cpu.get_flag(Zero));
        assert!(!cpu.get_flag(Negative));

        // A ORA 0xAB = 0xAB
        mock_ram.add_instruction(vec![0x09, 0xAB]);
        cpu.execute();
        assert_eq!(cpu.acc, 0xAB);
        assert!(!cpu.get_flag(Zero));
        assert!(cpu.get_flag(Negative));

        // A ORA 0xCC = 0xEF
        mock_ram.add_instruction(vec![0x09, 0xCC]);
        cpu.execute();
        assert_eq!(cpu.acc, 0xEF);
        assert!(!cpu.get_flag(Zero));
        assert!(cpu.get_flag(Negative));

        // A EOR 0x88 = 0x67
        mock_ram.add_instruction(vec![0x49, 0x88]);
        cpu.execute();
        assert_eq!(cpu.acc, 0x67);
        assert!(!cpu.get_flag(Zero));
        assert!(!cpu.get_flag(Negative));

        // A EOR 0x67 = 0x00
        mock_ram.add_instruction(vec![0x49, 0x67]);
        cpu.execute();
        assert_eq!(cpu.acc, 0x00);
        assert!(cpu.get_flag(Zero));
        assert!(!cpu.get_flag(Negative));

        // A EOR 0x80 = 0x80
        mock_ram.add_instruction(vec![0x49, 0x80]);
        cpu.execute();
        assert_eq!(cpu.acc, 0x80);
        assert!(!cpu.get_flag(Zero));
        assert!(cpu.get_flag(Negative));
    }
}
