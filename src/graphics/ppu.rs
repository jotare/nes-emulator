/// PPU module
///
/// This module emulates the NES Picture Processing Unit (PPU)
///
/// NES PPU registers ($2000-$2007) are mirrored from $2008 to $3FFF. That's
/// because it's address is not completely decoded, that is, the chip ignores
/// one or more address lines. This allows a cheaper hardware (less address
/// lines) and a faster decoding at expense of unused address space.
///
///
///
use std::cell::RefCell;
use std::rc::Rc;

use bitflags::bitflags;

use crate::graphics::palette::Palette;
use crate::graphics::ui::Frame;
use crate::graphics::ui::Pixel;
use crate::interfaces::{Bus, Memory};
use crate::processor::memory::Mirroring;
use crate::types::SharedBus;
use crate::utils;

const PPUCTRL: u16 = 0x2000;

bitflags! {
    struct PpuCtrl: u8 {
        /// Generate an NMI ate the start of the vertical blanking interval
        const NMI_ENABLE = 0b1000_0000;

        // TODO: PPU master/slave select

        /// 0: 8x8 pixels; 1: 8x16 pixels
        const SPRITE_SIZE = 0b0010_0000;

        /// Backgroun pattern table address (0 = $0000; 1 = $1000)
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

const PPUMASK: u16 = 0x2001;

const PPUSTATUS: u16 = 0x2002;

bitflags! {
    struct PpuStatus: u8 {
        // PPU is in VBL status
        const VERTICAL_BLANK = 0b1000_0000;
    }
}

const OAMADDR: u16 = 0x2003;
const OAMDATA: u16 = 0x2004;
const PPUSCROLL: u16 = 0x2005;
const PPUADDR: u16 = 0x2006;
const PPUDATA: u16 = 0x2007;
const OAMDMA: u16 = 0x4014;

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

pub struct Ppu {
    registers: RefCell<PpuRegisters>,
    bus: SharedBus,
    cycle: u16,
    scan_line: i16,
    frame_completed: bool,
    palette: Palette,
    mirroring: Mirroring,
    nmi_request: bool,
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
            bus,
            cycle: 0,
            scan_line: 0,
            frame_completed: false,
            palette: Palette::new(),
            mirroring: Mirroring::Horizontal,
            nmi_request: false,
        }
    }

    pub fn clock(&mut self) {
        // Screen rendering never stops
        self.cycle += 1;

        if self.cycle >= 341 {
            self.cycle = 0;
            self.scan_line += 1;

            if self.scan_line >= 261 {
                self.scan_line = -1;
                self.frame_completed = true;
            }
        }

        if self.scan_line == -1 && self.cycle == 1 {
            self.set_vertical_blank(false);
        } else if self.scan_line == 241 && self.cycle == 1 {
            self.set_vertical_blank(true);
            if self.registers.borrow().ctrl.contains(PpuCtrl::NMI_ENABLE) {
                self.request_nmi();
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

        println!("PPU rendering a frame");
        self.render_nametable()
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

    fn set_vertical_blank(&mut self, value: bool) {
        self.registers
            .borrow_mut()
            .status
            .set(PpuStatus::VERTICAL_BLANK, value);
    }

    fn render_pattern_tables(&self) -> Frame {
        let width = 256;
        let height = 240;

        let mut frame = vec![vec![Pixel::new_rgb(0.0, 0.0, 0.0); width]; height];

        for pattern_table in [0, 1] {
            let (palette_address, offset) = match pattern_table {
                0 => (0x0000, 0),
                1 => (0x1000, 16),
                _ => panic!("There's no pattern table {pattern_table}"),
            };

            for tile_number in 0..256 {
                let mut bit_planes = [0; 16];
                for i in 0..16 {
                    bit_planes[i] = self
                        .bus
                        .borrow()
                        .read(palette_address + (tile_number * 16 + i) as u16);
                }

                for x in 0..8 {
                    for y in 0..8 {
                        let pattern = (utils::bv(bit_planes[y + 8], x as u8) << 1)
                            | (utils::bv(bit_planes[y], x as u8));
                        let palette_number = 0;
                        // let color = (palette_number << 2) + pattern;
                        // let pixel = palette.decode_pixel(color);
                        let color = self
                            .bus
                            .borrow()
                            .read(0x3F00 + ((palette_number << 2) | pattern) as u16);
                        let pixel = self.palette.decode_pixel(color);

                        let row = (tile_number / 16) * 8 + y;
                        let col = ((tile_number % 16) + offset) * 8 + (7 - x);
                        frame[row][col] = pixel;
                    }
                }
            }
        }

        frame
    }

    fn render_nametable(&self) -> Frame {
        let width = 256;
        let height = 240;
        let mut screen = vec![vec![Pixel::new_rgb(0.0, 0.0, 0.0); width]; height];
        let palette = Palette::new();

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
        let attribute_table: Vec<u8> = Vec::with_capacity(64);
        for i in 0..64 {
            let byte = self.bus.borrow().read(attribute_table_address + i);
        }

        println!(
            "Pattern table: {pattern_table}. Nametable: {nametable}. Mirroring: {0:?}",
            self.mirroring
        );

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
                        let pixel = palette.decode_pixel(color);

                        // let mrow = (tile_number / 16) * 8 + y;
                        // let mcol = ((tile_number % 16) + offset) * 8 + (7 - x);
                        let mrow = (row as usize * 8 + y) as usize;
                        let mcol = (col as usize * 8 + (7 - x)) as usize;
                        screen[mrow][mcol] = pixel;
                    }
                }
            }
        }
        screen
    }

    fn render_palettes(&self) -> Frame {
        let width = 256;
        let height = 240;
        let default_pixel = Pixel::new_rgb_byte(0, 0, 0);

        let mut frame = vec![vec![default_pixel; width]; height];

        let mut row = 0;
        for address in 0x3F00..0x3F20 {
            let palette = self.bus.borrow().read(address);
            let color = self.palette.decode_pixel(palette);
            for i in 0..4 {
                frame[row + i] = vec![color; width];
            }

            row += 4;
        }

        frame
    }

    fn address_to_pattern_table(&self, address: u16) {
        let hi_address = ((address & 0xFF00) >> 8) as u8;
        let lo_address = (address & 0x00FF) as u8;

        // utils::bvs_16(address)
    }
}

impl Memory for Ppu {
    fn read(&self, address: u16) -> u8 {
        let address = address + 0x2000;
        if address > 0x2007 {
            panic!("Writing to a mirrored PPU register");
        }
        let data = match address {
            PPUSTATUS => {
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
            PPUDATA => {
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
                println!("Write to PPUCTRL: {data:0>8b}");
                regs.ctrl = PpuCtrl::from_bits(data)
                    .unwrap_or_else(|| panic!("Invalid PPUCTRL write value: 0b{data:0>8b}"));
            }
            PPUMASK => regs.mask = data,
            PPUADDR => {
                // regs.address = ((regs.address & 0xFFF) << 8) | data as u16;

                if !regs.ongoing_address_write {
                    regs.address = (regs.address & 0x00FF) | ((data as u16) << 8);
                    regs.ongoing_address_write = true;
                } else {
                    regs.address = (regs.address & 0xFF00) | data as u16;
                    regs.ongoing_address_write = false;
                }
            }
            OAMADDR | PPUSCROLL => {
                // println!("PPU ignored read to OAMADDR | PPUSCROLL");
            }
            PPUDATA => {
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
        let mirrors = 1;
        (0x2007 - 0x2000 + 1) * mirrors
    }
}
