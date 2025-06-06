//! PPU registers
//!
//! This module provides a better way to manage PPU register bits and bit
//! groups

use std::cell::Cell;

use bitflags::bitflags;

pub struct PpuRegisters {
    pub ctrl: PpuCtrl,
    pub mask: PpuMask,
    pub status: Cell<PpuStatus>,
    pub oam_addr: u8,
    pub data_buffer: Cell<u8>,
}

impl Default for PpuRegisters {
    fn default() -> Self {
        Self {
            ctrl: PpuCtrl::empty(),
            mask: PpuMask::empty(),
            status: Cell::new(PpuStatus::empty()),
            oam_addr: 0,
            data_buffer: Cell::new(0),
        }
    }
}

impl PpuRegisters {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    // PPUCTRL

    #[inline]
    pub fn nmi_enabled(&self) -> bool {
        self.ctrl.contains(PpuCtrl::NMI_ENABLE)
    }

    #[inline]
    pub fn sprite_size(&self) -> u8 {
        if self.ctrl.contains(PpuCtrl::SPRITE_SIZE) {
            16
        } else {
            8
        }
    }

    #[inline]
    pub fn background_pattern_table(&self) -> u8 {
        self.ctrl
            .intersection(PpuCtrl::BACKGROUND_PATTERN_TABLE)
            .bits()
            >> PpuCtrl::BACKGROUND_PATTERN_TABLE.bits().trailing_zeros()
    }

    #[inline]
    pub fn sprite_pattern_table(&self) -> u8 {
        self.ctrl.intersection(PpuCtrl::SPRITE_PATTERN_TABLE).bits()
            >> PpuCtrl::SPRITE_PATTERN_TABLE.bits().trailing_zeros()
    }

    #[inline]
    pub fn vram_address_increment(&self) -> u16 {
        match self.ctrl.contains(PpuCtrl::VRAM_ADDRESS_INCREMENT) {
            false => 1, // going across
            true => 32, // going down
        }
    }

    // PPUMASK

    #[inline]
    pub fn left_size_clipping_window_enabled(&self) -> bool {
        !self
            .mask
            .contains(PpuMask::SHOW_BACKGROUND_IN_LEFTMOST_8_PIXELS)
            || !self
                .mask
                .contains(PpuMask::SHOW_SPRITES_IN_LEFTMOST_8_PIXELS)
    }

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
    pub fn set_sprite_overflow(&self, value: bool) {
        let mut status = self.status.get();
        status.set(PpuStatus::SPRITE_OVERFLOW, value);
        self.status.set(status);
    }

    #[inline]
    pub fn set_sprite_0_hit(&self, value: bool) {
        let mut status = self.status.get();
        status.set(PpuStatus::SPRITE_0_HIT, value);
        self.status.set(status);
    }

    #[inline]
    pub fn set_vertical_blank(&self) {
        let mut status = self.status.get();
        status.set(PpuStatus::VERTICAL_BLANK, true);
        self.status.set(status);
    }

    #[inline]
    pub fn unset_vertical_blank(&self) {
        let mut status = self.status.get();
        status.set(PpuStatus::VERTICAL_BLANK, false);
        self.status.set(status);
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
        const SPRITE_PATTERN_TABLE = 0b0000_1000;

        /// VRAM address increment per CPU read/write of PPUDATA (0: add 1,
        /// going across; 1: add 32, going down)
        const VRAM_ADDRESS_INCREMENT = 0b0000_0100;

        /// Base nametable address (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
        const BASENAME_NAMETABLE_ADDRESS = 0b0000_0011;
    }
}

bitflags! {
    pub struct PpuMask: u8 {
        const SHOW_BACKGROUND_IN_LEFTMOST_8_PIXELS = 0b0000_0010;

        const SHOW_SPRITES_IN_LEFTMOST_8_PIXELS = 0b0000_0100;

        const BACKGROUND_RENDERING_ENABLE = 0b0000_1000;

        const SPRITE_RENDERING_ENABLED = 0b0001_0000;

        // TODO
    }
}

bitflags! {
    pub struct PpuStatus: u8 {
        /// PPU is in vertical blank (VBL) status
        const VERTICAL_BLANK = 0b1000_0000;

        const SPRITE_0_HIT = 0b0100_0000;

        /// Sprite overflow is active whenever more than 8 sprites appear on a
        /// scanline. The real NES had a hardware bug that generate false
        /// positives and negatives
        const SPRITE_OVERFLOW = 0b0010_0000;
    }
}
