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

use std::cell::RefCell;

use crate::graphics::Frame;
use crate::graphics::FramePixel;
use crate::graphics::Pixel;
use crate::hardware::{
    OAMADDR, PALETTE_MEMORY_START, PPUADDR, PPUCTRL, PPUDATA, PPUMASK, PPUSCROLL, PPUSTATUS,
};
use crate::interfaces::{Bus, Memory};
use crate::processor::memory::Mirroring;
use crate::types::SharedBus;
use crate::utils;
use crate::utils::BitGroup;

use super::pattern_table::PatternTableAddress;
use super::ppu_registers::{PpuCtrl, PpuStatus};

// PPU background scrolling functionality is implemented using nesdev loopy
// contributor design.
//
// Initial implementations of NES PPU emulations used and address register, a
// data and data buffer and a flag to indicate which byte was written in 16-bit
// writes.
//
// Although that implementation was useful for lots of games, a more accurate
// representation for the PPU behavior was found by loopy. Using two 16-bit
// address registers, a 3-bit tile X offset and a first/second write toggle,
// loopy registers are able to emulate more accurately the NES PPU. This
// registers are implemented as [`InternalRegisters`].
pub struct Ppu {
    registers: RefCell<PpuRegisters>,
    internal: RefCell<PpuInternalRegisters>,
    bus: SharedBus,
    cycle: u16,
    scan_line: u16,
    frame: Frame,
    frame_completed: bool,
    mirroring: Mirroring,
    nmi_request: bool,
}

struct PpuRegisters {
    ctrl: PpuCtrl,
    mask: u8,
    status: PpuStatus,

    // TODO
    address: u16,
    ongoing_address_write: bool,

    data: u8,
    data_buffer: u8,
}

#[derive(Clone, Copy)]
pub struct RenderAddress {
    value: BitGroup<u16>,
}

impl RenderAddress {
    pub const FINE_Y_SCROLL: u16 = 0b0111_0000_0000_0000;
    pub const NAMETABLE_SELECT: u16 = 0b0000_1100_0000_0000;
    pub const COARSE_Y_SCROLL: u16 = 0b0000_0011_1110_0000;
    pub const COARSE_X_SCROLL: u16 = 0b0000_0000_0001_1111;

    pub fn set(&mut self, group: u16, value: u8) {
        self.value.set(group, value.into());
    }

    pub fn value(&self) -> u16 {
        self.value.into()
    }
}

impl From<RenderAddress> for u16 {
    fn from(value: RenderAddress) -> Self {
        value.value.into()
    }
}

impl From<u16> for RenderAddress {
    fn from(value: u16) -> Self {
        Self {
            value: BitGroup::new(value),
        }
    }
}

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

#[derive(Default, Eq, PartialEq)]
enum WriteToggle {
    #[default]
    First,
    Second,
}

impl Ppu {
    pub fn new(bus: SharedBus) -> Self {
        Self {
            registers: RefCell::new(PpuRegisters {
                ctrl: PpuCtrl::empty(),
                mask: 0,
                status: PpuStatus::empty(),

                // TODO
                address: 0,
                ongoing_address_write: false,

                data: 0,
                data_buffer: 0,
            }),
            internal: RefCell::new(PpuInternalRegisters {
                vram_addr: RenderAddress::from(0),
                temp_vram_addr: RenderAddress::from(0),
                fine_x_scroll: 0,
                write_toggle: WriteToggle::First,
            }),
            bus,
            cycle: 0,
            scan_line: 0,
            frame: Frame::black(),
            frame_completed: false,
            mirroring: Mirroring::Horizontal,
            nmi_request: false,
        }
    }

    pub fn clock(&mut self) {
        // Screen rendering never stops

        match self.scan_line {
            0..=239 => {
                // visible scan lines. Background and foreground rendering
                // occurs here. PPU is busy fetching data, so the program should
                // not access PPU memory unless rendering is turned off

                match self.cycle {
                    0 => {
                        // idle cycle
                    }
                    1..=256 => {
                        // fetch data for tile 2 cycles per acces, 4 access in total:
                        // - nametable byte
                        // - attribute table byte
                        // - pattern table tile low
                        // - pattern table tile high
                        //
                        //
                        // implement a function 'render_tile(x, y)' or similar
                        // that, given computed coordinates from cycles and
                        // scanlines, renders a pixel or something like that

                        match self.cycle % 8 {
                            // Fetch nametable byte
                            0 => {}
                            1 => {
                                // let address = self.internal.borrow().vram_addr.bits();
                            }

                            // fetch attribute table byte
                            2 => {}
                            3 => {
                                // let address = self.internal.borrow().vram_addr.bits() + 960;
                            }

                            // fetch pattern table tile low
                            4 => {}
                            5 => {}

                            // fetch pattern table tile high
                            6 => {}
                            7 => {
                                let pattern_table = self
                                    .registers
                                    .borrow()
                                    .ctrl
                                    .intersection(PpuCtrl::BACKGROUND_PATTERN_TABLE)
                                    .bits()
                                    >> PpuCtrl::BACKGROUND_PATTERN_TABLE.bits().trailing_zeros();

                                let nametable = self
                                    .registers
                                    .borrow()
                                    .ctrl
                                    .intersection(PpuCtrl::BASENAME_NAMETABLE_ADDRESS)
                                    .bits()
                                    >> PpuCtrl::BASENAME_NAMETABLE_ADDRESS.bits().trailing_zeros();

                                let nametable_base_address = match nametable {
                                    0 => 0x2000,
                                    1 => 0x2400,
                                    2 => 0x2800,
                                    3 => 0x2C00,
                                    _ => panic!("Internal PPU error. Name table is {nametable}"),
                                };

                                let attribute_table_address = nametable_base_address + 960;

                                // scan_line to 0..30 rows
                                let row = self.scan_line >> 3;
                                // cycles to 0..32 cols
                                let col = self.cycle >> 3;

                                let fine_y = self.scan_line % 8;

                                // fetch name table
                                // let tile_number_address = nametable_base_address + row * 32 + col;
                                // let tile_number_address = self.internal.borrow().vram_addr.bits();
                                let tile_number_address = nametable_base_address + row * 32 + col;
                                let tile_number = self.bus.borrow().read(tile_number_address);

                                // fetch attribute table
                                let attribute_address =
                                    attribute_table_address + row / 4 * 8 + col / 4;
                                let attributes = self.bus.borrow().read(attribute_address);
                                let palette = match (col % 4, row % 4) {
                                    (x, y) if x < 2 && y < 2 => utils::bvs_8(attributes, 1, 0),
                                    (x, y) if x >= 2 && y < 2 => utils::bvs_8(attributes, 3, 2),
                                    (x, y) if x < 2 && y >= 2 => utils::bvs_8(attributes, 5, 4),
                                    (x, y) if x >= 2 && y >= 2 => utils::bvs_8(attributes, 7, 6),
                                    (x, y) => panic!("Impossible situation: x={x}, y={y}"),
                                };

                                let mut pattern_table_address =
                                    PatternTableAddress::new(pattern_table);
                                pattern_table_address
                                    .set(PatternTableAddress::TILE_NUMBER, tile_number);
                                pattern_table_address
                                    .set(PatternTableAddress::FINE_Y_OFFSET, fine_y as u8);

                                pattern_table_address.set(PatternTableAddress::BIT_PLANE, 0);
                                let low = self.bus.borrow().read(pattern_table_address.into());

                                pattern_table_address.set(PatternTableAddress::BIT_PLANE, 1);
                                let high = self.bus.borrow().read(pattern_table_address.into());

                                // let fine_x = self.internal.borrow().fine_x_scroll;

                                for x in 0..8 {
                                    let palette_offset = (palette << 2)
                                        | utils::bv(high, 7 - x) << 1
                                        | utils::bv(low, 7 - x);
                                    let palette_color = self
                                        .bus
                                        .borrow()
                                        .read(PALETTE_MEMORY_START + palette_offset as u16);
                                    let color = Pixel::from(palette_color);

                                    let m_row = self.scan_line as usize;
                                    let m_col = self.cycle - 7 + x as u16;
                                    self.frame.set_pixel(
                                        color,
                                        FramePixel {
                                            row: m_row as usize,
                                            col: m_col as usize,
                                        },
                                    );
                                }

                                // TODO (CONTINUE HERE!)
                            }
                            _ => panic!("Impossible condition!"),
                        }
                    }
                    257..=320 => {}
                    321..=336 => {}
                    337..=340 => {}
                    _ => panic!("Internal PPU error. Cycle is {}!", self.cycle),
                }
            }

            240 => {
                // post-render scan line. PPU idles
            }

            241 if self.cycle == 1 => {
                self.set_vertical_blank();

                if self.registers.borrow().ctrl.contains(PpuCtrl::NMI_ENABLE) {
                    self.request_nmi();
                }
            }

            241..=260 => {
                // vertical blank lines. After setting vertical blank and
                // trigger an NMI, the program access PPU's memory
            }

            261 if self.cycle == 1 => {
                self.unset_vertical_blank();
            }

            261 => {
                // dummy scan line to fill the first two tiles of the next
                // scanline
            }

            _ => panic!("Internal PPU error. Scanline is {}!", self.scan_line),
        }

        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scan_line += 1;

            if self.scan_line > 261 {
                self.scan_line = 0;
                self.frame_completed = true;
            }
        }
    }

    pub fn set_mirroring(&mut self, mirroring: Mirroring) {
        self.mirroring = mirroring;
    }

    pub fn frame_ready(&self) -> bool {
        self.frame_completed
    }

    pub fn take_frame(&mut self) -> Frame {
        self.frame_completed = false;

        let frame = self.frame.clone();
        self.frame = Frame::black();
        frame
    }

    pub fn is_nmi_requested(&self) -> bool {
        self.nmi_request
    }

    pub fn nmi_accepted(&mut self) {
        self.nmi_request = false;
    }

    fn request_nmi(&mut self) {
        self.nmi_request = true;
    }

    fn set_vertical_blank(&mut self) {
        self.registers
            .borrow_mut()
            .status
            .set(PpuStatus::VERTICAL_BLANK, true);
    }

    fn unset_vertical_blank(&mut self) {
        self.registers
            .borrow_mut()
            .status
            .set(PpuStatus::VERTICAL_BLANK, false);
    }

    fn render_nametable(&self) -> Frame {
        let mut screen = Frame::black();

        let pattern_table = self
            .registers
            .borrow()
            .ctrl
            .intersection(PpuCtrl::BACKGROUND_PATTERN_TABLE)
            .bits()
            >> PpuCtrl::BACKGROUND_PATTERN_TABLE.bits().trailing_zeros();
        let (pattern_table_address, offset) = match pattern_table {
            0 => (0x0000, 0),
            1 => (0x1000, 16),
            _ => panic!("There's no pattern table {pattern_table}"),
        };

        let nametable = self.registers.borrow().ctrl.bits() & 0b0000_0011;
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
                    (x, y) => panic!("Impossible situation: x={x}, y={y}"),
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
                        let mrow = (row as usize * 8 + y) as usize;
                        let mcol = (col as usize * 8 + (7 - x)) as usize;
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

    fn address_to_pattern_table(&self, address: u16) {
        let hi_address = ((address & 0xFF00) >> 8) as u8;
        let lo_address = (address & 0x00FF) as u8;

        // utils::bvs_16(address)
    }
}

impl Memory for Ppu {
    fn read(&self, address: u16) -> u8 {
        // PPU registers are mirrored every 8 bytes
        let address = (address & 0b0111) + 0x2000;
        if address > 0x2007 {
            panic!("Writing to a mirrored PPU register");
        }
        let data = match address {
            PPUSTATUS => {
                // Internal
                self.internal.borrow_mut().ppustatus_read();

                // Registers
                let mut registers = self.registers.borrow_mut();

                // The 5 lower bits reflect the PPU bus contents. Although
                // emulated, no games should relay on this behaviour
                let ppustatus = (registers.status.bits() & 0xE0) | (registers.data_buffer & 0x1F);

                // Reading PPU status clears VBL flag and the address latch
                registers.status.remove(PpuStatus::VERTICAL_BLANK);
                registers.data_buffer = 0;
                registers.ongoing_address_write = false;

                ppustatus | 0b1000_0000
                // ppustatus
            }

            PPUMASK => {
                // Registers
                self.registers.borrow().mask
            }

            PPUDATA => {
                // Internal

                // Registers
                let mut regs = self.registers.borrow_mut();
                regs.data = regs.data_buffer;
                regs.data_buffer = self.bus.borrow().read(regs.address);

                if regs.address >= 0x3F00 {
                    // some addresses used combinatory logic to avoid one clock
                    // delay between reading and having data available (palettes
                    // for example)
                    regs.data = regs.data_buffer;
                }

                let increment = match regs.ctrl.contains(PpuCtrl::VRAM_ADDRESS_INCREMENT) {
                    false => 1, // going across
                    true => 32, // going down
                };
                regs.address += increment;

                regs.data
            }
            _ => panic!("PPU read not implemented for address: {address:0>4X}"),
        };
        // println!("PPU read from: {address:0>4X} <- {data:0>2X}");
        data
    }

    fn write(&mut self, address: u16, data: u8) {
        // println!("PPU write to: {address:0>4X} -> {data:0>2X}");

        let address = address + 0x2000;
        let mut regs = self.registers.borrow_mut();
        match address {
            PPUCTRL => {
                // Internal
                self.internal.borrow_mut().ppuctrl_write(data);

                // Registers
                regs.ctrl = PpuCtrl::from_bits(data)
                    .unwrap_or_else(|| panic!("Invalid PPUCTRL write value: 0b{data:0>8b}"));
            }
            PPUMASK => {
                // Registers
                regs.mask = data
            }

            PPUADDR => {
                // Internal
                self.internal.borrow_mut().ppuaddr_write(data);

                // Registers

                // regs.address = ((regs.address & 0xFFF) << 8) | data as u16;

                if !regs.ongoing_address_write {
                    regs.address = (regs.address & 0x00FF) | ((data as u16) << 8);
                    regs.ongoing_address_write = true;
                } else {
                    regs.address = (regs.address & 0xFF00) | data as u16;
                    regs.ongoing_address_write = false;
                }
            }
            OAMADDR => {
                // println!("PPU ignored read to OAMADDR");
            }

            PPUSCROLL => {
                // Internal
                self.internal.borrow_mut().ppuscroll_write(data);

                // Registers
            }

            PPUDATA => {
                // Internal

                // Registers
                self.bus.borrow_mut().write(regs.address, data);

                let increment = match regs.ctrl.contains(PpuCtrl::VRAM_ADDRESS_INCREMENT) {
                    false => 1, // going across
                    true => 32, // going down
                };
                regs.address += increment;
            }

            _ => panic!("PPU write not implemented for address: {address:0>4X}"),
        }
    }

    fn size(&self) -> usize {
        // TODO: change number of mirrors to real number
        let mirrors = (0x3FFF - 0x2008 + 1) / 8;
        (0x2007 - 0x2000 + 1) * mirrors
    }
}

impl PpuInternalRegisters {
    #[inline]
    fn ppuctrl_write(&mut self, data: u8) {
        let temp_vram_addr =
            (self.temp_vram_addr.value() & !(0b11 << 10)) | ((data as u16 & 0b0000_0011) << 10);
        self.temp_vram_addr = RenderAddress::from(temp_vram_addr);
    }

    #[inline]
    fn ppustatus_read(&mut self) {
        self.write_toggle = WriteToggle::First;
    }

    #[inline]
    fn ppuscroll_write(&mut self, data: u8) {
        match self.write_toggle {
            WriteToggle::First => {
                self.temp_vram_addr = RenderAddress::from(
                    (self.temp_vram_addr.value() & !0b0001_1111) | (data as u16 & 0b1111_1000),
                );
                self.fine_x_scroll = data & 0b0000_0111;
                self.write_toggle = WriteToggle::Second;
            }
            WriteToggle::Second => {
                self.temp_vram_addr = RenderAddress::from(
                    (self.temp_vram_addr.value() & !0b0111_0011_1110_0000)
                        | (((data & 0b0000_0111) as u16) << 12)
                        | (((data & 0b1111_1000) as u16) << 5),
                );
                self.write_toggle = WriteToggle::First;
            }
        }
    }

    #[inline]
    fn ppuaddr_write(&mut self, data: u8) {
        match self.write_toggle {
            WriteToggle::First => {
                self.temp_vram_addr = RenderAddress::from(
                    (self.temp_vram_addr.value() & 0x3F00) | (((data & 0x3F) as u16) << 8),
                );
                // XXX according to nesdev, t (bit 15) = Z and bit Z is
                // cleared. What's bit Z?
                self.write_toggle = WriteToggle::Second;
            }
            WriteToggle::Second => {
                self.temp_vram_addr =
                    RenderAddress::from((self.temp_vram_addr.value() & 0x00FF) | data as u16);
                self.vram_addr = self.temp_vram_addr;
                self.write_toggle = WriteToggle::First;
            }
        }
    }
}
