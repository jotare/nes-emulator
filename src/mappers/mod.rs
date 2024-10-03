//! Mappers
//!
//! NES mappers are circuits and hardware found in cartridges that allow
//! extending the capabilities and bypass some NES limitations.
//!
//! They are commonly used to extend the NES memory limitations but they can
//! also add more RAM or even extend sound channels.
//!

mod mapper_000;

use crate::types::SharedBus;
use mapper_000::Mapper0;

pub struct MapperSpecs {
    pub program_rom_capacity: usize,
    pub program_ram_capacity: usize,
    pub character_rom_capacity: usize,
    pub character_ram: bool,
}

pub trait Mapper {
    // Startup operations

    /// Load PGR memory from the iNES ROM file to the mapper
    fn load_program_memory(&mut self, data: Vec<u8>);

    /// Load CHR memory from the iNES ROM file to the mapper
    fn load_character_memory(&mut self, data: Vec<u8>);

    // Cartridge insertion and ejection

    /// Attach mapper memories to NES buses
    fn connect(&self, main_bus: &SharedBus, graphics_bus: &SharedBus);

    /// Detach mapper memories to NES buses
    fn disconnect(&self, main_bus: &SharedBus, graphics_bus: &SharedBus);
}

pub fn mapper_map(mapper: u8, specs: MapperSpecs) -> Box<dyn Mapper> {
    Box::new(match mapper {
        0 => Mapper0::new(specs),
        _ => panic!("Mapper {mapper} not implemented"),
    })
}
