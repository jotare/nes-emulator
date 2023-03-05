use std::cell::RefCell;
use std::rc::Rc;

use crate::processor::bus::Bus;
use crate::graphics::ppu::Ppu;
use crate::interfaces::Memory;
use crate::processor::memory::{Ram, Rom, MirroredRom};


pub type SharedPpu = Rc<RefCell<Ppu>>;
pub type SharedBus = Rc<RefCell<Bus>>;
pub type SharedMemory = Rc<RefCell<dyn Memory>>;
pub type SharedRam = Rc<RefCell<Ram>>;
pub type SharedRom = Rc<RefCell<Rom>>;
pub type SharedMirroredRom = Rc<RefCell<MirroredRom>>;
