//! PPU module
//!
//! This module emulates the NES Picture Processing Unit (PPU) integrated
//! circuit: the 2C02
//!
//! NES PPU registers ($2000-$2007) are mirrored from $2008 to $3FFF. That's
//! because it's address is not completely decoded, that is, the chip ignores
//! one or more address lines. This allows a cheaper hardware (less address
//! lines) and a faster decoding at expense of unused address space.
//!
//! # How PPU rendering works?
//!
//! The PPU is connected to the graphics bus. On it, we found:
//! - Pattern memory (CHR ROM)
//! - Nametable memory (VRAM)
//! - Palette memory
//!
//!
//! ## Pattern memory
//!
//! The NES has 2 pattern 4kB pattern tables (they can be thought as tilemaps).
//! Each pattern memory is split in a 16x16 8-pixel tiles. The PPU can choose
//! tiles from both pattern tables to compose backgrounds and sprites.
//!
//! Usually, sprites are composed by various tiles (as in SMB clouds and bushes)
//!
//! Tiles are stored in two consecutive 8-byte bit planes. Each pixel is defined
//! with 2 bits that points to an specific color in a palette.
//!
//!
//! ## Nametable memory
//!
//!
//! ## Palette memory
//!
//!

use std::cell::Cell;
use std::cell::RefCell;
use std::io::Write;

use log::{debug, trace};

use crate::events::Event;
use crate::events::SharedEventBus;
use crate::graphics::oam::{Oam, OamSprite};
use crate::graphics::pattern_table::PatternTableAddress;
use crate::graphics::ppu_registers::PpuRegisters;
use crate::graphics::ppu_registers::{PpuCtrl, PpuMask};
use crate::graphics::render_address::RenderAddress;
use crate::graphics::Frame;
use crate::graphics::FramePixel;
use crate::graphics::Pixel;
use crate::hardware::OAMDATA;
use crate::hardware::PALETTE_MEMORY_START;
use crate::hardware::{OAMADDR, PPUADDR, PPUCTRL, PPUDATA, PPUMASK, PPUSCROLL, PPUSTATUS};
use crate::interfaces::{Bus, Memory};
use crate::types::SharedGraphicsBus;
use crate::utils;

// PPU background scrolling functionality is implemented using nesdev loopy
// contributor design.
//
// Initial implementations of NES PPU emulations used an address register, a
// data and data buffer and a flag to indicate which byte was written in 16-bit
// writes.
//
// Although that implementation was useful for lots of games, a more accurate
// representation for the PPU behavior was found by loopy. Using two 16-bit
// address registers, a 3-bit tile X offset and a first/second write toggle,
// loopy registers are able to emulate more accurately the NES PPU. This
// registers are implemented as [`InternalRegisters`].
pub struct Ppu {
    pub bus: SharedGraphicsBus,
    event_bus: SharedEventBus,

    frame: Frame,
    frame_parity: FrameParity,

    registers: PpuRegisters,
    internal: RefCell<PpuInternalRegisters>,

    oam: Oam,

    cycle: u16,
    scan_line: u16,

    pixel_producer: PixelProducer,

    supress_vertical_blank: Cell<bool>,
}

#[derive(Default)]
enum FrameParity {
    #[default]
    Odd,
    Even,
}

#[derive(Default)]
struct PpuInternalRegisters {
    /// Current VRAM address (15 bits)
    vram_addr: RenderAddress,

    /// Temporary VRAM address, can also be thought of as the address of the top
    /// left onscreen tile
    temp_vram_addr: RenderAddress,

    /// Fine X scroll (3 bits)
    fine_x_scroll: u8,

    /// First or second write toggle (1 bit). `false` indicates first write
    write_toggle: WriteToggle,
}

#[derive(Default, Debug, Eq, PartialEq)]
enum WriteToggle {
    #[default]
    First,
    Second,
}

/// PPU's internal set of shift registers and multiplexers responsible of
/// producing pixel data.
///
/// In consists in shifters for background tile pattern and attributes as well
/// as sprite information.
///
/// Priority multiplexers decide how to combine all data to produce the correct
/// pixel.
struct PixelProducer {
    // Background
    buffers: Buffers,
    shifters: Shifters,

    // Sprites
    /// Up to 8 sprites used for a single scanline
    sprites: [OamSprite; 8],
}

/// Internal PPU latches that store temporary pixel data information while
/// rendering
#[derive(Default)]
pub struct Buffers {
    pub next_tile_number: u8,
    pub next_attributes: u8,
    pub next_bit_plane_high: u8,
    pub next_bit_plane_low: u8,
}

/// PPU Serial-to-parallel shift registers responsible of producing background
/// pixel data.
///
/// Shifters are 16-bit wide, the high 8 bits are used in the current pixels
/// being drawn while the low 8 bits will be used for the next tile
#[derive(Default)]
pub struct Shifters {
    pub attributes: (u16, u16),
    pub tile_pattern: (u16, u16),
}

impl Ppu {
    pub fn new(bus: SharedGraphicsBus, event_bus: SharedEventBus) -> Self {
        Self {
            bus: bus.clone(),
            event_bus,

            frame: Frame::black(),
            frame_parity: FrameParity::default(),

            registers: PpuRegisters::default(),
            internal: RefCell::new(PpuInternalRegisters::default()),

            oam: Oam::new(),

            cycle: 0,
            scan_line: 0,

            pixel_producer: PixelProducer {
                buffers: Buffers::default(),
                shifters: Shifters::default(),
                sprites: [OamSprite {
                    x: 0xFF,
                    y: 0xFF,
                    tile: 0xFF,
                    attributes: 0xFF,
                }; 8],
            },

            supress_vertical_blank: Cell::new(false),
        }
    }

    pub fn clock(&mut self) {
        // Screen rendering never stops

        if self.scan_line == 0 && self.cycle == 0 {
            if self.rendering_enabled() && matches!(self.frame_parity, FrameParity::Odd) {
                // "Odd frame" cycle skip
                self.cycle = 1;
            }
        }

        match self.scan_line {
            0..=239 | 261 => {
                // Scan lines responsible to render picture data
                //
                // 0..=239 -- Visible scan lines
                //
                // Background and foreground rendering occurs here. PPU is busy
                // fetching data, so the program should not access PPU memory
                // unless rendering is turned off
                //
                // 261 -- pre-render scanline
                //
                // This is a dummy scanline, whose sole purpose is to fill the
                // shift registers with the data for the first two tiles of the
                // next scanline. Although no pixels are rendered, the PPU still
                // makes the same memory accesses it would for a regular
                // scanline
                if self.scan_line == 261 && self.cycle == 1 {
                    self.end_vertical_blank();
                    self.registers.set_sprite_overflow(false);
                    self.registers.set_sprite_0_hit(false);
                    self.pixel_producer.sprites = [OamSprite {
                        x: 0xFF,
                        y: 0xFF,
                        tile: 0xFF,
                        attributes: 0xFF,
                    }; 8]
                }

                match self.cycle {
                    0 => {
                        // idle cycle
                    }

                    1..=256 | 321..=336 => {
                        // Fetching data takes 2 cycles per access, we ignore
                        // the first and do it in the second. To render a tile
                        // we need 4 accesses, then we need 8 clocks

                        if self.bg_rendering_enabled() {
                            self.update_shifters();
                        }

                        match (self.cycle - 1) % 8 {
                            // Fetch nametable byte
                            0 => {}
                            1 => {
                                self.pixel_producer.buffers.next_tile_number =
                                    self.nametable_fetch();
                            }

                            // fetch attribute table byte
                            2 => {
                                let mut next_attributes = self.attributes_fetch();
                                if self
                                    .internal
                                    .borrow()
                                    .vram_addr
                                    .get(RenderAddress::COARSE_Y_SCROLL)
                                    & 0x02
                                    > 0
                                {
                                    next_attributes >>= 4;
                                }
                                if self
                                    .internal
                                    .borrow()
                                    .vram_addr
                                    .get(RenderAddress::COARSE_X_SCROLL)
                                    & 0x02
                                    > 0
                                {
                                    next_attributes >>= 2;
                                }
                                next_attributes &= 0x03;

                                self.pixel_producer.buffers.next_attributes = next_attributes;
                            }
                            3 => {}

                            // fetch pattern table tile low
                            4 => {}
                            5 => {}

                            // fetch pattern table tile high
                            6 => {
                                let (high_plane, low_plane) = self.fetch_pattern_planes(
                                    self.pixel_producer.buffers.next_tile_number,
                                );
                                self.pixel_producer.buffers.next_bit_plane_high = high_plane;
                                self.pixel_producer.buffers.next_bit_plane_low = low_plane;
                            }
                            7 => {
                                self.load_shifters();
                                if self.rendering_enabled() {
                                    self.internal.borrow_mut().vram_addr.increment_x();
                                }
                            }
                            _ => unreachable!("We are matching exhausively all possible values"),
                        }

                        if self.cycle == 256 {
                            if self.rendering_enabled() {
                                self.internal.borrow_mut().vram_addr.increment_y();
                            }
                        }
                    }

                    257 => {
                        self.load_shifters();
                        if self.rendering_enabled() {
                            self.internal.borrow_mut().transfer_x();
                        }
                    }

                    280..=304 if self.scan_line == 261 => {
                        if self.rendering_enabled() {
                            self.internal.borrow_mut().transfer_y();
                        }
                    }

                    338 | 340 => {
                        // Unused NT fetches
                        self.pixel_producer.buffers.next_tile_number = self.nametable_fetch();
                    }

                    _ => {
                        // There are some unimplemented cycles with garbage NT,
                        // ignore them
                    }
                }

                // if 257 <= self.cycle && self.cycle <= 320 {
                //     self.registers.oam_addr = 0;
                // }
            }

            240 => {
                // post-render scan line. PPU idles
            }

            241 if self.cycle == 1 => {
                self.begin_vertical_blank();
            }

            241..=260 => {
                // vertical blank lines. After setting vertical blank and
                // trigger an NMI, the program can access PPU's memory
            }

            _ => panic!("Internal PPU error. Scanline is {}!", self.scan_line),
        }

        self.render_pixel();

        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.prepare_scanline_sprites();
            self.scan_line += 1;

            if self.scan_line > 261 {
                self.scan_line = 0;
                self.event_bus.emit(Event::FrameReady);
                self.frame_parity.reverse();
            }
        }
    }

    /// Begin the vertical blank period. It sets the PPU status to VBL and
    /// triggers an NMI if needed
    fn begin_vertical_blank(&mut self) {
        if !self.supress_vertical_blank.get() {
            self.registers.set_vertical_blank();
            if self.registers.nmi_enabled() {
                self.event_bus.emit(Event::NMI)
            }
        }
        self.supress_vertical_blank.set(false);
    }

    /// End vertical blank period. This unsets the VBL flag on PPU status
    fn end_vertical_blank(&mut self) {
        self.registers.unset_vertical_blank();
    }

    /// Fetch next tile ID to render using internal state: loopy v register and
    /// PPU configuration.
    ///
    /// Returns a byte specifying with tile to choose from the currently selected
    /// nametable
    fn nametable_fetch(&self) -> u8 {
        // High bits of v are used for fine Y during rendering and aren't needed
        // for nametable fetch. We fix the high 2 CHR address lines to 0x2000
        // region and use the remaining 12 bits from v.
        //
        // As nametables start in positions 0x2000, 0x2400, 0x2800 and 0x2C00,
        // the previous implementation use to add an offset for the selected
        // nametable. With the loopy v register this is no longer needed. We fix
        // the high 2 CR address lines to 0x2000 region and use the remaining 12
        // bits from v. High bits of v are used for fine Y during rendering, so
        // we aren't interested in them during nametable fetch
        let tile_number_address = 0x2000 | (self.internal.borrow().vram_addr.value() & 0x0FFF);
        self.bus.borrow().read(tile_number_address)
    }

    /// Fetch the attributes data corresponding to the next tile to render
    fn attributes_fetch(&self) -> u8 {
        // See
        // https://www.nesdev.org/wiki/PPU_scrolling#Tile_and_attribute_fetching
        // for further reference
        //
        // Attribute address is composed in the following way:
        // NN 1111 YYY XXX
        //  || |||| ||| +++-- high 3 bits of coarse X (x/4)
        //  || |||| +++------ high 3 bits of coarse Y (y/4)
        //  || ++++---------- attribute offset (960 bytes)
        //  ++--------------- nametable select
        let attributes_address = {
            let v = self.internal.borrow().vram_addr.value();
            0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)
        };
        self.bus.borrow().read(attributes_address)
    }

    /// Fetch background pattern planes corresponding to the next tile to render
    fn fetch_pattern_planes(&self, tile_number: u8) -> (u8, u8) {
        let pattern_table = self.registers.background_pattern_table();
        let fine_y = self
            .internal
            .borrow()
            .vram_addr
            .get(RenderAddress::FINE_Y_SCROLL) as u8;

        let mut pattern_table_address = PatternTableAddress::new(pattern_table);
        pattern_table_address.set(PatternTableAddress::TILE_NUMBER, tile_number);
        pattern_table_address.set(PatternTableAddress::FINE_Y_OFFSET, fine_y);

        pattern_table_address.set(PatternTableAddress::BIT_PLANE, 0);
        let low = self.bus.borrow().read(pattern_table_address.into());

        pattern_table_address.set(PatternTableAddress::BIT_PLANE, 1);
        let high = self.bus.borrow().read(pattern_table_address.into());

        (high, low)
    }

    fn render_pixel(&mut self) {
        let col = self.cycle as usize;
        let row = self.scan_line as usize;
        if self.registers.sprite_size() == 16 {
            unimplemented!("8x16 sprite");
        }
        let pixel = self.produce_pixel(col, row);
        if let Some(pixel) = pixel {
            self.frame.set_pixel(pixel, FramePixel { col, row });
        }
    }

    // Reexport for readability
    fn rendering_enabled(&self) -> bool {
        self.registers.rendering_enabled()
    }

    // Reexport for readability
    fn bg_rendering_enabled(&self) -> bool {
        self.registers.background_rendering_enabled()
    }

    // Reexport for readability
    fn sprite_rendering_enabled(&self) -> bool {
        self.registers.sprite_rendering_enabled()
    }

    /// Get the current frame being rendered by the PPU. Once the PPU signals
    /// `FrameReady` event through the event bus, this Frame is complete.
    pub fn take_frame(&mut self) -> Frame {
        let frame = self.frame.clone();
        self.frame = Frame::black();
        frame
    }

    pub fn oam_dma_write(&mut self, address: u8, data: u8) {
        self.oam.write(address as u16, data);
    }

    pub fn dump_oam(&self, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        file.write(format!("{:?}", self.oam).as_bytes())?;
        Ok(())
    }

    // First part of sprite rendering by scanline.
    //
    // In this setp, OAM is read looking for sprites to render in the next
    // scanline. It chooses a max of 8 sprites and load them in the pixel
    // producer
    fn prepare_scanline_sprites(&mut self) {
        if self.scan_line >= 240 {
            // only render in visible scanlines
            return;
        }

        // Cycles 1-64: secondary OAM initialization, all to 0xFF as if Y
        // coordinate is out of screen, we won't paint the sprite
        let mut secondary_oam = [OamSprite {
            x: 0xFF,
            y: 0xFF,
            tile: 0xFF,
            attributes: 0xFF,
        }; 8];

        // Cycles 65-256: read 8 sprites from OAM and write them into secondary
        // OAM if they are in screen
        let mut n = 0;
        let mut sprites_in_screen = 0;
        while n < 64 && sprites_in_screen < 9 {
            let sprite = self.oam.read_sprite(n);

            let diff = (self.scan_line as i16) - (sprite.y as i16);
            if diff >= 0 && diff < 8 {
                if sprites_in_screen < 8 {
                    secondary_oam[sprites_in_screen] = sprite;
                    sprites_in_screen += 1;
                }
            }

            n += 1;
        }

        self.registers.set_sprite_overflow(sprites_in_screen > 9);

        self.pixel_producer.sprites = secondary_oam;
    }

    // Load shift registers from internal latches (buffers) so next 8 pixels can
    // be drawn by the PPU in the next clock cycles
    pub fn load_shifters(&mut self) {
        self.pixel_producer.shifters.tile_pattern.0 = (self.pixel_producer.shifters.tile_pattern.0
            & 0xFF00)
            | (self.pixel_producer.buffers.next_bit_plane_low as u16);
        self.pixel_producer.shifters.tile_pattern.1 = (self.pixel_producer.shifters.tile_pattern.1
            & 0xFF00)
            | (self.pixel_producer.buffers.next_bit_plane_high as u16);

        let attributes_0 = if utils::bv(self.pixel_producer.buffers.next_attributes, 0) == 0 {
            0
        } else {
            0xFF
        };
        self.pixel_producer.shifters.attributes.0 =
            (self.pixel_producer.shifters.attributes.0 & 0xFF00) | attributes_0 as u16;

        let attributes_1 = if utils::bv(self.pixel_producer.buffers.next_attributes, 1) == 0 {
            0
        } else {
            0xFF
        };
        self.pixel_producer.shifters.attributes.1 =
            (self.pixel_producer.shifters.attributes.1 & 0xFF00) | attributes_1 as u16;
    }

    pub fn update_shifters(&mut self) {
        self.pixel_producer.shifters.tile_pattern.0 =
            self.pixel_producer.shifters.tile_pattern.0 << 1;
        self.pixel_producer.shifters.tile_pattern.1 =
            self.pixel_producer.shifters.tile_pattern.1 << 1;
        self.pixel_producer.shifters.attributes.0 = self.pixel_producer.shifters.attributes.0 << 1;
        self.pixel_producer.shifters.attributes.1 = self.pixel_producer.shifters.attributes.1 << 1;
    }

    pub fn produce_pixel(&mut self, col: usize, row: usize) -> Option<Pixel> {
        if col >= 256 || row >= 240 {
            return None;
        }

        // Background

        let mut background_palette = 0;
        let mut background_bit_plane = 0;

        if self.bg_rendering_enabled() {
            let fine_x = self.internal.borrow().fine_x_scroll;
            let fine_x_bit = 15 - fine_x;

            background_palette = {
                let palette_lo =
                    utils::bv_16(self.pixel_producer.shifters.attributes.0, fine_x_bit);
                let palette_hi =
                    utils::bv_16(self.pixel_producer.shifters.attributes.1, fine_x_bit);
                (palette_hi << 1) | palette_lo
            };
            background_bit_plane = {
                let bit_plane_lo =
                    utils::bv_16(self.pixel_producer.shifters.tile_pattern.0, fine_x_bit);
                let bit_plane_hi =
                    utils::bv_16(self.pixel_producer.shifters.tile_pattern.1, fine_x_bit);
                (bit_plane_hi << 1) | bit_plane_lo
            };
        }

        // ----------------------------------------------------------------------------------------------------

        // Sprites

        let mut sprite_palette = 0;
        let mut sprite_bit_plane = 0;
        let mut priority = 0; // 0 -> front of background, 1 -> behind background
        let mut sprite_number = u8::MAX;

        if self.sprite_rendering_enabled() {
            for (idx, sprite) in self.pixel_producer.sprites.iter_mut().enumerate() {
                // no more valid sprites
                if sprite.y == 0xFF {
                    break;
                }

                if col < (sprite.x as usize) || col >= (sprite.x as usize + 8) {
                    continue;
                }

                sprite_number = idx as u8;

                let mut pattern_table_address =
                    PatternTableAddress::new(self.registers.sprite_pattern_table());
                pattern_table_address.set(PatternTableAddress::TILE_NUMBER, sprite.tile);

                sprite_palette = (sprite.attributes & 0b0000_0011) + 4; // sprite palettes are 4 to 7

                priority = utils::bv(sprite.attributes, 5);
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

                sprite_bit_plane = utils::bv(high, x as u8) << 1 | utils::bv(low, x as u8);

                if sprite_bit_plane > 0 {
                    // first non transparent sprite, let's render it
                    break;
                }
            }
        }

        // Choose background or sprite pixel

        let palette_offset = if background_bit_plane == 0 && sprite_bit_plane == 0 {
            // EXT in $3F00
            0
        } else if background_bit_plane == 0 && sprite_bit_plane > 0 {
            // paint sprite
            ((sprite_palette << 2) | sprite_bit_plane) as u16
        } else if background_bit_plane > 0 && sprite_bit_plane == 0 {
            // paint background
            (background_palette << 2) | background_bit_plane
        } else {
            if sprite_number == 0 {
                if !(
                    // skip at x=0 to x=7 i left-side clipping window is enabled
                    (col <= 7 && self.registers.left_size_clipping_window_enabled())
                    // skip at x=255, for an obscure reason related to the pixel
                    // pipeline
                    || (col == 255)
                ) {
                    self.registers.set_sprite_0_hit(true);
                }
            }

            if priority == 0 {
                // paint sprite
                ((sprite_palette << 2) | sprite_bit_plane) as u16
            } else {
                // paint background
                (background_palette << 2) | background_bit_plane
            }
        };

        let color = Pixel::from(
            self.bus
                .borrow()
                .read(PALETTE_MEMORY_START + palette_offset),
        );

        Some(color)
    }

    // TODO: move to example?
    fn render_nametable(&self) -> Frame {
        let mut screen = Frame::black();

        let pattern_table = self.registers.background_pattern_table();
        let (pattern_table_address, offset) = match pattern_table {
            0 => (0x0000, 0),
            1 => (0x1000, 16),
            _ => panic!("There's no pattern table {pattern_table}"),
        };

        let nametable = self.registers.ctrl.bits() & 0b0000_0011;
        let nametable_address = match nametable {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => panic!("There's no name table {nametable}"),
        };

        let attribute_table_address = nametable_address + 960;

        // println!(
        //     "Pattern table: {pattern_table}. Nametable: {nametable}. Mirroring: {0:?}",
        //     self.mirroring
        // );

        for row in 0..30 {
            for col in 0..32 {
                let tile_number_address = (nametable_address + row * 32 + col) as u16;
                let tile_number = self.bus.borrow().read(tile_number_address) as usize;

                let mut bit_planes = [0; 16];
                for (i, item) in bit_planes.iter_mut().enumerate() {
                    let address = pattern_table_address + (tile_number * 16 + i) as u16;
                    *item = self.bus.borrow().read(address);
                }

                let attributes_address = (attribute_table_address + row / 4 * 8 + col / 4) as u16;
                let attributes = self.bus.borrow().read(attributes_address);

                let palette_number = match (col % 4, row % 4) {
                    (x, y) if x < 2 && y < 2 => utils::bvs_8(attributes, 1, 0),
                    (x, y) if x >= 2 && y < 2 => utils::bvs_8(attributes, 3, 2),
                    (x, y) if x < 2 && y >= 2 => utils::bvs_8(attributes, 5, 4),
                    (x, y) if x >= 2 && y >= 2 => utils::bvs_8(attributes, 7, 6),
                    (x, y) => unreachable!("Impossible situation: x={x}, y={y}"),
                };

                for x in 0..8 {
                    for y in 0..8 {
                        let pattern = (utils::bv(bit_planes[y + 8], x as u8) << 1)
                            | (utils::bv(bit_planes[y], x as u8));
                        let color = self
                            .bus
                            .borrow()
                            .read(0x3F00 + ((palette_number << 2) | pattern) as u16);
                        let pixel = Pixel::from(color);

                        // let mrow = (tile_number / 16) * 8 + y;
                        // let mcol = ((tile_number % 16) + offset) * 8 + (7 - x);
                        let mrow = row as usize * 8 + y;
                        let mcol = col as usize * 8 + (7 - x);
                        screen.set_pixel(
                            pixel,
                            FramePixel {
                                row: mrow,
                                col: mcol,
                            },
                        );
                    }
                }
            }
        }
        screen
    }
}

impl Memory for Ppu {
    fn read(&self, address: u16) -> u8 {
        // PPU registers are mirrored every 8 bytes
        let address = (address & 0b0111) + 0x2000;
        let data = match address {
            PPUCTRL => self.registers.data_buffer.get() & 0x1F,

            PPUSTATUS => {
                // Internal
                let mut internal = self.internal.borrow_mut();
                internal.write_toggle = WriteToggle::First;

                // The 5 lower bits reflect the PPU bus contents. Although
                // emulated, no games should relay on this behaviour
                let ppustatus = (self.registers.status.get().bits() & 0xE0)
                    | (self.registers.data_buffer.get() & 0x1F);

                // Reading PPU status clears VBL flag and the address latch
                self.registers.unset_vertical_blank();
                // self.registers.data_buffer.set(0);

                // also, if status is read one PPU clock before vertical blank,
                // it'll supress the NMI
                self.supress_vertical_blank
                    .set(self.scan_line == 241 && self.cycle == (1 - 1));

                // reading in the VBL clock or one later reads the flag as true
                if self.scan_line == 241
                    && (self.cycle == 1 || self.cycle == (1 + 1) || self.cycle == (1 + 2))
                {
                    // if self.scan_line == 241 && (self.cycle == 1) {
                    self.event_bus.mark_as_processed(Event::NMI);
                    ppustatus | 0b1000_0000
                } else {
                    ppustatus
                }
            }

            OAMDATA => {
                let oam_addr = self.registers.oam_addr as u16;
                self.oam.read(oam_addr)
            }

            PPUDATA => {
                let mut internal = self.internal.borrow_mut();

                // Delayed read
                let mut data = self.registers.data_buffer.get();

                // Update buffer for next read
                let vram_address = internal.vram_addr.value();
                let vram_data = self.bus.borrow().read(vram_address);
                self.registers.data_buffer.set(vram_data);

                if vram_address >= 0x3F00 {
                    // some addresses used combinatory logic to avoid one clock
                    // delay between reading and having data available (palettes
                    // for example)
                    data = vram_data;
                }

                // Auto-increment vram address horizontally or vertically
                let increment = self.registers.vram_address_increment();
                internal.vram_addr = RenderAddress::from(vram_address + increment);

                data
            }
            _ => unimplemented!("PPU read not implemented for address: {address:0>4X}"),
        };
        trace!("PPU read from: {address:0>4X} <- {data:0>2X}");
        data
    }

    fn write(&mut self, address: u16, data: u8) {
        trace!("PPU write to: {address:0>4X} -> {data:0>2X}");

        let address = address + 0x2000;
        match address {
            PPUCTRL => {
                let mut internal = self.internal.borrow_mut();
                internal
                    .temp_vram_addr
                    .set(RenderAddress::NAMETABLES_SELECT, data & 0b00000011);

                // Registers
                self.registers.ctrl = PpuCtrl::from_bits_truncate(data);
            }
            PPUMASK => {
                // Registers
                self.registers.mask = PpuMask::from_bits_truncate(data);
            }

            OAMADDR => {
                self.registers.oam_addr = data;
            }

            OAMDATA => {
                let oam_addr = self.registers.oam_addr;
                self.oam.write(oam_addr as u16, data);
                self.registers.oam_addr = oam_addr.wrapping_add(1);
            }

            PPUSCROLL => {
                let mut internal = self.internal.borrow_mut();
                match internal.write_toggle {
                    WriteToggle::First => {
                        internal
                            .temp_vram_addr
                            .set(RenderAddress::COARSE_X_SCROLL, data >> 3);
                        internal.fine_x_scroll = data & 0b0000_0111;
                        internal.write_toggle = WriteToggle::Second;
                    }
                    WriteToggle::Second => {
                        internal
                            .temp_vram_addr
                            .set(RenderAddress::FINE_Y_SCROLL, data & 0b0000_0111);
                        internal
                            .temp_vram_addr
                            .set(RenderAddress::COARSE_Y_SCROLL, data >> 3);
                        internal.write_toggle = WriteToggle::First;
                    }
                }
            }

            PPUADDR => {
                let mut internal = self.internal.borrow_mut();
                match internal.write_toggle {
                    WriteToggle::First => {
                        internal.temp_vram_addr = RenderAddress::from(
                            (
                                (internal.temp_vram_addr.value() & 0b1100_0000_1111_1111)
                                    | ((data as u16 & 0b0011_1111) << 8)
                            )
                            // XXX according to nesdev, t (bit 15) = Z and bit Z is cleared.
                            // What's bit Z?
                                & 0b0011_1111_1111_1111,
                        );
                        internal.write_toggle = WriteToggle::Second;
                    }
                    WriteToggle::Second => {
                        internal.temp_vram_addr = RenderAddress::from(
                            (internal.temp_vram_addr.value() & 0xFF00) | data as u16,
                        );
                        internal.vram_addr = internal.temp_vram_addr;
                        internal.write_toggle = WriteToggle::First;
                    }
                }
            }

            PPUDATA => {
                let mut internal = self.internal.borrow_mut();

                let vram_address = internal.vram_addr.value();
                self.bus.borrow_mut().write(vram_address, data);

                // Auto-increment vram address horizontally or vertically
                let increment = self.registers.vram_address_increment();
                internal.vram_addr = RenderAddress::from(vram_address + increment);
            }

            _ => unimplemented!("PPU write not implemented for address: {address:0>4X}"),
        }
    }

    fn size(&self) -> usize {
        // TODO: change number of mirrors to real number
        let mirrors = (0x3FFF - 0x2008 + 1) / 8;
        (0x2007 - 0x2000 + 1) * mirrors
    }
}

impl PpuInternalRegisters {
    // Transfer X address from the temporary VRAM address (t) to the current
    // VRAM address (v)
    fn transfer_x(&mut self) {
        debug!(
            "Move X temp vram to vram: {:016b} -> {:016b}",
            self.temp_vram_addr.value(),
            self.vram_addr.value()
        );
        self.vram_addr.set(
            RenderAddress::HORIZONTAL_NAMETABLE,
            self.temp_vram_addr.get(RenderAddress::HORIZONTAL_NAMETABLE) as u8,
        );
        self.vram_addr.set(
            RenderAddress::COARSE_X_SCROLL,
            self.temp_vram_addr.get(RenderAddress::COARSE_X_SCROLL) as u8,
        );
    }

    // Transfer Y address from the temporary VRAM address (t) to the current
    // VRAM address (v)
    fn transfer_y(&mut self) {
        debug!(
            "Move Y temp vram to vram: {:016b} -> {:016b}",
            self.temp_vram_addr.value(),
            self.vram_addr.value()
        );
        self.vram_addr.set(
            RenderAddress::VERTICAL_NAMETABLE,
            self.temp_vram_addr.get(RenderAddress::VERTICAL_NAMETABLE) as u8,
        );
        self.vram_addr.set(
            RenderAddress::COARSE_Y_SCROLL,
            self.temp_vram_addr.get(RenderAddress::COARSE_Y_SCROLL) as u8,
        );
        self.vram_addr.set(
            RenderAddress::FINE_Y_SCROLL,
            self.temp_vram_addr.get(RenderAddress::FINE_Y_SCROLL) as u8,
        );
    }
}

impl FrameParity {
    fn reverse(&mut self) {
        *self = match self {
            Self::Odd => Self::Even,
            Self::Even => Self::Odd,
        }
    }
}

#[cfg(test)]
impl PpuInternalRegisters {
    fn reset(&mut self) {
        self.vram_addr = RenderAddress::from(0);
        self.temp_vram_addr = RenderAddress::from(0);
        self.fine_x_scroll = 0;
        self.write_toggle = WriteToggle::First;
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::hardware::PPU_REGISTERS_START;
    use crate::processor::bus::GraphicsBus;

    use super::*;

    fn test_ppu() -> Ppu {
        let graphics_bus = Rc::new(RefCell::new(GraphicsBus::new()));
        let event_bus = SharedEventBus::new();
        Ppu::new(graphics_bus, event_bus)
    }

    #[test]
    fn test_loopy_scrolling_registers_read_and_write() {
        // Test inspired by example in:
        // https://www.nesdev.org/wiki/PPU_scrolling#Summary

        let mut ppu = test_ppu();

        // LDA $00
        // STA $2000
        ppu.write(PPUCTRL - PPU_REGISTERS_START, 0);
        assert_eq!(ppu.internal.borrow().temp_vram_addr.value(), 0);

        // LDA $2002 -- resets write latch to 0
        ppu.read(PPUSTATUS - PPU_REGISTERS_START);
        assert_eq!(ppu.internal.borrow().write_toggle, WriteToggle::First);

        // LDA $7D
        // STA $2005 -- PPUSCROLL write 1
        // LDA $5E
        // STA $2005 -- PPUSCROLL write 2
        ppu.write(PPUSCROLL - PPU_REGISTERS_START, 0x7D);
        assert_eq!(
            ppu.internal.borrow().temp_vram_addr.value(),
            0b0000000_00001111
        );
        assert_eq!(ppu.internal.borrow().vram_addr.value(), 0);
        assert_eq!(ppu.internal.borrow().fine_x_scroll, 0b101);
        assert_eq!(ppu.internal.borrow().write_toggle, WriteToggle::Second);

        ppu.write(PPUSCROLL - PPU_REGISTERS_START, 0x5E);
        assert_eq!(
            ppu.internal.borrow().temp_vram_addr.value(),
            0b1100001_01101111
        );
        assert_eq!(ppu.internal.borrow().vram_addr.value(), 0);
        assert_eq!(ppu.internal.borrow().fine_x_scroll, 0b101);
        assert_eq!(ppu.internal.borrow().write_toggle, WriteToggle::First);

        // LDA $3D
        // STA $2006 -- PPUADDR write 1
        // LDA $F0
        // STA $2006 -- PPUADDR write 2
        ppu.write(PPUADDR - PPU_REGISTERS_START, 0x3D);
        assert_eq!(
            ppu.internal.borrow().temp_vram_addr.value(),
            0b0111101_01101111
        );
        assert_eq!(ppu.internal.borrow().vram_addr.value(), 0);
        assert_eq!(ppu.internal.borrow().fine_x_scroll, 0b101);
        assert_eq!(ppu.internal.borrow().write_toggle, WriteToggle::Second);

        ppu.write(PPUADDR - PPU_REGISTERS_START, 0xF0);
        assert_eq!(
            ppu.internal.borrow().temp_vram_addr.value(),
            0b0111101_11110000
        );
        assert_eq!(ppu.internal.borrow().vram_addr.value(), 0b0111101_11110000);
        assert_eq!(ppu.internal.borrow().fine_x_scroll, 0b101);
        assert_eq!(ppu.internal.borrow().write_toggle, WriteToggle::First);
    }

    // See https://www.nesdev.org/wiki/PPU_scrolling#Register_controls
    // for further reference
    mod test_ppu_internal_registers {
        use super::*;

        #[test]
        fn test_ppuctrl_write() {
            let mut ppu = test_ppu();

            ppu.write(PPUCTRL - PPU_REGISTERS_START, 0b1111_1111);
            let regs = ppu.internal.borrow();
            assert_eq!(regs.vram_addr.value(), 0);
            assert_eq!(regs.temp_vram_addr.value(), 0b000_1100_0000_0000);
            assert_eq!(regs.fine_x_scroll, 0);
            assert_eq!(regs.write_toggle, WriteToggle::First);
        }

        #[test]
        fn test_ppustatus_read() {
            let ppu = test_ppu();

            ppu.internal.borrow_mut().write_toggle = WriteToggle::First;
            ppu.read(PPUSTATUS - PPU_REGISTERS_START);
            {
                let regs = ppu.internal.borrow();
                assert_eq!(regs.vram_addr.value(), 0);
                assert_eq!(regs.temp_vram_addr.value(), 0);
                assert_eq!(regs.fine_x_scroll, 0);
                assert_eq!(regs.write_toggle, WriteToggle::First);
            }

            ppu.internal.borrow_mut().write_toggle = WriteToggle::Second;
            ppu.read(PPUSTATUS - PPU_REGISTERS_START);
            {
                let regs = ppu.internal.borrow();
                assert_eq!(regs.vram_addr.value(), 0);
                assert_eq!(regs.temp_vram_addr.value(), 0);
                assert_eq!(regs.fine_x_scroll, 0);
                assert_eq!(regs.write_toggle, WriteToggle::First);
            }
        }
    }

    #[test]
    fn test_ppuscroll_writes() {
        let mut ppu = test_ppu();

        // first write

        ppu.write(PPUSCROLL - PPU_REGISTERS_START, 0b1111_1000);
        {
            let regs = ppu.internal.borrow();
            assert_eq!(regs.vram_addr.value(), 0);
            assert_eq!(regs.temp_vram_addr.value(), 0b0000_0000_0001_1111);
            assert_eq!(regs.fine_x_scroll, 0);
            assert_eq!(regs.write_toggle, WriteToggle::Second);
        }

        ppu.internal.borrow_mut().reset();

        ppu.write(PPUSCROLL - PPU_REGISTERS_START, 0b0000_0111);
        {
            let regs = ppu.internal.borrow();
            assert_eq!(regs.vram_addr.value(), 0);
            assert_eq!(regs.temp_vram_addr.value(), 0);
            assert_eq!(regs.fine_x_scroll, 0b111);
            assert_eq!(regs.write_toggle, WriteToggle::Second);
        }

        // second write

        ppu.internal.borrow_mut().reset();
        ppu.internal.borrow_mut().write_toggle = WriteToggle::Second;
        ppu.write(PPUSCROLL - PPU_REGISTERS_START, 0b1111_1000);
        {
            let regs = ppu.internal.borrow();
            assert_eq!(regs.vram_addr.value(), 0);
            assert_eq!(regs.temp_vram_addr.value(), 0b0000_0011_1110_0000);
            assert_eq!(regs.fine_x_scroll, 0);
            assert_eq!(regs.write_toggle, WriteToggle::First);
        }

        ppu.internal.borrow_mut().reset();
        ppu.internal.borrow_mut().write_toggle = WriteToggle::Second;
        ppu.write(PPUSCROLL - PPU_REGISTERS_START, 0b0000_0111);
        {
            let regs = ppu.internal.borrow();
            assert_eq!(regs.vram_addr.value(), 0);
            assert_eq!(regs.temp_vram_addr.value(), 0b0111_0000_0000_0000);
            assert_eq!(regs.fine_x_scroll, 0);
            assert_eq!(regs.write_toggle, WriteToggle::First);
        }
    }

    #[test]
    fn test_ppuaddr_writes() {
        let mut ppu = test_ppu();

        // first write

        // bit Z (15) is cleared
        ppu.internal.borrow_mut().temp_vram_addr = RenderAddress::from(0b0100_0000_0000_0000);
        ppu.write(PPUADDR - PPU_REGISTERS_START, 0);
        {
            let regs = ppu.internal.borrow();
            assert_eq!(regs.vram_addr.value(), 0);
            assert_eq!(regs.temp_vram_addr.value(), 0);
            assert_eq!(regs.fine_x_scroll, 0);
            assert_eq!(regs.write_toggle, WriteToggle::Second);
        }

        // First two bits of data are unused
        ppu.internal.borrow_mut().reset();
        ppu.write(PPUADDR - PPU_REGISTERS_START, 0b1100_0000);
        {
            let regs = ppu.internal.borrow();
            assert_eq!(regs.vram_addr.value(), 0);
            assert_eq!(regs.temp_vram_addr.value(), 0);
            assert_eq!(regs.fine_x_scroll, 0);
            assert_eq!(regs.write_toggle, WriteToggle::Second);
        }

        // set value into temp_vram_addr
        ppu.internal.borrow_mut().reset();
        ppu.write(PPUADDR - PPU_REGISTERS_START, 0b0011_1111);
        {
            let regs = ppu.internal.borrow();
            assert_eq!(regs.vram_addr.value(), 0);
            assert_eq!(regs.temp_vram_addr.value(), 0b0011_1111_0000_0000);
            assert_eq!(regs.fine_x_scroll, 0);
            assert_eq!(regs.write_toggle, WriteToggle::Second);
        }

        // second write

        ppu.internal.borrow_mut().reset();
        ppu.internal.borrow_mut().write_toggle = WriteToggle::Second;
        ppu.write(PPUADDR - PPU_REGISTERS_START, 0b1111_1111);
        {
            let regs = ppu.internal.borrow();
            assert_eq!(regs.temp_vram_addr.value(), 0b0000_0000_1111_1111);
            assert_eq!(regs.vram_addr.value(), 0b0000_0000_1111_1111);
            assert_eq!(regs.fine_x_scroll, 0);
            assert_eq!(regs.write_toggle, WriteToggle::First);
        }

        ppu.internal.borrow_mut().write_toggle = WriteToggle::Second;
        ppu.write(PPUADDR - PPU_REGISTERS_START, 0b1010_1010);
        {
            let regs = ppu.internal.borrow();
            assert_eq!(regs.temp_vram_addr.value(), 0b0000_0000_1010_1010);
            assert_eq!(regs.vram_addr.value(), 0b0000_0000_1010_1010);
            assert_eq!(regs.fine_x_scroll, 0);
            assert_eq!(regs.write_toggle, WriteToggle::First);
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_ppudata_reads_and_writes_TEST_NOT_IMPLEMENTED() {
        // TODO
    }
}
