use std::cell::RefCell;
use std::rc::Rc;

use crate::interfaces::LoadableMemory;
use crate::processor::memory::{MirroredMemory, Ram, Rom};
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
    pub character_ram: bool,
}

pub struct Mapper0 {
    // Program memory (RAM)
    program_ram: SharedRam,

    // Program memory (ROM)
    program_rom: SharedMirroredRom,

    // Character memory, stores patterns and graphics for the PPU
    character_memory: SharedRam,
}


enum CharacterMemory {
    Rom(SharedRom),
    Ram(SharedRam),
}

impl Mapper0 {
    pub fn new(specs: MapperSpecs) -> Self {
        let program_rom = match specs.program_rom_capacity {
            16384 => Rc::new(RefCell::new(MirroredMemory::new(
                Rom::new(specs.program_rom_capacity),
                1,
            ))),
            32768 => Rc::new(RefCell::new(MirroredMemory::new(
                Rom::new(specs.program_rom_capacity),
                0,
            ))),
            _ => panic!(
                "Unexpected PGR ROM capacity: {}",
                specs.program_rom_capacity
            ),
        };

        let character_memory = if specs.character_ram {
            CharacterMemory::Ram(Rc::new(RefCell::new(Ram::new(CHR_MEMORY_SIZE as usize))))
        } else {
            // CharacterMemory::Rom(Rc::new(RefCell::new(Rom::new(
            CharacterMemory::Ram(Rc::new(RefCell::new(Ram::new(
                specs.character_rom_capacity,
            ))))
        };

        Self {
            program_rom,
            program_ram: Rc::new(RefCell::new(Ram::new(specs.program_ram_capacity))),
            character_memory,
        }
    }
}

impl Mapper for Mapper0 {
    fn load_program_rom(&mut self, data: &[u8]) {
        self.program_rom.borrow_mut().load(0, data);
    }
    fn load_character_memory(&mut self, data: &[u8]) {
        match self.character_memory {
            CharacterMemory::Rom(ref memory) => memory.borrow_mut().load(0, data),
            CharacterMemory::Ram(ref memory) => memory.borrow_mut().load(0, data),
        };
    }

    fn program_ram_ref(&self) -> SharedMemory {
        Rc::clone(&self.program_ram) as _
    }

    fn program_rom_ref(&self) -> SharedMemory {
        Rc::clone(&self.program_rom) as _
    }

    fn character_memory_ref(&self) -> SharedMemory {
        match self.character_memory {
            CharacterMemory::Rom(ref memory) => Rc::clone(memory) as _,
            CharacterMemory::Ram(ref memory) => Rc::clone(memory) as _,
        }
    }
}
