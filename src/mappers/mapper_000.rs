use std::cell::RefCell;
use std::rc::Rc;

use crate::hardware::{
    CARTIDGE_RAM_END, CARTIDGE_RAM_START, CARTIDGE_ROM_END, CARTIDGE_ROM_START, CHR_MEMORY_END,
    CHR_MEMORY_SIZE, CHR_MEMORY_START,
};
use crate::interfaces::Bus;
use crate::interfaces::{AddressRange, LoadableMemory};
use crate::mappers::{Mapper, MapperSpecs};
use crate::processor::memory::{MirroredMemory, Ram, Rom};
use crate::types::{SharedBus, SharedGraphicsBus};
use crate::types::{SharedMirroredRom, SharedRam, SharedRom};

const CARTRIDGE_ROM_ID: &'static str = "Cartridge ROM";
const CARTRIDGE_RAM_ID: &'static str = "Cartridge RAM";
const CARTRIDGE_CHR_MEM_ID: &'static str = "Cartridge CHR memory (pattern tables)";

pub struct Mapper0 {
    // Program memory (RAM) -- Attached to CPU address bus $6000 - $7FFF
    program_ram: SharedRam,

    // Program memory (ROM) -- Attached to CPU address bus $8000 - $FFFF
    program_rom: SharedMirroredRom,

    // Character memory, stores patterns and graphics for the PPU -- Attached to
    // PPU address bus $0000-$1FFF (used for pattern tables)
    character_memory: CharacterMemory,
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
    fn load_program_memory(&mut self, data: Vec<u8>) {
        self.program_rom.borrow_mut().load(0, &data);
    }

    fn load_character_memory(&mut self, data: Vec<u8>) {
        match self.character_memory {
            CharacterMemory::Rom(ref memory) => memory.borrow_mut().load(0, &data),
            CharacterMemory::Ram(ref memory) => memory.borrow_mut().load(0, &data),
        };
    }

    fn connect(&self, main_bus: &SharedBus, graphics_bus: &SharedGraphicsBus) {
        let ram = Rc::clone(&self.program_ram);
        let rom = Rc::clone(&self.program_rom);
        let chr = match self.character_memory {
            CharacterMemory::Rom(ref memory) => Rc::clone(memory) as _,
            CharacterMemory::Ram(ref memory) => Rc::clone(memory) as _,
        };

        main_bus
            .borrow_mut()
            .attach(
                CARTRIDGE_RAM_ID,
                ram,
                AddressRange {
                    start: CARTIDGE_RAM_START,
                    end: CARTIDGE_RAM_END,
                },
            )
            .unwrap();

        main_bus
            .borrow_mut()
            .attach(
                CARTRIDGE_ROM_ID,
                rom,
                AddressRange {
                    start: CARTIDGE_ROM_START,
                    end: CARTIDGE_ROM_END,
                },
            )
            .unwrap();

        // Pattern memory - also known as CHR ROM is a 8 kB memory where two
        // pattern tables are stored. It contains all graphical information the
        // PPU require to draw.
        //
        // It can be split into two 4 kB (0x1000) sections containing the
        // pattern tables 0 and 1
        graphics_bus.borrow_mut().connect_cartridge(
            chr,
            AddressRange {
                start: CHR_MEMORY_START,
                end: CHR_MEMORY_END,
            },
        );
    }

    fn disconnect(&self, main_bus: &SharedBus, graphics_bus: &SharedGraphicsBus) {
        todo!("Not needed until ejection of cartridges is implemented")
    }
}
