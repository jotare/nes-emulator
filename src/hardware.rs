//! NES hardware constants

// Main bus
// --------
//
// Main address space for the NES. CPU, RAM and registers are mapped to this
// space.
//
// Cartidges PGR ROM and RAM are mapped to this space

// Memory - 2kB RAM mirrored 3 times (used by the CPU)
pub const RAM_START: u16 = 0x0000;
pub const RAM_END: u16 = 0x1FFF;
pub const RAM_SIZE: u16 = RAM_END - RAM_START + 1;
pub const RAM_MIRRORS: u16 = 3;

// PPU registers - 8 registers mirrored 1023 times
pub const PPU_REGISTERS_START: u16 = 0x2000;
pub const PPU_REGISTERS_END: u16 = 0x3FFF;

pub const PPUCTRL: u16 = 0x2000;
pub const PPUMASK: u16 = 0x2001;
pub const PPUSTATUS: u16 = 0x2002;
pub const OAMADDR: u16 = 0x2003;
pub const OAMDATA: u16 = 0x2004;
pub const PPUSCROLL: u16 = 0x2005;
pub const PPUADDR: u16 = 0x2006;
pub const PPUDATA: u16 = 0x2007;
pub const OAMDMA: u16 = 0x4014;

// APU and I/O REGISTERS
pub const APU_AND_IO_REGISTERS_START: u16 = 0x4000;
pub const APU_AND_IO_REGISTERS_END: u16 = 0x4015;
pub const APU_AND_IO_REGISTERS_SIZE: u16 =
    APU_AND_IO_REGISTERS_END - APU_AND_IO_REGISTERS_START + 1;

// Controllers
pub const CONTROLLER_PORT_1: u16 = 0x4016;
pub const CONTROLLER_PORT_2: u16 = 0x4017;

// Cartidge PGR ROM and RAM space
pub const CARTIDGE_EXPANSION_ROM_START: u16 = 0x4020;
pub const CARTIDGE_EXPANSION_ROM_END: u16 = 0x5FFF;
pub const CARTIDGE_EXPANSION_ROM_SIZE: u16 =
    CARTIDGE_EXPANSION_ROM_END - CARTIDGE_EXPANSION_ROM_START + 1;

pub const CARTIDGE_RAM_START: u16 = 0x6000;
pub const CARTIDGE_RAM_END: u16 = 0x7FFF;

pub const CARTIDGE_ROM_START: u16 = 0x8000;
pub const CARTIDGE_ROM_END: u16 = 0xFFFF;

// Graphics bus
// ------------
//
// Address space for the PPU and graphics. It's a 16-bit address space
// completely separated from the main bus (used by the CPU).
//
// Cartidges CHR ROM and RAM are usually mapped to this space

// Pattern tables - area of memory that defines the shapes of tiles that make up
// backgrounds and sprites. It's data is also known as CHR (from "character")
// and is attached from the cartidges
pub const PATTERN_TABLES_START: u16 = 0x0000;
pub const PATTERN_TABLES_END: u16 = 0x1FFF;

// Nametables - also known as VRAM. 1024-byte area used by the PPU to lay out
// backgrounds
pub const NAMETABLES_START: u16 = 0x2000;
pub const NAMETABLES_END: u16 = 0x2FFF;

// TODO: nametables mirrors

// Palettes - 256-byte memory storing which colors should be displayed on the
// screen when sprites and background are combined
pub const PALETTE_MEMORY_START: u16 = 0x3F00;
pub const PALETTE_MEMORY_END: u16 = 0x3F1F;
pub const PALETTE_MEMORY_SIZE: u16 = PALETTE_MEMORY_END - PALETTE_MEMORY_START + 1;
pub const PALETTE_MIRRORS: u16 = 7;
pub const PALETTE_MEMORY_MIRRORS_END: u16 = 0x3FFF;

// Screen
// ------

pub const SCREEN_HEIGHT: usize = 240;
pub const SCREEN_WIDTH: usize = 256;
