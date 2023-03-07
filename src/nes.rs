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
use crate::interfaces::AddressRange;
use crate::interfaces::Bus as BusTrait;
use crate::processor::bus::Bus;
use crate::processor::cpu::Cpu;
use crate::processor::memory::{MirroredRam, Ram};
use crate::types::{SharedBus, SharedMemory, SharedPpu};

pub struct Nes {
    // XXX: change to u128 if overflow occur
    system_clock: u64,

    cartidge: Option<Cartidge>,

    cpu: Cpu,
    main_bus: SharedBus,

    ppu: SharedPpu,
    graphics_bus: SharedBus,

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

        let ram = cartidge.mapper.program_ram_ref();
        let rom = cartidge.mapper.program_rom_ref();
        let chr = cartidge.mapper.character_memory_ref();

        self.main_bus.borrow_mut().attach(
            ram,
            AddressRange {
                start: 0x6000,
                end: 0x7FFF,
            },
        );

        self.main_bus.borrow_mut().attach(
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
    pub fn run(&mut self) -> Result<(), String> {
        info!("NES indefinedly running game");


        self.ui.start();

        loop {
            self.clock()?;
        }

        Ok(())
    }

    /// NES system clocks runs at ~21.47 MHz
    fn clock(&mut self) -> Result<(), String> {
        self.system_clock += 1;

        // PPU clock runs every 4 system clocks
        if self.system_clock % 4 == 0 {
            self.ppu.borrow_mut().clock();

            {
                let ppu = self.ppu.borrow();
                if ppu.frame_ready() {
                    let color = (self.system_clock / 4 % (u8::MAX as u64 + 1)) as u8;
                    let frame = ppu.get_frame(color);
                    self.ui.render(frame);
                    std::thread::sleep(std::time::Duration::from_millis(16));
                }
            }
        }

        // CPU clock runs every 12 system clocks
        if self.system_clock % 12 == 0 {
            self.cpu.clock()?;
        }

        Ok(())
    }
}

impl Default for Nes {
    fn default() -> Self {
        Self::new()
    }
}
