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

use crossbeam_channel::unbounded;
use log::{error, info};

use crate::cartidge::Cartidge;
use crate::controller::Controller;
use crate::controller::ControllerButtons;
use crate::graphics::palette_memory::PaletteMemory;
use crate::graphics::ppu::Ppu;
use crate::graphics::ui::gtk_ui::GtkUi;
use crate::graphics::ui::{Frame, Pixel, ORIGINAL_SCREEN_HEIGHT, ORIGINAL_SCREEN_WIDTH};
use crate::hardware::*;
use crate::interfaces::AddressRange;
use crate::interfaces::Bus as BusTrait;
use crate::processor::bus::Bus;
use crate::processor::cpu::{Cpu, Interrupt};
use crate::processor::memory::{Ciram, MirroredRam, Ram};
use crate::types::{SharedBus, SharedCiram, SharedController, SharedMemory, SharedPpu};

pub struct Nes {
    // XXX: change to u128 if overflow occur
    system_clock: u64,

    cartidge: Option<Cartidge>,

    cpu: Cpu,
    main_bus: SharedBus,

    ppu: SharedPpu,
    graphics_bus: SharedBus,

    ram: SharedMemory,
    nametable: SharedCiram,
    palettes: SharedMemory,

    ui: GtkUi,

    controller_one: SharedController,
    controller_two: SharedController,
}

impl Nes {
    pub fn new() -> Self {
        let main_bus = Rc::new(RefCell::new(Bus::new("CPU")));
        let graphics_bus = Rc::new(RefCell::new(Bus::new("PPU")));

        let main_bus_ptr = Rc::clone(&main_bus);
        let cpu = Cpu::new(main_bus_ptr);

        let graphics_bus_ptr = Rc::clone(&graphics_bus);
        let ppu = Rc::new(RefCell::new(Ppu::new(graphics_bus_ptr)));

        let (sender, receiver_one) = unbounded();
        let receiver_two = receiver_one.clone();

        let ui = GtkUi::builder().keyboard_channel(sender).build();

        // Main Bus
        // ----------------------------------------------------------------------------------------

        // Memory - 2 kB RAM mirrored 3 times. It's used by the CPU
        let ram = Rc::new(RefCell::new(MirroredRam::new(0x0800, 3)));
        let ram_ptr = Rc::clone(&ram);
        main_bus.borrow_mut().attach(
            "RAM",
            ram_ptr,
            AddressRange {
                start: 0,
                end: 0x1FFF,
            },
        ).unwrap();

        let ppu_ptr = Rc::clone(&ppu); // The 8 PPU registers are mirrored 1023 times
        main_bus.borrow_mut().attach(
            "PPU registers",
            ppu_ptr,
            AddressRange {
                start: 0x2000,
                // end: 0x2007,
                end: 0x3FFF,
            },
        ).unwrap();

        let fake_apu = Rc::new(RefCell::new(Ram::new(0x18))); // 0x18 B RAM - NES APU and I/O registers
        let fake_apu_ptr = Rc::clone(&fake_apu);
        main_bus.borrow_mut().attach(
            "Fake APU",
            fake_apu_ptr,
            AddressRange {
                start: 0x4000,
                end: 0x4015,
            },
        ).unwrap();

        let controller_one = Rc::new(RefCell::new(Controller::new(receiver_one, false)));
        let controller_one_ptr = Rc::clone(&controller_one);
        main_bus.borrow_mut().attach(
            "Controller 1",
            controller_one_ptr,
            AddressRange {
                start: 0x4016,
                end: 0x4016,
            },
        ).unwrap();

        let controller_two = Rc::new(RefCell::new(Controller::new(receiver_two, false)));
        let controller_two_ptr = Rc::clone(&controller_two);
        main_bus.borrow_mut().attach(
            "Controller 2",
            controller_two_ptr,
            AddressRange {
                start: 0x4017,
                end: 0x4017,
            },
        ).unwrap();

        let cartidge_expansion_rom = Rc::new(RefCell::new(Ram::new(0x5FFF - 0x4020 + 1)));
        let cartidge_expansion_rom_ptr = Rc::clone(&cartidge_expansion_rom);
        main_bus.borrow_mut().attach(
            "Cartidge Expansion ROM",
            cartidge_expansion_rom_ptr,
            AddressRange {
                start: 0x4020,
                end: 0x5FFF,
            },
        ).unwrap();

        // Graphics Bus
        // ----------------------------------------------------------------------------------------

        // Pattern tables - attached from cartidge from 0x0000 to 0x1FFF

        // Name table memory - also known as VRAM
        let nametable = Rc::new(RefCell::new(Ciram::new(0x0400))); // 2 kB mirrored
        let name_table_memory_ptr = Rc::clone(&nametable);
        graphics_bus.borrow_mut().attach(
            "Nametables",
            name_table_memory_ptr,
            AddressRange {
                start: 0x2000,
                end: 0x2FFF,
            },
        ).unwrap();

        // Palette memory - 256-byte memory. It stores which colors should be
        // displayed on the screen when spites and background are combined
        let palette_memory = Rc::new(RefCell::new(PaletteMemory::new()));
        let palette_memory_ptr = Rc::clone(&palette_memory);
        graphics_bus.borrow_mut().attach(
            "Palettes",
            palette_memory_ptr,
            AddressRange {
                start: PALETTE_MEMORY_START,
                end: PALETTE_MEMORY_END,
            },
        ).unwrap();

        // ----------------------------------------------------------------------------------------

        Self {
            system_clock: 0,
            cartidge: None,
            cpu,
            main_bus,
            ppu,
            graphics_bus,
            ram,
            nametable,
            palettes: palette_memory,
            ui,
            controller_one,
            controller_two,
        }
    }

    pub fn load_cartidge(&mut self, cartidge: Cartidge) {
        info!("Cartidge inserted: {}", cartidge);

        let ram = cartidge.mapper.program_ram_ref();
        let rom = cartidge.mapper.program_rom_ref();
        let chr = cartidge.mapper.character_memory_ref();

        self.main_bus.borrow_mut().attach(
            "Cartidge RAM",
            ram,
            AddressRange {
                start: 0x6000,
                end: 0x7FFF,
            },
        ).unwrap();

        self.main_bus.borrow_mut().attach(
            "Cartidge ROM",
            rom,
            AddressRange {
                start: 0x8000,
                end: 0xFFFF,
            },
        ).unwrap();

        // Pattern memory - also known as CHR ROM is a 8 kB memory where two
        // pattern tables are stored. It contains all graphical information the
        // PPU require to draw.
        //
        // It can be split into two 4 kB (0x1000) sections containing the
        // pattern tables 0 and 1
        self.graphics_bus.borrow_mut().attach(
            "CHR ROM (pattern memories)",
            chr,
            AddressRange {
                start: 0x0000,
                end: 0x1FFF,
            },
        ).unwrap();

        self.ppu.borrow_mut().set_mirroring(cartidge.mirroring());
        self.nametable
            .borrow_mut()
            .set_mirroring(cartidge.mirroring());

        self.cartidge = Some(cartidge);
        self.cpu.reset();
    }

    pub fn connect_controller_one(&mut self, buttons: ControllerButtons) {
        self.controller_one.borrow_mut().connect(buttons);
    }

    pub fn disconnect_controller_one(&mut self) {
        self.controller_one.borrow_mut().disconnect();
    }

    /// Blocking NES run
    pub fn run(&mut self) -> Result<(), String> {
        info!("NES indefinedly running game");

        self.ui.start();

        loop {
            self.clock()?;
        }
    }

    /// NES system clocks runs at ~21.47 MHz
    fn clock(&mut self) -> Result<(), String> {
        self.system_clock += 4;

        // PPU clock runs every 4 system clocks
        if self.system_clock % 4 == 0 {
            let mut ppu = self.ppu.borrow_mut();
            ppu.clock();

            if ppu.is_nmi_requested() {
                self.cpu.interrupt(Interrupt::NonMaskableInterrupt);
                ppu.nmi_accepted();
            }

            if ppu.frame_ready() {
                let frame = ppu.take_frame();
                self.ui.render(frame);
                std::thread::sleep(std::time::Duration::from_millis(16));
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
