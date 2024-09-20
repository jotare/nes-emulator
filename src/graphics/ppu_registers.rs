//! PPU registers
//!
//! This module provides a better way to manage PPU register bits and bit
//! groups

use bitflags::bitflags;

pub struct PpuRegisters {
    pub ctrl: PpuCtrl,
    pub mask: PpuMask,
    pub status: PpuStatus,
    pub oam_addr: u8,
    pub data_buffer: u8,
}

impl Default for PpuRegisters {
    fn default() -> Self {
        Self {
            ctrl: PpuCtrl::empty(),
            mask: PpuMask::empty(),
            status: PpuStatus::empty(),
            oam_addr: 0,
            data_buffer: 0,
        }
    }
}

impl PpuRegisters {
    pub fn reset(&mut self) {
        self.ctrl = PpuCtrl::empty();
        self.mask = PpuMask::empty();
        self.status = PpuStatus::empty();
        self.data_buffer = 0;
    }

    // PPUCTRL

    #[inline]
    pub fn nmi_enabled(&self) -> bool {
        self.ctrl.contains(PpuCtrl::NMI_ENABLE)
    }

    #[inline]
    pub fn background_pattern_table(&self) -> u8 {
        self.ctrl
            .intersection(PpuCtrl::BACKGROUND_PATTERN_TABLE)
            .bits()
            >> PpuCtrl::BACKGROUND_PATTERN_TABLE.bits().trailing_zeros()
    }

    #[inline]
    pub fn vram_address_increment(&self) -> usize {
        match self.ctrl.contains(PpuCtrl::VRAM_ADDRESS_INCREMENT) {
            false => 1, // going across
            true => 32, // going down
        }
    }

    // PPUMASK

    #[inline]
    pub fn rendering_enabled(&self) -> bool {
        self.background_rendering_enabled() || self.sprite_rendering_enabled()
    }

    #[inline]
    pub fn background_rendering_enabled(&self) -> bool {
        self.mask.contains(PpuMask::BACKGROUND_RENDERING_ENABLE)
    }

    #[inline]
    pub fn sprite_rendering_enabled(&self) -> bool {
        self.mask.contains(PpuMask::SPRITE_RENDERING_ENABLED)
    }

    // PPUSTATUS

    #[inline]
    pub fn set_vertical_blank(&mut self) {
        self.status.set(PpuStatus::VERTICAL_BLANK, true);
    }

    #[inline]
    pub fn unset_vertical_blank(&mut self) {
        self.status.set(PpuStatus::VERTICAL_BLANK, false);
    }
}

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
