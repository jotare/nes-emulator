use log::{debug, info, warn};

use crate::interfaces::Bus as _;
use crate::processor::instruction::{
    AddressingMode, Instruction, InstructionKind, MiscInstructionKind,
};
use crate::processor::instruction_set;
use crate::processor::instruction_set::InstructionSet;
use crate::processor::internal_cpu::InternalCpu;
use crate::processor::status_register::StatusRegisterFlag;
use crate::types::SharedBus;

use AddressingMode::*;
use InstructionKind::*;
use MiscInstructionKind::*;
use StatusRegisterFlag::*;

pub struct Cpu {
    cpu: InternalCpu,
    instruction_set: InstructionSet,
    bus: SharedBus,

    clocks_before_next_execution: u8,
    page_boundary_cross_extra_clocks: u8,

    interrupt_request: Option<Interrupt>,
}

#[derive(Copy, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Interrupt {
    NonMaskableInterrupt, // NMI
    Reset,                // RES
    InterruptRequest,     // IRQ
}

impl Cpu {
    pub fn new(bus: SharedBus) -> Self {
        Self {
            cpu: InternalCpu::default(),
            instruction_set: InstructionSet::new_legal_opcode_set(),
            bus,
            clocks_before_next_execution: 1,
            page_boundary_cross_extra_clocks: 0,
            interrupt_request: None,
        }
    }

    /// Reset the processor to an init state. After concrete CPU
    /// initializations, it'll call the Reset vector (RES interrupt) and leave
    /// further state initialization to it.
    pub fn reset(&mut self) {
        info!("CPU reset");
        self.cpu.acc = 0;
        self.cpu.x_reg = 0;
        self.cpu.y_reg = 0;
        self.cpu.sp = 0xFF;
        self.cpu.sr.reset();

        self.clocks_before_next_execution = 1;
        self.page_boundary_cross_extra_clocks = 0;

        // read address provided in the reset vector
        let pcl = self.bus_read(0xFFFC) as u16;
        let pch = self.bus_read(0xFFFD) as u16;
        self.cpu.pc = (pch << 8) | pcl;
    }

    /// Perform a clock on the CPU. This emulation of CPU doesn't perform
    /// operations at clock level. Instead, at the end of an instruction cycle,
    /// it is executed atomically. The CPU will wait until the last cycle
    /// though, emulating real CPU clock time. A pending interrupt will wait
    /// until the current instruction is completely executed.
    pub fn clock(&mut self) -> Result<(), String> {
        self.clocks_before_next_execution -= 1;
        if self.clocks_before_next_execution > 0 {
            return Ok(());
        }

        match self.interrupt_request.take() {
            Some(interrupt) => {
                self.execute_interrupt(interrupt);
                // Attending an interrupt takes 7 clocks: 2 for internal
                // operations, 2 to push the return address, 1 for the status
                // register, and 2 more to get the interrupt begin address
                self.clocks_before_next_execution = 7;
                Ok(())
            }
            None => {
                let instruction = self.fetch()?;
                let name = instruction.name;
                let cycles = instruction.cycles;
                let page_crossing_cost = instruction.page_crossing_cost;

                self.cpu.page_boundary_crossed = false;
                self.execute_instruction(instruction)?;

                if self.cpu.page_boundary_crossed {
                    self.page_boundary_cross_extra_clocks += page_crossing_cost;
                }

                self.clocks_before_next_execution = cycles + self.page_boundary_cross_extra_clocks;
                self.page_boundary_cross_extra_clocks = 0;

                Ok(())
            }
        }
    }

    /// Execute a CPU interrupt
    pub fn interrupt(&mut self, interrupt: Interrupt) {
        if self.interrupt_request.is_some() {
            warn!("Attempting to interrupt CPU while there's a pending interruption");
        }
        self.interrupt_request.replace(interrupt);
    }

    /// Execute a complete instruction and return the number of clocks used
    pub fn execute(&mut self) -> Result<u8, String> {
        let instruction = self.fetch()?;
        let mut clocks = instruction.cycles;
        self.execute_instruction(instruction)?;
        clocks += self.page_boundary_cross_extra_clocks;
        Ok(clocks)
    }

    /// Execute a concrete instruction
    pub fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), String> {
        let previous_cpu_status = self.cpu.clone();

        match instruction.instruction {
            SingleByte(fun) => {
                fun(&mut self.cpu);
            }
            InternalExecOnMemoryData(fun) => {
                let (_, data) = self.load(instruction.addressing_mode);
                fun(&mut self.cpu, data);
            }
            StoreOp(fun) => {
                let data = fun(&mut self.cpu);
                self.store(data, instruction.addressing_mode);
            }
            ReadModifyWrite(fun) => {
                let (_, data) = self.load(instruction.addressing_mode);
                let result = fun(&mut self.cpu, data);
                self.store(result, instruction.addressing_mode);
            }
            Misc(t) => match t {
                Push(fun) => {
                    fun(&mut self.cpu, &self.bus);
                }
                Pull(fun) => fun(&mut self.cpu, &self.bus),
                Jump(fun) => {
                    let (addr, _) = self.load(instruction.addressing_mode);
                    fun(&mut self.cpu, addr);
                }
                Branch(fun) => {
                    let (_, data) = self.load(instruction.addressing_mode);
                    fun(&mut self.cpu, data);
                    // when a branch is taken, a boolean is set to indicate
                    // whether page boundary is crossed. Extra clocks are added
                    // to the instruction execution
                    if let Some(page_crossed) = self.cpu.branch_crossed_page_boundary.take() {
                        self.page_boundary_cross_extra_clocks += if page_crossed { 2 } else { 1 }
                    }
                }
                Call(fun) => {
                    let (addr, _) = self.load(instruction.addressing_mode);
                    fun(&mut self.cpu, addr, &self.bus);
                }
                Return(fun) => {
                    fun(&mut self.cpu, &self.bus);
                }
                HardwareInterrupt(fun) => fun(&mut self.cpu, &self.bus),
                ReturnFromInterrupt(fun) => {
                    fun(&mut self.cpu, &self.bus);
                }
            },
        }

        // Increase PC
        match instruction.name {
            "JMP" | "JSR" | "RTS" | "BRK" | "RTI" => {}
            _ => self.cpu.pc += instruction.bytes as u16,
        }

        debug!(
            "CPU executed (PC: ${:0>4X} >> ${:0>4X}): \x1b[93m{}\x1b[0m (${:0>2X})| {}",
            previous_cpu_status.pc,
            self.cpu.pc,
            instruction.name,
            instruction.opcode,
            Self::status_diff(&previous_cpu_status, &self.cpu)
        );

        Ok(())
    }

    fn execute_interrupt(&mut self, interrupt: Interrupt) {
        let (lb, hb) = match interrupt {
            Interrupt::NonMaskableInterrupt => {
                // println!("CPU executing NMI");
                (0xFFFA, 0xFFFB)
            }
            Interrupt::Reset => (0xFFFC, 0xFFFD),
            Interrupt::InterruptRequest => {
                // IRQ is not executed if Interrupt disable flag is active
                if self.cpu.sr.get(InterruptDisable) {
                    return;
                }
                (0xFFFE, 0xFFFF)
            }
        };

        // Push PC and SR to stack
        let pch = ((self.cpu.pc & 0xFF00) >> 8) as u8;
        let pcl = (self.cpu.pc & 0x00FF) as u8;
        let sr: u8 = self.cpu.sr.into();
        instruction_set::push(&mut self.cpu, pch, &self.bus);
        instruction_set::push(&mut self.cpu, pcl, &self.bus);
        instruction_set::push(&mut self.cpu, sr, &self.bus);

        // Fetch interrupt vector address
        let pcl = self.bus_read(lb) as u16;
        let pch = self.bus_read(hb) as u16;

        // Go to interrupt handler
        self.cpu.pc = (pch << 8) | pcl;
    }

    fn load(&mut self, addr_mode: AddressingMode) -> (u16, u8) {
        let (addr, data) = match addr_mode {
            Implied => {
                let addr = self.cpu.pc + 1;
                let opcode = self.bus_read(self.cpu.pc);
                let data = opcode; // discarted
                (addr, data)
            }
            Accumulator => {
                let addr = self.cpu.pc + 1;
                let data = self.cpu.acc;
                (addr, data)
            }
            Immediate => {
                let addr = self.cpu.pc + 1;
                let data = self.bus_read(addr);
                (addr, data)
            }
            ZeroPage => {
                // Effective address is 00, ADL
                let adl = self.bus_read(self.cpu.pc + 1) as u16;
                let addr = adl;
                let data = self.bus_read(addr);
                (addr, data)
            }
            Absolute => {
                // Effective address is ADH, ADL
                let adl = self.bus_read(self.cpu.pc + 1) as u16;
                let adh = self.bus_read(self.cpu.pc + 2) as u16;
                let addr = (adh << 8) | adl;
                let data = self.bus_read(addr);
                (addr, data)
            }
            IndirectX => {
                // page zero base address
                let bal = self.bus_read(self.cpu.pc + 1) as u16;
                let adl = self.bus_read((bal + (self.cpu.x_reg as u16)) & 0x00FF) as u16;
                let adh = self.bus_read((bal + (self.cpu.x_reg as u16) + 1) & 0x00FF) as u16;
                let addr = (adh << 8) | adl;
                let data = self.bus_read(addr);
                (addr, data)
            }
            AbsoluteX => {
                let bal = self.bus_read(self.cpu.pc + 1) as u16;
                let bah = self.bus_read(self.cpu.pc + 2) as u16;
                let addr = ((bah << 8) | bal) + self.cpu.x_reg as u16;
                if (addr & 0xFF00) >> 8 != bah {
                    self.cpu.page_boundary_crossed = true;
                }
                let data = self.bus_read(addr);
                (addr, data)
            }
            AbsoluteY => {
                let bal = self.bus_read(self.cpu.pc + 1) as u16;
                let bah = self.bus_read(self.cpu.pc + 2) as u16;
                // ignore overflow while computing address
                let addr = (((bah << 8) | bal) as u32 + self.cpu.y_reg as u32) as u16;
                if (addr & 0xFF00) >> 8 != bah {
                    self.cpu.page_boundary_crossed = true;
                }
                let data = self.bus_read(addr);
                (addr, data)
            }
            ZeroPageX => {
                let bal = self.bus_read(self.cpu.pc + 1) as u16;
                // Zero page indexing can't cross page boundaries
                let addr = (bal + (self.cpu.x_reg as u16)) & 0x00FF;
                let data = self.bus_read(addr);
                (addr, data)
            }
            ZeroPageY => {
                let bal = self.bus_read(self.cpu.pc + 1) as u16;
                // Zero page indexing can't cross page boundaries
                let addr = (bal + (self.cpu.y_reg as u16)) & 0x00FF;
                let data = self.bus_read(addr);
                (addr, data)
            }
            IndirectY => {
                let ial = self.bus_read(self.cpu.pc + 1) as u16;
                let bal = self.bus_read(ial) as u16;
                let bah = self.bus_read((ial + 1) & 0x00FF) as u16;
                let base_addr = (bah << 8) | bal;
                // ignore overflow while computing address
                let addr = (base_addr as u32 + self.cpu.y_reg as u32) as u16;
                let data = self.bus_read(addr);
                // Hardware CPU behaviour would be doing a fetch of the wrong
                // address and then another for the correct page. We don't need
                // that as we are directly fetching the correct address
                self.cpu.page_boundary_crossed = (addr & 0xFF00) != (bah << 8);
                (addr, data)
            }
            Relative => {
                let offset = self.bus_read(self.cpu.pc + 1) as i8 as u8;
                (self.cpu.pc + 2, offset)
            }
            Indirect => {
                let ind_l = self.bus_read(self.cpu.pc + 1) as u16;
                let ind_h = self.bus_read(self.cpu.pc + 2) as u16;
                let addr_l = self.bus_read((ind_h << 8) | ind_l) as u16;
                let addr_h = self.bus_read((ind_h << 8) | ((ind_l + 1) & 0x00FF)) as u16;
                let address = (addr_h << 8) | addr_l;

                (address, 0)
            }
        };
        (addr, data)
    }

    fn store(&mut self, data: u8, addr_mode: AddressingMode) {
        let addr = match addr_mode {
            ZeroPage => self.bus_read(self.cpu.pc + 1) as u16,
            Absolute => {
                let adl = self.bus_read(self.cpu.pc + 1) as u16;
                let adh = self.bus_read(self.cpu.pc + 2) as u16;
                (adh << 8) | adl
            }
            IndirectX => {
                let bal = self.bus_read(self.cpu.pc + 1) as u16;
                let adl = self.bus_read((bal + (self.cpu.x_reg as u16)) & 0x00FF) as u16;
                let adh = self.bus_read((bal + (self.cpu.x_reg as u16) + 1) & 0x00FF) as u16;
                (adh << 8) | adl
            }
            AbsoluteX => {
                let bal = self.bus_read(self.cpu.pc + 1) as u16;
                let bah = self.bus_read(self.cpu.pc + 2) as u16;
                let addr = ((bah << 8) | bal) + (self.cpu.x_reg as u16);
                if (addr & 0xFF00) >> 8 != bah {
                    self.cpu.page_boundary_crossed = true;
                }
                addr
            }
            AbsoluteY => {
                let bal = self.bus_read(self.cpu.pc + 1) as u16;
                let bah = self.bus_read(self.cpu.pc + 2) as u16;
                let addr = ((bah << 8) | bal) + (self.cpu.y_reg as u16);
                if (addr & 0xFF00) >> 8 != bah {
                    self.cpu.page_boundary_crossed = true;
                }
                addr
            }
            ZeroPageX => {
                let bal = self.bus_read(self.cpu.pc + 1) as u16;
                (bal + (self.cpu.x_reg as u16)) & 0x00FF
            }
            ZeroPageY => {
                let bal = self.bus_read(self.cpu.pc + 1) as u16;
                (bal + (self.cpu.y_reg as u16)) & 0x00FF
            }
            IndirectY => {
                let ial = self.bus_read(self.cpu.pc + 1) as u16;
                let bal = self.bus_read(ial) as u16;
                let bah = self.bus_read((ial + 1) & 0x00FF) as u16;
                let base_addr = (bah << 8) | bal;
                let addr = base_addr + self.cpu.y_reg as u16;
                self.cpu.page_boundary_crossed = (addr & 0xFF00) != (bah << 8);
                addr
            }
            _ => {
                panic!("Invalid store addressing mode: {addr_mode:?}");
            }
        };
        self.bus_write(addr, data);
    }

    fn fetch(&self) -> Result<Instruction, String> {
        let opcode = self.bus_read(self.cpu.pc);
        let instruction = self.instruction_set.lookup(opcode).ok_or(format!(
            "Invalid instruction 0x{:0>2X} at PC 0x{:0>4X}",
            opcode, self.cpu.pc
        ))?;
        Ok(instruction)
    }

    fn bus_read(&self, address: u16) -> u8 {
        self.bus.borrow().read(address)
    }

    fn bus_write(&self, address: u16, data: u8) {
        self.bus.borrow().write(address, data);
    }

    fn status_diff(previous: &InternalCpu, current: &InternalCpu) -> String {
        let registers = format!(
            "A: ${:0>2X} >> ${:0>2X}  X: ${:0>2X} >> ${:0>2X}  Y: ${:0>2X} >> ${:0>2X}  SP: ${:0>2X} >> ${:0>2X}",
            previous.acc,
            current.acc,
            previous.x_reg,
            current.x_reg,
            previous.y_reg,
            current.y_reg,
            previous.sp,
            current.sp,
        );
        let status_register = format!(
            // "SR: (N: {}>{} Z: {}>{} C: {}>{} ...)",
            "SR: (N: {}>{} Z: {}>{} C: {}>{} I: {}>{} D: {}>{} V: {}>{} B: {}>{})",
            if previous.sr.get(Negative) { "1" } else { "0" },
            if current.sr.get(Negative) { "1" } else { "0" },
            if previous.sr.get(Zero) { "1" } else { "0" },
            if current.sr.get(Zero) { "1" } else { "0" },
            if previous.sr.get(Carry) { "1" } else { "0" },
            if current.sr.get(Carry) { "1" } else { "0" },
            if previous.sr.get(InterruptDisable) {
                "1"
            } else {
                "0"
            },
            if current.sr.get(InterruptDisable) {
                "1"
            } else {
                "0"
            },
            if previous.sr.get(Decimal) { "1" } else { "0" },
            if current.sr.get(Decimal) { "1" } else { "0" },
            if previous.sr.get(Overflow) { "1" } else { "0" },
            if current.sr.get(Overflow) { "1" } else { "0" },
            if previous.sr.get(Break) { "1" } else { "0" },
            if current.sr.get(Break) { "1" } else { "0" },
        );
        format!("{registers}  {status_register}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::interfaces::{AddressRange, LoadableMemory};
    use crate::processor::bus::Bus;
    use crate::processor::memory::Ram;

    fn cpu_with_program(program: Vec<u8>) -> Cpu {
        // env_logger::builder()
        //     .is_test(true)
        //     .try_init()
        //     .unwrap_or_default();

        let bus = Rc::new(RefCell::new(Bus::new("test-bus")));
        let bus_ptr = Rc::clone(&bus);

        let cpu = Cpu::new(bus_ptr);

        let memory = Rc::new(RefCell::new(Ram::new(0xFFFF + 1)));
        memory.borrow_mut().load(0, &program);

        let memory_ptr = Rc::clone(&memory);
        bus.borrow_mut()
            .attach(
                "Test Memory",
                memory_ptr,
                AddressRange {
                    start: 0,
                    end: 0xFFFF,
                },
            )
            .unwrap();

        cpu
    }

    #[test]
    fn test_program_multiply_by_10() {
        let program = vec![
            0x0A, // ASL - A << 1 = A x2
            0x85, 0xFF, // STA 0 - store in 0x00
            0x0A, // ASL - A << 1 = A x2
            0x0A, // ASL - A << 1 = A x2
            0x18, // CLC - delete carry
            0x65, 0xFF, // ADC r0 - A = x*8 + x*2
        ];
        let program_size = 6;
        let mut cpu = cpu_with_program(program);

        let value = 4;
        cpu.cpu.acc = value;
        for _ in 0..program_size {
            cpu.execute().unwrap();
        }

        assert_eq!(cpu.cpu.acc, value * 10);
    }
}
