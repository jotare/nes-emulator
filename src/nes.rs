/// Nintendo Entertainment System (NES) abstraction.
///
/// This module defines the higher level abstractions to run the NES
/// emulator. It defines the video game console `Nes` and `Cartidges`
/// representing games. To use it, create a Nes instance, create a
/// Cartidge from a ROM file, put the game on the machine and `run` to
/// start playing!
///
///
use std::cell::RefCell;
use std::rc::Rc;

use log::{error, info};

use crate::cartidge::Cartidge;
use crate::graphics::ppu::Ppu;
use crate::graphics::ui::gtk_ui::GtkUi;
use crate::graphics::ui::{Frame, Pixel, ORIGINAL_SCREEN_HEIGHT, ORIGINAL_SCREEN_WIDTH};
use crate::interfaces::Bus as BusTrait;
use crate::interfaces::{AddressRange, Processor};
use crate::processor::bus::Bus;
use crate::processor::cpu::Cpu;
use crate::processor::memory::{MirroredRam, Ram, Rom};

type NesBus = Rc<RefCell<Bus>>;
type NesCpu = Rc<RefCell<Cpu>>;

pub struct Nes {
    cartidge: Option<Cartidge>,

    cpu: Cpu,
    main_bus: NesBus,

    ppu: Ppu,
    graphics_bus: NesBus,

    ui: GtkUi,
}

impl Nes {
    pub fn new() -> Self {

        let main_bus = Rc::new(RefCell::new(Bus::new()));
        let graphics_bus = Rc::new(RefCell::new(Bus::new()));

        let main_bus_ptr = Rc::clone(&main_bus);
        let cpu = Cpu::new(main_bus_ptr);

        let graphics_bus_ptr = Rc::clone(&graphics_bus);
        let ppu = Ppu::new(graphics_bus_ptr);

        // Memory - 2 kB RAM mirrored 3 times. It's used by the CPU
        let ram = Box::new(MirroredRam::new(0x0800, 3));
        main_bus.borrow_mut().attach(
            ram,
            AddressRange {
                start: 0,
                end: 0x1FFF,
            },
        );

        // Pattern memory - also known as CHR ROM is a 8 kB memory where two
        // pattern tables are stored. It contains all graphical information the
        // PPU require to drawIt's main purpose is sprite storage.
        //
        // It can be split into two 4 kB (0x1000) sections containing the
        // pattern tables 0 and 1
        let pattern_memory = Box::new(Rom::new(0x2000));
        graphics_bus.borrow_mut().attach(
            pattern_memory,
            AddressRange {
                start: 0,
                end: 0x1FFF,
            },
        );

        // Name table memory - also known as VRAM
        let name_table_memory = Box::new(Ram::new(0x3EFF - 0x2000 + 1));
        graphics_bus.borrow_mut().attach(
            name_table_memory,
            AddressRange {
                start: 0x2000,
                end: 0x3EFF,
            },
        );

        // Palette memory - 256-byte memory. It stores which colors should be
        // displayed on the screen when spites and background are combined
        let palette_memory = Box::new(Ram::new(0x3FFF - 0x3F00 + 1));
        graphics_bus.borrow_mut().attach(
            palette_memory,
            AddressRange {
                start: 0x3F00,
                end: 0x3FFF,
            },
        );

        // ----------------------------------------------------------------------------------------

        let fake_ppu = Box::new(MirroredRam::new(8, 1023)); // 8 B mirrored RAM
        main_bus.borrow_mut().attach(
            fake_ppu,
            AddressRange {
                start: 0x2000,
                end: 0x3FFF,
            },
        );

        let fake_apu = Box::new(Ram::new(0x18)); // 0x18 B RAM - NES APU and I/O registers
        main_bus.borrow_mut().attach(
            fake_apu,
            AddressRange {
                start: 0x4000,
                end: 0x4017,
            },
        );

        let ui = GtkUi::new();

        Self {
            cpu,
            main_bus,
            ppu,
            graphics_bus,
            cartidge: None,
            ui,
        }
    }

    pub fn load_cartidge(&mut self, cartidge: Cartidge) {
        info!("Cartidge inserted: {}", cartidge);

        let ram = cartidge.program_ram.clone();
        let rom = cartidge.program_rom.clone();

        // XXX: use references to avoid cloning memory
        self.main_bus.borrow_mut().attach(
            Box::new(ram),
            AddressRange {
                start: 0x6000,
                end: 0x7FFF,
            },
        );
        // XXX: use references to avoid cloning memory
        self.main_bus.borrow_mut().attach(
            Box::new(rom),
            AddressRange {
                start: 0x8000,
                end: 0xFFFF,
            },
        );

        self.cartidge = Some(cartidge);
        self.cpu.reset();
    }

    /// Blocking NES run
    pub fn run(&mut self) {
        info!("NES indefinedly running game");

        self.ui.start();

        let inter_frame_delay = std::time::Duration::from_millis(16);

        'outer: loop {
            for direction in [true, false] {
                for step in 0..40 {
                    let result = self.cpu.execute();
                    if let Err(error) = result {
                        error!("CPU execution error: {}", error);
                        break 'outer;
                    }

                    let frame = self.colors_animation_frame(step, direction);
                    self.ui.render(frame);
                    std::thread::sleep(inter_frame_delay);
                }
            }
        }

        self.ui.join();
    }

    fn colors_animation_frame(&self, step: usize, forwards: bool) -> Frame {
        fn compute_coloured_pixel(x: usize, y: usize, factor: f64, forwards: bool) -> Pixel {
            let color = 1.0 / ((x + y) as f64 / 10.0 / factor);
            if forwards {
                Pixel::new_rgb(1.0, 1.0 - color, color)
            } else {
                Pixel::new_rgb(1.0, color, 1.0 - color)
            }
        }

        let mut frame =
            vec![[Pixel::new_rgb(0.0, 0.0, 0.0); ORIGINAL_SCREEN_WIDTH]; ORIGINAL_SCREEN_HEIGHT];

        for y in 0..ORIGINAL_SCREEN_HEIGHT {
            for x in 0..ORIGINAL_SCREEN_WIDTH {
                if forwards {
                    frame[y][x] = compute_coloured_pixel(x, y, step as f64, forwards);
                } else {
                    frame[ORIGINAL_SCREEN_HEIGHT - y - 1][ORIGINAL_SCREEN_WIDTH - x - 1] =
                        compute_coloured_pixel(x, y, step as f64, forwards);
                }
            }
        }

        frame
    }

    fn clock(&self) {
        todo!("Implement clocks in CPU and PPU");
    }
}

impl Default for Nes {
    fn default() -> Self {
        Self::new()
    }
}
