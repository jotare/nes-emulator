//! Object Attribute Memory (OAM)
//!
//! TODO docs

use crate::interfaces::Memory;
use crate::processor::memory::Ram;

pub struct Oam {
    memory: Ram,
}

#[derive(Copy, Clone, Debug)]
pub struct OamSprite {
    pub x: u8,
    pub y: u8,
    pub tile: u8,
    pub attributes: u8,
}

impl Oam {
    pub fn new() -> Self {
        Self {
            memory: Ram::new(64 * 4), // 64 sprites, 4 bytes each
        }
    }

    pub fn read_sprite(&self, sprite: u8) -> OamSprite {
        assert!(sprite < 64, "OAM only contains 64 sprites of 4 bytes each");

        let base_addr = (sprite as u16) << 2;

        let y = self.memory.read(base_addr | 0b00);
        let tile = self.memory.read(base_addr | 0b01);
        let attributes = self.memory.read(base_addr | 0b10);
        let x = self.memory.read(base_addr | 0b11);

        OamSprite {
            y,
            tile,
            attributes,
            x,
        }
    }
}

impl std::fmt::Debug for Oam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..64 {
            let sprite = self.read_sprite(i);
            sprite.fmt(f)?;
            f.write_str("\n")?;
        }
        Ok(())
    }
}

impl Memory for Oam {
    fn read(&self, address: u16) -> u8 {
        self.memory.read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        self.memory.write(address, data);
    }

    fn size(&self) -> usize {
        self.memory.size()
    }
}
