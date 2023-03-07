use std::cell::RefCell;
use std::rc::Rc;

use crate::processor::memory::{MirroredRom, Ram};
use crate::types::{SharedMemory, SharedMirroredRom, SharedRam};

pub trait Mapper {
    fn load_program_rom(&mut self, data: &[u8]);
    fn load_character_memory(&mut self, data: &[u8]);

    fn program_ram_ref(&self) -> SharedMemory;
    fn program_rom_ref(&self) -> SharedMemory;
    fn character_memory_ref(&self) -> SharedMemory;
}

pub fn mapper_map(mapper: u8, specs: MapperSpecs) -> Box<dyn Mapper> {
    Box::new(match mapper {
        0 => Mapper0::new(specs),
        _ => panic!("Mapper {mapper} not implemented"),
    })
}

pub struct MapperSpecs {
    pub program_rom_capacity: usize,
    pub program_ram_capacity: usize,
    pub character_memory_capacity: usize,
}

pub struct Mapper0 {
    // Program memory (RAM)
    program_ram: SharedRam,

    // Program memory (ROM)
    program_rom: SharedMirroredRom,

    // Character memory, stores patterns and graphics for the PPU
    character_memory: SharedRam,
}

impl Mapper0 {
    pub fn new(specs: MapperSpecs) -> Self {
        let program_rom = match specs.program_rom_capacity {
            16384 => Rc::new(RefCell::new(MirroredRom::new(
                specs.program_rom_capacity,
                1,
            ))),
            32768 => Rc::new(RefCell::new(MirroredRom::new(
                specs.program_rom_capacity,
                0,
            ))),
            _ => panic!(
                "Unexpected PGR ROM capacity: {}",
                specs.program_rom_capacity
            ),
        };

        Self {
            program_rom,
            program_ram: Rc::new(RefCell::new(Ram::new(specs.program_ram_capacity))),
            character_memory: Rc::new(RefCell::new(Ram::new(specs.character_memory_capacity))),
        }
    }
}

impl Mapper for Mapper0 {
    fn load_program_rom(&mut self, data: &[u8]) {
        self.program_rom.borrow_mut().load(0, data);
    }
    fn load_character_memory(&mut self, data: &[u8]) {
        self.character_memory.borrow_mut().load(0, data);
    }

    fn program_ram_ref(&self) -> SharedMemory {
        let shared = Rc::clone(&self.program_ram);
        shared
    }

    fn program_rom_ref(&self) -> SharedMemory {
        let shared = Rc::clone(&self.program_rom);
        shared
    }

    fn character_memory_ref(&self) -> SharedMemory {
        let shared = Rc::clone(&self.character_memory);
        shared
    }
}
