//! PPU registers
//!
//! This module provides a better way to manage PPU register bits and bit
//! groups

use bitflags::bitflags;

bitflags! {
    pub struct PpuCtrl: u8 {
        /// Generate an NMI at the start of the vertical blanking interval
        const NMI_ENABLE = 0b1000_0000;

        // TODO: PPU master/slave select

        /// 0: 8x8 pixels; 1: 8x16 pixels
        const SPRITE_SIZE = 0b0010_0000;

        /// Background pattern table address (0 = $0000; 1 = $1000)
        const BACKGROUND_PATTERN_TABLE = 0b0001_0000;

        /// Sprite pattern table address for 8x8 sprites (0: $0000; 1: $1000;
        /// ignored in 8x16 mode)
        const SPRITE_PATTERN_TABLE_ADDRESS = 0b0000_1000;

        /// VRAM address increment per CPU read/write of PPUDATA (0: add 1,
        /// going across; 1: add 32, going down)
        const VRAM_ADDRESS_INCREMENT = 0b0000_0100;

        /// Base nametable address (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
        const BASENAME_NAMETABLE_ADDRESS = 0b0000_0011;
    }
}

bitflags! {
    pub struct PpuMask: u8 {
        const BACKGROUND_RENDERING_ENABLE = 0b0000_1000;

        const SPRITE_RENDERING_ENABLED = 0b0001_0000;

        // TODO
    }
}

bitflags! {
    pub struct PpuStatus: u8 {
        // PPU is in VBL status
        const VERTICAL_BLANK = 0b1000_0000;
    }
}
