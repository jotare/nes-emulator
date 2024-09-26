//! PPU hardware to produce pixel data, mixing background and sprites
//!
//! Refer to https://www.nesdev.org/wiki/PPU_rendering for more information
//! about this module

use crate::interfaces::Bus;
use crate::{hardware::PALETTE_MEMORY_START, types::SharedBus, utils};

use super::pattern_table::PatternTableAddress;
use super::{oam::OamSprite, Pixel};

/// PPU's internal set of shift registers and multiplexers responsible of
/// producing pixel data.
///
/// In consists in shifters for background tile pattern and attributes as well
/// as sprite information.
///
/// Priority multiplexers decide how to combine all data to produce the correct
/// pixel.
///
pub struct PixelProducer {
    bus: SharedBus,

    // Background
    pub fine_x: u8,
    pub buffers: Buffers,
    pub shifters: Shifters,

    // Sprites
    /// Up to 8 sprites used for a single scanline
    pub sprites: [OamSprite; 8],

    pub sprite_pattern_table: u8,
}

/// XXX TODO
///
/// Internal PPU latches that store temporary information while rendering
#[derive(Default)]
pub struct Buffers {
    pub next_tile_number: u8,
    pub next_attributes: u8,
    pub next_bit_plane_high: u8,
    pub next_bit_plane_low: u8,
}

/// XXX TODO
///
/// Internal PPU shift registers responsible of producing pixel data.
///
/// Shifters are 16-bit wide, the high 8 bits are used in the current pixels
/// being drawn while the low 8 bits will be used for the next tile
#[derive(Default)]
pub struct Shifters {
    pub attributes: (u16, u16),
    pub tile_pattern: (u16, u16),
}

impl PixelProducer {
    pub fn new(bus: SharedBus) -> Self {
        Self {
            bus,
            fine_x: 0,
            sprite_pattern_table: 0,
            buffers: Buffers::default(),
            shifters: Shifters::default(),
            sprites: [OamSprite {
                x: 0xFF,
                y: 0xFF,
                tile: 0xFF,
                attributes: 0xFF,
            }; 8],
        }
    }

    // Load shift registers from internal latches (buffers) so next 8 pixels can
    // be drawn by the PPU in the next clock cycles
    pub fn load_shifters(&mut self) {
        self.shifters.tile_pattern.0 =
            (self.shifters.tile_pattern.0 & 0xFF00) | (self.buffers.next_bit_plane_low as u16);
        self.shifters.tile_pattern.1 =
            (self.shifters.tile_pattern.1 & 0xFF00) | (self.buffers.next_bit_plane_high as u16);

        let attributes_0 = if utils::bv(self.buffers.next_attributes, 0) == 0 {
            0
        } else {
            0xFF
        };
        self.shifters.attributes.0 = (self.shifters.attributes.0 & 0xFF00) | attributes_0 as u16;

        let attributes_1 = if utils::bv(self.buffers.next_attributes, 1) == 0 {
            0
        } else {
            0xFF
        };
        self.shifters.attributes.1 = (self.shifters.attributes.1 & 0xFF00) | attributes_1 as u16;
    }

    pub fn update_shifters(&mut self) {
        self.shifters.tile_pattern.0 = self.shifters.tile_pattern.0 << 1;
        self.shifters.tile_pattern.1 = self.shifters.tile_pattern.1 << 1;
        self.shifters.attributes.0 = self.shifters.attributes.0 << 1;
        self.shifters.attributes.1 = self.shifters.attributes.1 << 1;
    }

    pub fn produce_pixel(&mut self, col: usize, row: usize) -> Option<Pixel> {
        if col >= 256 || row >= 240 {
            return None;
        }

        // Background

        let fine_x_bit = 15 - self.fine_x;

        let background_palette = {
            let palette_lo = utils::bv_16(self.shifters.attributes.0, fine_x_bit);
            let palette_hi = utils::bv_16(self.shifters.attributes.1, fine_x_bit);
            (palette_hi << 1) | palette_lo
        };
        let background_bit_plane = {
            let bit_plane_lo = utils::bv_16(self.shifters.tile_pattern.0, fine_x_bit);
            let bit_plane_hi = utils::bv_16(self.shifters.tile_pattern.1, fine_x_bit);
            (bit_plane_hi << 1) | bit_plane_lo
        };

        // ----------------------------------------------------------------------------------------------------
        let mut palette_offset = (background_palette << 2) | background_bit_plane;

        // Sprites

        for sprite in self.sprites.iter_mut() {
            // no more valid sprites
            if sprite.y == 0xFF {
                break;
            }

            if col < (sprite.x as usize) || col >= (sprite.x as usize + 8) {
                continue;
            }

            let mut pattern_table_address = PatternTableAddress::new(self.sprite_pattern_table);
            pattern_table_address.set(PatternTableAddress::TILE_NUMBER, sprite.tile);

            let sprite_palette = (sprite.attributes & 0b0000_0011) + 4; // sprite palettes are 4 to 7

            // 0 -> front of background, 1 -> behind background
            let priority = utils::bv(sprite.attributes, 5);
            let flip_horizontally = utils::bv(sprite.attributes, 6) > 0;
            let flip_vertically = utils::bv(sprite.attributes, 7) > 0;

            // sprites are rendered with 1 scan line offset, we need to
            // substract it from the row to place it in the correct position
            let mut y = (row - 1 - sprite.y as usize) as u8;
            if flip_vertically {
                y = 7 - y;
            }

            pattern_table_address.set(PatternTableAddress::FINE_Y_OFFSET, y);

            pattern_table_address.set(PatternTableAddress::BIT_PLANE, 0);
            let low = self.bus.borrow().read(pattern_table_address.into());

            pattern_table_address.set(PatternTableAddress::BIT_PLANE, 1);
            let high = self.bus.borrow().read(pattern_table_address.into());

            let mut x = (7 - (col - sprite.x as usize)) as u8;
            if flip_horizontally {
                x = 7 - x
            }

            let sprite_bit_plane = utils::bv(high, x as u8) << 1 | utils::bv(low, x as u8);

            if background_bit_plane == 0 && sprite_bit_plane == 0 {
                // EXT in $3F00
                palette_offset = 0;
            } else if background_bit_plane == 0 && sprite_bit_plane > 0 {
                // paint sprite
                palette_offset = ((sprite_palette << 2) | sprite_bit_plane) as u16;
                break;
            } else if background_bit_plane > 0 && sprite_bit_plane == 0 {
                // paint background
            } else {
                if priority == 0 {
                    // paint sprite
                    palette_offset = ((sprite_palette << 2) | sprite_bit_plane) as u16;
                    break;
                } else {
                    // paint background
                }
            }
        }

        let color = Pixel::from(
            self.bus
                .borrow()
                .read(PALETTE_MEMORY_START + palette_offset),
        );

        Some(color)
    }
}
