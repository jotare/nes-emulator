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
use crate::interfaces::AddressRange;
use crate::interfaces::Bus as BusTrait;
use crate::interfaces::Processor;
use crate::processor::bus::Bus;
use crate::processor::cpu::Cpu;
use crate::processor::memory::MirroredRam;
use crate::processor::memory::Ram;

type NesBus = Rc<RefCell<Bus>>;
type NesCpu = Rc<RefCell<Cpu>>;

pub struct Nes {
    cartidge: Option<Cartidge>,
    bus: NesBus,
    cpu: Cpu,
}

impl Nes {
    pub fn new() -> Self {
        let bus = Rc::new(RefCell::new(Bus::new()));
        let bus_ptr = Rc::clone(&bus);
        let cpu = Cpu::new(bus_ptr);

        let ram = Box::new(MirroredRam::new(2048, 3)); // 8 kB mirrored RAM
        bus.borrow_mut().attach(
            ram,
            AddressRange {
                start: 0x0000,
                end: 0x1FFF,
            },
        );

        let expansion_rom = Box::new(Ram::new(8160));
        bus.borrow_mut().attach(
            expansion_rom,
            AddressRange {
                start: 0x4020,
                end: 0x5FFF,
            },
        );

        let fake_ppu = Box::new(MirroredRam::new(8, 1023)); // 8 B mirrored RAM
        bus.borrow_mut().attach(
            fake_ppu,
            AddressRange {
                start: 0x2000,
                end: 0x3FFF,
            },
        );

        let fake_apu = Box::new(Ram::new(0x18)); // 0x18 B RAM - NES APU and I/O registers
        bus.borrow_mut().attach(
            fake_apu,
            AddressRange {
                start: 0x4000,
                end: 0x4017,
            },
        );


        Self {
            bus,
            cpu,
            cartidge: None,
        }
    }

    pub fn load_cartidge(&mut self, cartidge: Cartidge) {
        info!("Cartidge inserted: {}", cartidge);

        let ram = cartidge.program_ram.clone();
        let rom = cartidge.program_rom.clone();

        // XXX: use references to avoid cloning memory
        self.bus.borrow_mut().attach(
            Box::new(ram),
            AddressRange {
                start: 0x6000,
                end: 0x7FFF,
            },
        );
        // XXX: use references to avoid cloning memory
        self.bus.borrow_mut().attach(
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
        loop {
            self.cpu.execute();
        }
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
