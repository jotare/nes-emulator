use std::cell::RefCell;
use std::rc::Rc;

use crate::graphics::ppu::Ppu;
use crate::interfaces::Memory;
use crate::processor::bus::Bus;
use crate::processor::memory::{MirroredRom, Ram, Rom};

pub type SharedPpu = Rc<RefCell<Ppu>>;
pub type SharedBus = Rc<RefCell<Bus>>;
pub type SharedMemory = Rc<RefCell<dyn Memory>>;
pub type SharedRam = Rc<RefCell<Ram>>;
pub type SharedRom = Rc<RefCell<Rom>>;
pub type SharedMirroredRom = Rc<RefCell<MirroredRom>>;
