#![allow(dead_code, unused_variables)]

use std::collections::HashMap;

use super::memory::Memory;

/// MOS 6502 has multiple addressing modes to fetch operands for
/// instructions.
pub enum Addressing {
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

/// An `Instruction` represents a single MOS 6502 instruction. It has
/// a name, an addressing mode, number of bytes and a function pointer
/// to execute it's corresponding CPU operation.
pub struct Instruction {
    name: &'static str,
    addressing: Addressing,
    bytes: u8,
    cycles: u8,
    execute: fn(&mut Cpu) -> (),
}

/// Status register flags
#[derive(Debug, Clone, Copy)]
enum SRFlag {
    Carry = 1 << 0,
    Zero = 1 << 1,
    DisableInterrupts = 1 << 2,
    // bit 3 is for Decimal Mode, not used in the NES
    Break = 1 << 4,
    // bit 5 is unused and is always 1
    Overflow = 1 << 6,
    Negative = 1 << 7,
}

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
    // TODO: other
}

use self::Addressing::*;
use self::SRFlag::*;

fn legal_opcode_instruction_set() -> HashMap<u8, Instruction> {
    let mut instruction_set = HashMap::new();

    instruction_set.insert(
        0x29,
        Instruction {
            name: "AND",
            addressing: Immediate,
            bytes: 2,
            cycles: 2,
            execute: |cpu| {
                cpu.and(Immediate);
                cpu.pc += 2;
            },
        },
    );

    instruction_set
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

    /// Fetch from the memory address pointed by the program counter
    /// and execute the instruction atomically.
    pub fn execute(&mut self) {
        let opcode = self.memory.read(self.pc);

        let instruction = self
            .instruction_set
            .get(&opcode)
            .expect(&format!("Invalid instruction '{:x}'", opcode));

        (instruction.execute)(self);
    }

    /// Return a status register flag.
    fn get_flag(&self, flag: SRFlag) -> bool {
        (self.sr & (flag as u8)) > 0
    }

    /// Set an specific status register flag.
    fn set_flag(&mut self, flag: SRFlag, enable: bool) {
        if enable {
            self.sr |= flag as u8;
        } else {
            self.sr &= !(flag as u8);
        }
    }

    // Instruction Set implementation

    /// Fetch the operand to perform the instruction. Instructions
    /// with implied addressing doesn't return a value.
    fn fetch(&mut self, addressing: Addressing) -> Option<u8> {
        match addressing {
            Implied => None,
            // Operation on the accumulator
            Accum => Some(self.acc),
            // Operand is in the second byte of the instruction
            Immediate => Some(self.memory.read(self.pc + 1)),
            Absolute => None,
            ZeroPage => None,
            AbsX => None,
            AbsY => None,
            ZpgX => None,
            ZpgY => None,
            Relative => None,
            IndX => None,
            IndY => None,
            AbsIndirect => None,
        }
    }

    /// AND - AND Memory with Accumulator
    ///
    /// Operation:
    /// A AND M -> A
    ///
    /// Status Register:
    /// N Z C I D V
    /// + + - -	- -
    fn and(&mut self, addressing: Addressing) {
        let operand = self.fetch(addressing).unwrap();
        self.acc &= operand;

        self.set_flag(Negative, (self.acc & 1 << 7) > 0);
        self.set_flag(Zero, self.acc == 0);
    }
}

#[cfg(test)]
mod tests {
    use crate::processor::memory::Ram;

    use super::*;

    #[test]
    fn test_status_register() {
        let ram = Ram::new();
        let mut cpu = Cpu::new(&ram);
        let flags = vec![
            SRFlag::Carry,
            SRFlag::Zero,
            SRFlag::DisableInterrupts,
            SRFlag::Break,
            SRFlag::Overflow,
            SRFlag::Negative,
        ];

        for flag in flags {
            assert!(!cpu.get_flag(flag));
            cpu.set_flag(flag, true);
            assert!(cpu.get_flag(flag));
            cpu.set_flag(flag, false);
            assert!(!cpu.get_flag(flag));
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_instruction_AND() {
        let mut ram = Ram::new();
        // Load test program
        #[rustfmt::skip]
            ram.load(0, &vec![
                0x29, 0xff,         // A AND 0xff
                0x29, 0x0f,         // A AND 0x0f
                0x29, 0x00,         // A AND 0x00
            ]);

        let mut cpu = Cpu::new(&ram);
        cpu.acc = 0xac;

        cpu.execute();
        assert_eq!(cpu.acc, 0xac);
        assert!(!cpu.get_flag(Zero));
        assert!(cpu.get_flag(Negative));

        cpu.execute();
        assert_eq!(cpu.acc, 0x0c);
        assert!(!cpu.get_flag(Zero));
        assert!(!cpu.get_flag(Negative));

        cpu.execute();
        assert_eq!(cpu.acc, 0x00);
        assert!(cpu.get_flag(Zero));
        assert!(!cpu.get_flag(Negative));
    }
}
