use std::time::Duration;

use nes_emulator::events::{Event, SharedEventBus};
use nes_emulator::graphics::{Frame, FramePixel, Pixel};
use nes_emulator::hardware::{SCREEN_HEIGHT, SCREEN_WIDTH};
use nes_emulator::ui::{GtkUi, Ui};

fn main() {
    const INTER_FRAME_DELAY: Duration = Duration::from_millis(16);

    let event_bus = SharedEventBus::new();
    let mut ui = GtkUi::builder().with_event_bus(event_bus.clone()).build();
    ui.start();

    'outer: loop {
        for direction in [true, false] {
            for step in 0..160 {
                let frame = colors_animation_frame(step, direction);
                ui.render(frame);
                std::thread::sleep(INTER_FRAME_DELAY);

                if event_bus.access().emitted(Event::SwitchOff) {
                    break 'outer;
                }
            }
        }
    }

    ui.join();
}

fn colors_animation_frame(step: usize, forwards: bool) -> Frame {
    let mut frame = Frame::black();

    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            if forwards {
                frame.set_pixel(
                    compute_coloured_pixel(x, y, step as f64, forwards),
                    FramePixel { row: y, col: x },
                )
            } else {
                frame.set_pixel(
                    compute_coloured_pixel(x, y, step as f64, forwards),
                    FramePixel {
                        row: SCREEN_HEIGHT - y - 1,
                        col: SCREEN_WIDTH - x - 1,
                    },
                )
            }
        }
    }

    frame
}

fn compute_coloured_pixel(x: usize, y: usize, factor: f64, forwards: bool) -> Pixel {
    let color = 1.0 / ((x + y) as f64 / 2.0 / factor);
    if forwards {
        Pixel::new_rgb(1.0, 1.0 - color, color)
    } else {
        Pixel::new_rgb(1.0, color, 1.0 - color)
    }
}
