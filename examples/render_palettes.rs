//! Render palettes
//!
//! NES games have a limited set of colors. It has 4 background palettes and 4
//! sprite palettes that can use for rendering parts of the screen.
//!
//! Each palette has three colors and one backdrop color.
//!
//! This example creates a GUI to render them.
//!
//! Read more about NES palettes here:
//! https://www.nesdev.org/wiki/PPU_palettes

use nes_emulator::graphics::{Frame, FramePixel, Pixel};
use nes_emulator::hardware::{PALETTE_MEMORY_START, SCREEN_HEIGHT, SCREEN_WIDTH};
use nes_emulator::interfaces::Bus;
use nes_emulator::settings::NesSettings;
use nes_emulator::settings::UiKind;
use nes_emulator::ui::{GtkUi, Ui};
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
    // usually loaded by software. Starting a NES and rendering the palettes
    // might give a different result after some clock time.
    //
    // To avoid this, we need to run some clocks to obtain the proper colors
    // before rendering
    const ONE_CLOCK_SECOND: usize = 21_470_000;
    for _ in 0..(ONE_CLOCK_SECOND / 4) {
        nes.clock().unwrap();
    }

    const PIXEL_SCALE_FACTOR: usize = 4;
    let mut ui = GtkUi::builder()
        .pixel_scale_factor(PIXEL_SCALE_FACTOR)
        .build();

    ui.start().unwrap();

    let frame = render_palettes(&nes);
    ui.render(frame);

    ui.stop().unwrap();
}

fn render_palettes(nes: &Nes) -> Frame {
    let mut frame = Frame::default();

    const COLORS_PER_PALETTE: usize = 4;
    const COLOR_ROW_HEIGHT: usize = SCREEN_HEIGHT / COLORS_PER_PALETTE / 2;

    for palette in 0..8 {
        let palette_address = PALETTE_MEMORY_START as usize + palette * 4;

        for palette_color_number in 0..4 {
            let address = (palette_address + palette_color_number) as u16;
            let palette_color = nes.graphics_bus().read(address);
            let color = Pixel::from(palette_color);

            paint(
                &mut frame,
                color,
                Rectangle {
                    top_left: FramePixel {
                        row: palette * COLOR_ROW_HEIGHT,
                        col: palette_color_number * (SCREEN_WIDTH / 4),
                    },
                    bottom_right: FramePixel {
                        row: palette * COLOR_ROW_HEIGHT + COLOR_ROW_HEIGHT - 1,
                        col: (palette_color_number + 1) * (SCREEN_WIDTH / 4) - 1,
                    },
                },
            );
        }
    }

    frame
}

struct Rectangle {
    top_left: FramePixel,
    bottom_right: FramePixel,
}

fn paint(frame: &mut Frame, color: Pixel, rectangle: Rectangle) {
    for row in rectangle.top_left.row..=rectangle.bottom_right.row {
        for col in rectangle.top_left.col..=rectangle.bottom_right.col {
            frame.set_pixel(color, FramePixel { row, col });
        }
    }
}
