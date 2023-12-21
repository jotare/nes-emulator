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
use crate::hardware::*;
use crate::interfaces::AddressRange;
use crate::interfaces::Bus as BusTrait;
use crate::processor::bus::Bus;
use crate::processor::cpu::{Cpu, Interrupt};
use crate::processor::memory::{Ciram, MirroredRam, Ram};
use crate::types::{SharedBus, SharedCiram, SharedController, SharedMemory, SharedPpu};
use crate::ui::{GtkUi, Ui};

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

        let ram = Rc::new(RefCell::new(MirroredRam::new(
            (RAM_SIZE / (RAM_MIRRORS + 1)).into(),
            RAM_MIRRORS.into(),
        )));
        let ram_ptr = Rc::clone(&ram);
        main_bus
            .borrow_mut()
            .attach(
                "RAM",
                ram_ptr,
                AddressRange {
                    start: RAM_START,
                    end: RAM_END,
                },
            )
            .unwrap();

        let ppu_ptr = Rc::clone(&ppu); // The 8 PPU registers are mirrored 1023 times
        main_bus
            .borrow_mut()
            .attach(
                "PPU registers",
                ppu_ptr,
                AddressRange {
                    start: PPU_REGISTERS_START,
                    end: PPU_REGISTERS_END,
                },
            )
            .unwrap();

        let fake_apu = Rc::new(RefCell::new(Ram::new(APU_AND_IO_REGISTERS_SIZE.into())));
        let fake_apu_ptr = Rc::clone(&fake_apu);
        main_bus
            .borrow_mut()
            .attach(
                "Fake APU",
                fake_apu_ptr,
                AddressRange {
                    start: APU_AND_IO_REGISTERS_START,
                    end: APU_AND_IO_REGISTERS_END,
                },
            )
            .unwrap();

        let controller_one = Rc::new(RefCell::new(Controller::new(receiver_one, false)));
        let controller_one_ptr = Rc::clone(&controller_one);
        main_bus
            .borrow_mut()
            .attach(
                "Controller 1",
                controller_one_ptr,
                AddressRange {
                    start: CONTROLLER_PORT_1,
                    end: CONTROLLER_PORT_1,
                },
            )
            .unwrap();

        let controller_two = Rc::new(RefCell::new(Controller::new(receiver_two, false)));
        let controller_two_ptr = Rc::clone(&controller_two);
        main_bus
            .borrow_mut()
            .attach(
                "Controller 2",
                controller_two_ptr,
                AddressRange {
                    start: CONTROLLER_PORT_2,
                    end: CONTROLLER_PORT_2,
                },
            )
            .unwrap();

        let cartidge_expansion_rom =
            Rc::new(RefCell::new(Ram::new(CARTIDGE_EXPANSION_ROM_SIZE.into())));
        let cartidge_expansion_rom_ptr = Rc::clone(&cartidge_expansion_rom);
        main_bus
            .borrow_mut()
            .attach(
                "Cartidge Expansion ROM",
                cartidge_expansion_rom_ptr,
                AddressRange {
                    start: CARTIDGE_EXPANSION_ROM_START,
                    end: CARTIDGE_EXPANSION_ROM_END,
                },
            )
            .unwrap();

        // Graphics Bus
        // ----------------------------------------------------------------------------------------

        let nametable = Rc::new(RefCell::new(Ciram::new(0x0400))); // 2 kB mirrored
        let name_table_memory_ptr = Rc::clone(&nametable);
        graphics_bus
            .borrow_mut()
            .attach(
                "Nametables",
                name_table_memory_ptr,
                AddressRange {
                    start: NAMETABLES_START,
                    end: NAMETABLES_END,
                },
            )
            .unwrap();

        let palette_memory = Rc::new(RefCell::new(PaletteMemory::new()));
        let palette_memory_ptr = Rc::clone(&palette_memory);
        graphics_bus
            .borrow_mut()
            .attach(
                "Palettes",
                palette_memory_ptr,
                AddressRange {
                    start: PALETTE_MEMORY_START,
                    end: PALETTE_MEMORY_END,
                },
            )
            .unwrap();

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

    /// Insert a [`Cartidge`] into the NES. If there was already a cartidge,
    /// replace it.
    ///
    /// Remember, to run the NES, you must insert a cartidge on it. What would
    /// you play otherwise?
    pub fn load_cartidge(&mut self, cartidge: Cartidge) {
        info!("Cartidge inserted: {}", cartidge);

        let ram = cartidge.mapper.program_ram_ref();
        let rom = cartidge.mapper.program_rom_ref();
        let chr = cartidge.mapper.character_memory_ref();

        self.main_bus
            .borrow_mut()
            .attach(
                "Cartidge RAM",
                ram,
                AddressRange {
                    start: CARTIDGE_RAM_START,
                    end: CARTIDGE_RAM_END,
                },
            )
            .unwrap();

        self.main_bus
            .borrow_mut()
            .attach(
                "Cartidge ROM",
                rom,
                AddressRange {
                    start: CARTIDGE_ROM_START,
                    end: CARTIDGE_ROM_END,
                },
            )
            .unwrap();

        // Pattern memory - also known as CHR ROM is a 8 kB memory where two
        // pattern tables are stored. It contains all graphical information the
        // PPU require to draw.
        //
        // It can be split into two 4 kB (0x1000) sections containing the
        // pattern tables 0 and 1
        self.graphics_bus
            .borrow_mut()
            .attach(
                "CHR ROM (pattern memories)",
                chr,
                AddressRange {
                    start: PATTERN_TABLES_START,
                    end: PATTERN_TABLES_END,
                },
            )
            .unwrap();

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
        if self.cartidge.is_none() {
            return Err("NES can't run without a cartidge!".to_string());
        }

        info!("NES indefinedly running game");

        // self.cpu_execute_forever();
        self.ui.start();

        loop {
            self.clock()?;
        }
    }

    /// Execute a NES simulated system clock.
    ///
    /// In the NES NTSC (2C02), this clock runs at ~21.47 MHz.
    ///
    /// NES components run at certain system clock divisions:
    /// - CPU clocks every 12 system clocks
    /// - PPU clocks every 4 system clocks
    ///
    /// See more information:
    /// https://www.nesdev.org/wiki/Cycle_reference_chart#Clock_rates
    pub fn clock(&mut self) -> Result<(), String> {
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
