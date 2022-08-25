/// Nintendo Entertainment System (NES) abstraction.
///
/// This module defines the higher level abstractions to run the NES
/// emulator. It defines the video game console `Nes` and `Cartidges`
/// representing games. To use it, create a Nes instance, create a
/// Cartidge from a ROM file, put the game on the machine and `run` to
/// start playing!
///
///
use std::rc::Rc;

use crate::interfaces::AddressRange;
use crate::interfaces::Bus as BusTrait;
use crate::processor::bus::Bus;
use crate::processor::cpu::Cpu;
use crate::processor::memory::MirroredRam;
use crate::cartidge::Catridge;

pub struct Nes {
    cartidge: Option<Catridge>,
    bus: Rc<Bus>
    cpu: Cpu,
}

impl Nes {
    pub fn new() -> Self {
        let ram = Box::new(MirroredRam::new(2048, 3)); // 8 kB mirrored RAM
        let mut bus = Bus::new();
        bus.attach(
            ram,
            AddressRange {
                start: 0x0000,
                end: 0x1FFF,
            },
        );


        let bus = Rc::new(bus);
        let bus_ptr = Rc::clone(&bus);
        let cpu = Cpu::new(bus_ptr);

        Self {
            bus,
            cpu,
            cartidge: None,
        }
    }

    pub fn load_cartidge(&mut self, cartidge: Catridge) {
        self.cartidge = Some(cartidge);
    }

    pub fn run(&self) {
        todo!()
    }
}

impl Default for Nes {
    fn default() -> Self {
        Self::new()
    }
}
