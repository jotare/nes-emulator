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
use crate::controller::ControllerButtons;
use crate::controller::Controllers;
use crate::dma::DmaController;
use crate::errors::NesError;
use crate::events::Event;
use crate::events::KeyboardChannel;
use crate::events::SharedEventBus;
use crate::graphics::ppu::Ppu;
use crate::hardware::*;
use crate::interfaces::AddressRange;
use crate::interfaces::Bus as BusTrait;
use crate::metrics::Collector;
use crate::processor::bus::Bus;
use crate::processor::bus::GraphicsBus;
use crate::processor::cpu::{Cpu, Interrupt};
use crate::processor::memory::Ram;
use crate::settings::NesSettings;
use crate::settings::UiKind;
use crate::types::SharedGraphicsBus;
use crate::types::{SharedBus, SharedPpu};
use crate::ui::{GtkUi, Ui};

pub struct Nes {
    // XXX: change to u128 if overflow occur
    system_clock: u64,

    cartidge: Option<Cartidge>,

    pub cpu: Cpu,
    pub main_bus: SharedBus,

    pub ppu: SharedPpu,
    pub graphics_bus: SharedGraphicsBus,

    dma_controller: Rc<RefCell<DmaController>>,

    pub ui: Option<GtkUi>,

    controllers: Rc<RefCell<Controllers>>,

    event_bus: SharedEventBus,
    keyboard_channel: KeyboardChannel,

    settings: NesSettings,
    metrics: Collector,
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
        let graphics_bus = Rc::new(RefCell::new(GraphicsBus::new()));

        let main_bus_ptr = Rc::clone(&main_bus);
        let cpu = Cpu::new(main_bus_ptr);

        let graphics_bus_ptr = Rc::clone(&graphics_bus);
        let ppu = Rc::new(RefCell::new(Ppu::new(graphics_bus_ptr, event_bus.clone())));

        // Main Bus
        // ----------------------------------------------------------------------------------------

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

        // Fake APU registers to avoid the games panicking for unattached
        // address
        main_bus
            .borrow_mut()
            .attach(
                "Fake APU (1)",
                Rc::new(RefCell::new(Ram::new(0x4014 - 0x4000))),
                AddressRange {
                    start: 0x4000,
                    end: 0x4013,
                },
            )
            .unwrap();
        main_bus
            .borrow_mut()
            .attach(
                "Fake APU (2)",
                Rc::new(RefCell::new(Ram::new(0x4015 - 0x4014))),
                AddressRange {
                    start: 0x4015,
                    end: 0x4015,
                },
            )
            .unwrap();

        let controllers = Rc::new(RefCell::new(Controllers::new(keyboard_channel.listener())));
        let controllers_ptr = Rc::clone(&controllers);
        main_bus
            .borrow_mut()
            .attach(
                "Controllers",
                controllers_ptr,
                AddressRange {
                    start: CONTROLLER_PORT_1,
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

        // ----------------------------------------------------------------------------------------

        Self {
            system_clock: 0,
            cartidge: None,
            cpu,
            main_bus,
            ppu,
            graphics_bus,
            dma_controller,
            ui: None,
            controllers,
            event_bus,
            keyboard_channel,
            settings,
            metrics: Collector::new(),
        }
    }

    /// Insert a [`Cartidge`] into the NES. If there was already a cartidge,
    /// replace it.
    ///
    /// Remember, to run the NES, you must insert a cartidge on it. What would
    /// you play otherwise?
    pub fn load_cartidge(&mut self, cartidge: Cartidge) {
        info!("Cartidge inserted: {}", cartidge);

        cartidge.mapper.connect(&self.main_bus, &self.graphics_bus);

        self.graphics_bus
            .borrow_mut()
            .nametables
            .inner_mut()
            .set_mirroring(cartidge.mirroring());

        self.cartidge = Some(cartidge);
        self.cpu.reset();
    }

    /// Connect controller one to the NES and define its configuration
    pub fn connect_controller_one(&mut self, buttons: ControllerButtons) {
        self.controllers
            .borrow_mut()
            .connect_controller_one(buttons);
    }

    /// Diconnect controller one from the NES. After this action, the controls
    /// defined for this controller won't do anything anymore
    pub fn disconnect_controller_one(&mut self) {
        self.controllers.borrow_mut().disconnect_controller_one();
    }

    /// Connect controller two to the NES and define its configuration
    pub fn connect_controller_two(&mut self, buttons: ControllerButtons) {
        self.controllers
            .borrow_mut()
            .connect_controller_two(buttons);
    }

    /// Diconnect controller two from the NES. After this action, the controls
    /// defined for this controller won't do anything anymore
    pub fn disconnect_controller_two(&mut self) {
        self.controllers.borrow_mut().disconnect_controller_two();
    }

    /// Power-up the NES. This should be done before running the NES. It presets
    /// the emulator with some specific values
    pub fn power_up(&mut self) {
        self.cpu.power_up()
    }

    /// Blocking NES run
    pub fn run(&mut self) -> Result<(), NesError> {
        self.power_up();

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
            {
                let mut event_bus = self.event_bus.access();
                if event_bus.emitted(Event::SwitchOff) {
                    println!("Switching off NES");
                    break;
                } else if event_bus.emitted(Event::Reset) {
                    println!("Resetting the NES at clock {}", self.system_clock);
                    self.cpu.reset();
                    // TODO: PPU reset
                    event_bus.mark_as_processed(Event::Reset);
                }
            }

            if self.system_clock % (2_u64.pow(25)) == 0 {
                self.metrics.observe_system_clocks(2_u64.pow(25));
                let metrics = self.metrics.collect();
                println!("FPS: {}", metrics.frames_per_second);
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
                self.metrics.observe_frame_ready();
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
            self.dma_controller.borrow_mut().clock();
            let ongoing_dma = self.dma_controller.borrow().is_oam_dma_active();
            if ongoing_dma {
                self.dma_controller
                    .borrow_mut()
                    .oam_dma_transfer(&self.main_bus, &self.ppu);
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
