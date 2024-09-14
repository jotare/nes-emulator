//! Render pattern tables
//!
//! This example crates a NES, loads a game and render both pattern tables in a
//! GUI.
//!
//! Read more about NES pattern tables and palettes here:
//! https://www.nesdev.org/wiki/PPU_pattern_tables
//! https://www.nesdev.org/wiki/PPU_palettes
//!

use nes_emulator::graphics::pattern_table::PatternTableAddress;
use nes_emulator::graphics::{Frame, FramePixel, Pixel};
use nes_emulator::hardware::PALETTE_MEMORY_START;
use nes_emulator::interfaces::Bus;
use nes_emulator::settings::NesSettings;
use nes_emulator::settings::UiKind;
use nes_emulator::ui::{GtkUi, Ui};
use nes_emulator::utils;
use nes_emulator::{Cartidge, Nes};

// ATENTION! ROMs are not provided in this repository, you should download your
// owns and change this path.
const CARTIDGE_PATH: &str = "roms/Super Mario Bros. (World).nes";

fn main() {
    let mut nes = Nes::new(NesSettings {
        ui_kind: UiKind::None,
        ..Default::default()
    });
    let cartidge = Cartidge::new(CARTIDGE_PATH);

    nes.load_cartidge(cartidge);

    // NES games load their colors to the palette memory region and this is
    // usually loaded by software. Starting a NES and rendering the pattern
    // tables might give a different result after some clock time.
    //
    // To avoid this, we need to run some clocks to obtain the proper colors
    // before rendering the pattern tables
    const ONE_CLOCK_SECOND: usize = 21_470_000;
    for _ in 0..(ONE_CLOCK_SECOND / 4) {
        nes.clock().unwrap();
    }

    // Create a GUI to display the pattern tables. Height and width are chosen
    // so the frame bottom is not shown (as it's black).
    const HEIGHT: usize = 128;
    const WIDTH: usize = HEIGHT * 2;
    const PIXEL_SCALE_FACTOR: usize = 7;
    let mut ui = GtkUi::builder()
        .screen_size(WIDTH, HEIGHT)
        .pixel_scale_factor(PIXEL_SCALE_FACTOR)
        .build();

    ui.start();

    // NES have 8 palettes, 4 for background colors (palettes 0 to 3) and 4 for
    // sprites (palettes 4 to 7)
    let palette: u8 = 4;
    assert!(palette <= 7);

    let frame = render_pattern_tables(&nes, palette);
    ui.render(frame);

    ui.join();
}

/// Generate a [`Frame`] with the two NES pattern tables.
///
/// As pattern tables are square, some margin will be shown black at the bottom.
fn render_pattern_tables(nes: &Nes, palette: u8) -> Frame {
    const TILES_PER_PATTERN_TABLE: usize = 256;
    const PATTERN_TABLE_WIDTH: usize = 128;

    let mut frame = Frame::black();

    for (pattern_table, offset) in [(0, 0), (1, PATTERN_TABLE_WIDTH)] {
        let mut pattern_table_address = PatternTableAddress::new(pattern_table);

        for tile_number in 0..TILES_PER_PATTERN_TABLE {
            pattern_table_address.set(PatternTableAddress::TILE_NUMBER, tile_number as u8);

            for x in 0..8usize {
                for y in 0..8usize {
                    pattern_table_address.set(PatternTableAddress::FINE_Y_OFFSET, y as u8);

                    pattern_table_address.set(PatternTableAddress::BIT_PLANE, 0);
                    let low = nes.graphics_bus.borrow().read(pattern_table_address.into());

                    pattern_table_address.set(PatternTableAddress::BIT_PLANE, 1);
                    let high = nes.graphics_bus.borrow().read(pattern_table_address.into());

                    let palette_offset =
                        (palette << 2) | utils::bv(high, x as u8) << 1 | utils::bv(low, x as u8);
                    let palette_color = nes
                        .graphics_bus
                        .borrow()
                        .read(PALETTE_MEMORY_START + palette_offset as u16);
                    let color = Pixel::from(palette_color);

                    let row = (tile_number / 16) * 8 + y;
                    let col = (tile_number % 16) * 8 + (7 - x) + offset;

                    frame.set_pixel(color, FramePixel { row, col });
                }
            }
        }
    }

    frame
}
