use crate::hardware::PALETTE_MEMORY_SIZE;
use crate::interfaces::Memory;
use crate::processor::memory::Ram;

pub struct PaletteMemory {
    memory: Ram,
}

impl PaletteMemory {
    pub fn new() -> Self {
        Self {
            memory: Ram::new(PALETTE_MEMORY_SIZE.into()),
        }
    }
}

impl Memory for PaletteMemory {
    fn read(&self, address: u16) -> u8 {
        let address = match address {
            0x10 => 0x00,
            0x14 => 0x04,
            0x18 => 0x08,
            0x1C => 0x0C,
            address => address,
        };

        self.memory.read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        let address = match address {
            0x10 => 0x00,
            0x14 => 0x04,
            0x18 => 0x08,
            0x1C => 0x0C,
            address => address,
        };

        self.memory.write(address, data);
    }

    fn size(&self) -> usize {
        PALETTE_MEMORY_SIZE.into()
    }
}
