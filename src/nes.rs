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

use log::info;

use crate::cartidge::Cartidge;
use crate::controller::Controller;
use crate::controller::ControllerButtons;
use crate::dma::DmaController;
use crate::errors::NesError;
use crate::events::Event;
use crate::events::KeyboardChannel;
use crate::events::SharedEventBus;
use crate::graphics::palette_memory::PaletteMemory;
use crate::graphics::ppu::Ppu;
use crate::hardware::*;
use crate::interfaces::AddressRange;
use crate::interfaces::Bus as BusTrait;
use crate::processor::bus::Bus;
use crate::processor::cpu::{Cpu, Interrupt};
use crate::processor::memory::MirroredMemory;
use crate::processor::memory::{Ciram, Ram};
use crate::settings::NesSettings;
use crate::settings::UiKind;
use crate::types::{SharedBus, SharedCiram, SharedController, SharedMemory, SharedPpu};
use crate::ui::{GtkUi, Ui};

pub struct Nes {
    // XXX: change to u128 if overflow occur
    system_clock: u64,

    cartidge: Option<Cartidge>,

    pub cpu: Cpu,
    pub main_bus: SharedBus,

    pub ppu: SharedPpu,
    pub graphics_bus: SharedBus,

    ram: SharedMemory,
    nametable: SharedCiram,
    palettes: SharedMemory,

    dma_controller: Rc<RefCell<DmaController>>,

    pub ui: Option<GtkUi>,

    controller_one: SharedController,
    controller_two: SharedController,

    event_bus: SharedEventBus,
    keyboard_channel: KeyboardChannel,

    settings: NesSettings,
}

impl Default for Nes {
    fn default() -> Self {
        Nes::new(NesSettings::default())
    }
}

impl Nes {
    pub fn new(settings: NesSettings) -> Self {
        let event_bus = SharedEventBus::new();
        let keyboard_channel = KeyboardChannel::default();

        let main_bus = Rc::new(RefCell::new(Bus::new("CPU")));
        let graphics_bus = Rc::new(RefCell::new(Bus::new("PPU")));

        let main_bus_ptr = Rc::clone(&main_bus);
        let cpu = Cpu::new(main_bus_ptr);

        let graphics_bus_ptr = Rc::clone(&graphics_bus);
        let ppu = Rc::new(RefCell::new(Ppu::new(graphics_bus_ptr, event_bus.clone())));

        // Main Bus
        // ----------------------------------------------------------------------------------------

        let ram = Rc::new(RefCell::new(MirroredMemory::new(
            Ram::new((RAM_SIZE / (RAM_MIRRORS + 1)).into()),
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

        let keyboard_listener_one = keyboard_channel.listener();
        let controller_one = Rc::new(RefCell::new(Controller::new(keyboard_listener_one)));
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

        let keyboard_listener_two = keyboard_channel.listener();
        let controller_two = Rc::new(RefCell::new(Controller::new(keyboard_listener_two)));
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

        let dma_controller = Rc::new(RefCell::new(DmaController::new()));
        main_bus
            .borrow_mut()
            .attach(
                "DMA controller",
                dma_controller.clone(),
                AddressRange {
                    start: OAM_DMA,
                    end: OAM_DMA,
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

        let palette_memory = Rc::new(RefCell::new(MirroredMemory::new(
            PaletteMemory::new(),
            PALETTE_MIRRORS.into(),
        )));
        let palette_memory_ptr = Rc::clone(&palette_memory);
        graphics_bus
            .borrow_mut()
            .attach(
                "Palettes",
                palette_memory_ptr,
                AddressRange {
                    start: PALETTE_MEMORY_START,
                    end: PALETTE_MEMORY_MIRRORS_END,
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
            dma_controller,
            ui: None,
            controller_one,
            controller_two,
            event_bus,
            keyboard_channel,
            settings,
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

        self.nametable
            .borrow_mut()
            .set_mirroring(cartidge.mirroring());

        self.cartidge = Some(cartidge);
        self.cpu.reset();
    }

    /// Connect controller one to the NES and define its configuration
    pub fn connect_controller_one(&mut self, buttons: ControllerButtons) {
        self.controller_one.borrow_mut().connect(buttons);
    }

    /// Diconnect controller one from the NES. After this action, the controls
    /// defined for this controller won't do anything anymore
    pub fn disconnect_controller_one(&mut self) {
        self.controller_one.borrow_mut().disconnect();
    }

    /// Blocking NES run
    pub fn run(&mut self) -> Result<(), NesError> {
        if self.cartidge.is_none() {
            return Err(NesError::NoCartidgeInserted);
        }

        info!("NES indefinedly running game");

        // self.cpu_execute_forever();

        if let Some(ui) = self.ui.as_mut() {
            ui.start().map_err(|error| NesError::UiError {
                details: "Failed to start UI".to_string(),
                source: error,
            })?;
        }

        loop {
            if self.event_bus.access().emitted(Event::SwitchOff) {
                break;
            }

            self.clock()
                .map_err(|error| NesError::NesInternalError(error))?;
        }

        if let Some(ui) = self.ui.as_mut() {
            ui.stop().map_err(|error| NesError::UiError {
                details: "Failed to stop UI after execution stopped".to_string(),
                source: error,
            })?;
        }

        Ok(())
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

            if self.event_bus.access().emitted(Event::NMI) {
                self.cpu.interrupt(Interrupt::NonMaskableInterrupt);
                self.event_bus.access().mark_as_processed(Event::NMI);
            }

            if self.event_bus.access().emitted(Event::FrameReady) {
                let frame = ppu.take_frame();
                self.event_bus.access().mark_as_processed(Event::FrameReady);

                if let Some(ui) = self.ui.as_mut() {
                    ui.render(frame);
                }
                // std::thread::sleep(std::time::Duration::from_millis(33)); // ~30 FPS
                // std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
                // std::thread::sleep(std::time::Duration::from_millis(8)); // ~120 FPS
            }
        }

        // CPU clock runs every 12 system clocks
        if self.system_clock % 12 == 0 {
            let cpu_clock = self.system_clock / 12;
            let ongoing_dma = self.dma_controller.borrow().is_oam_dma_active(cpu_clock);
            if ongoing_dma {
                self.dma_controller.borrow_mut().oam_dma_transfer(
                    cpu_clock,
                    &self.main_bus,
                    &self.ppu,
                );
            } else {
                self.cpu.clock()?;
            }
        }

        Ok(())
    }

    /// Creates a new TV (UI) to render NES picture data and play audio. It must
    /// be called before running if one want to view and listen to the games
    pub fn setup_tv(&mut self) {
        let ui = match self.settings.ui_kind {
            UiKind::None => None,

            UiKind::Gtk => {
                let gtk_ui = GtkUi::builder()
                    .screen_size(SCREEN_WIDTH, SCREEN_HEIGHT)
                    .pixel_scale_factor(self.settings.pixel_scale_factor)
                    .with_keyboard_publisher(self.keyboard_channel.publisher())
                    .with_event_bus(self.event_bus.clone())
                    .build();
                Some(gtk_ui)
            }
        };

        if let Some(ui) = ui {
            self.ui.replace(ui);
        }
    }
}
