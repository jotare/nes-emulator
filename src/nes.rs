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
use crate::interfaces::{AddressRange, Memory, Processor};
use crate::processor::bus::Bus;
use crate::processor::cpu::Cpu;
use crate::processor::memory::{MirroredRam, Ram};

type NesPpu = Rc<RefCell<Ppu>>;
type NesBus = Rc<RefCell<Bus>>;
type SharedMemory = Rc<RefCell<dyn Memory>>;

pub struct Nes {
    // XXX: change to u128 if overflow occur
    system_clock: u64,

    cartidge: Option<Cartidge>,

    cpu: Cpu,
    main_bus: NesBus,

    ppu: NesPpu,
    graphics_bus: NesBus,

    ram: SharedMemory,
    name_table: SharedMemory,
    palettes: SharedMemory,

    ui: GtkUi,
}

impl Nes {
    pub fn new() -> Self {

        let main_bus = Rc::new(RefCell::new(Bus::new()));
        let graphics_bus = Rc::new(RefCell::new(Bus::new()));

        let main_bus_ptr = Rc::clone(&main_bus);
        let cpu = Cpu::new(main_bus_ptr);

        let graphics_bus_ptr = Rc::clone(&graphics_bus);
        let ppu = Rc::new(RefCell::new(Ppu::new(graphics_bus_ptr)));

        // Memory - 2 kB RAM mirrored 3 times. It's used by the CPU
        let ram = Rc::new(RefCell::new(MirroredRam::new(0x0800, 3)));
        let ram_ptr = Rc::clone(&ram);
        main_bus.borrow_mut().attach(
            ram_ptr,
            AddressRange {
                start: 0,
                end: 0x1FFF,
            },
        );

        // Name table memory - also known as VRAM
        let name_table_memory = Rc::new(RefCell::new(Ram::new(0x3EFF - 0x2000 + 1)));
        let name_table_memory_ptr = Rc::clone(&name_table_memory);
        graphics_bus.borrow_mut().attach(
            name_table_memory_ptr,
            AddressRange {
                start: 0x2000,
                end: 0x3EFF,
            },
        );

        // Palette memory - 256-byte memory. It stores which colors should be
        // displayed on the screen when spites and background are combined
        let palette_memory = Rc::new(RefCell::new(Ram::new(0x3FFF - 0x3F00 + 1)));
        let palette_memory_ptr = Rc::clone(&palette_memory);
        graphics_bus.borrow_mut().attach(
            palette_memory_ptr,
            AddressRange {
                start: 0x3F00,
                end: 0x3FFF,
            },
        );

        // ----------------------------------------------------------------------------------------

        // let fake_ppu = Box::new(MirroredRam::new(8, 1023)); // 8 B mirrored RAM
        // main_bus.borrow_mut().attach(
        //     fake_ppu,
        //     AddressRange {
        //         start: 0x2000,
        //         end: 0x3FFF,
        //     },
        // );

        let ppu_ptr = Rc::clone(&ppu);
        main_bus.borrow_mut().attach(
            ppu_ptr,
            AddressRange {
                start: 0x2000,
                end: 0x2007,
            },
        );

        let fake_apu = Rc::new(RefCell::new(Ram::new(0x18))); // 0x18 B RAM - NES APU and I/O registers
        let fake_apu_ptr = Rc::clone(&fake_apu);
        main_bus.borrow_mut().attach(
            fake_apu_ptr,
            AddressRange {
                start: 0x4000,
                end: 0x4017,
            },
        );

        let ui = GtkUi::default();

        Self {
            system_clock: 0,
            cartidge: None,
            cpu,
            main_bus,
            ppu,
            graphics_bus,
            ram,
            name_table: name_table_memory,
            palettes: palette_memory,
            ui,
        }
    }

    pub fn load_cartidge(&mut self, cartidge: Cartidge) {
        info!("Cartidge inserted: {}", cartidge);

        let ram = Rc::clone(&cartidge.program_ram);
        let rom = Rc::clone(&cartidge.program_rom);
        let chr = Rc::clone(&cartidge.character_memory);

        self.main_bus.borrow_mut().attach(
            // XXX: use references to avoid cloning memory
            ram,
            AddressRange {
                start: 0x6000,
                end: 0x7FFF,
            },
        );

        self.main_bus.borrow_mut().attach(
            // XXX: use references to avoid cloning memory
            rom,
            AddressRange {
                start: 0x8000,
                end: 0xFFFF,
            },
        );

        // Pattern memory - also known as CHR ROM is a 8 kB memory where two
        // pattern tables are stored. It contains all graphical information the
        // PPU require to draw.
        //
        // It can be split into two 4 kB (0x1000) sections containing the
        // pattern tables 0 and 1
        self.graphics_bus.borrow_mut().attach(
            // XXX: use references to avoid cloning memory
            chr,
            AddressRange {
                start: 0x0000,
                end: 0x1FFF,
            },
        );

        self.cartidge = Some(cartidge);
        self.cpu.reset();
    }

    /// Blocking NES run
    pub fn run(&mut self) {
        info!("NES indefinedly running game");

        let cpu_enable = false;
        let ui_enable = true;

        if ui_enable {
            self.ui.start();
        }

        let inter_frame_delay = std::time::Duration::from_millis(16);
        // let inter_frame_delay = std::time::Duration::from_millis(1000);

        if cpu_enable || ui_enable {
            'outer: loop {
                for direction in [true, false] {
                    for step in 0..160 {
                        if cpu_enable {
                            let result = self.cpu.execute();
                            if let Err(error) = result {
                                error!("CPU execution error: {}", error);
                                break 'outer;
                            }
                        }

                        if ui_enable {
                            let frame = self.colors_animation_frame(step, direction);
                            self.ui.render(frame);
                            std::thread::sleep(inter_frame_delay);
                        }
                    }
                }
            }
        }

        if ui_enable {
            self.ui.join();
        }
    }

    fn colors_animation_frame(&self, step: usize, forwards: bool) -> Frame {
        fn compute_coloured_pixel(x: usize, y: usize, factor: f64, forwards: bool) -> Pixel {
            let color = 1.0 / ((x + y) as f64 / 2.0 / factor);
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
