use std::cell::RefCell;
use std::rc::Rc;

use crate::graphics::ppu::Ppu;
use crate::interfaces::Memory;
use crate::processor::bus::Bus;
use crate::processor::memory::{Ciram, MirroredRom, Ram, Rom};
use crate::controller::Controller;

pub type SharedPpu = Rc<RefCell<Ppu>>;
pub type SharedBus = Rc<RefCell<Bus>>;
pub type SharedMemory = Rc<RefCell<dyn Memory>>;
pub type SharedRam = Rc<RefCell<Ram>>;
pub type SharedCiram = Rc<RefCell<Ciram>>;
pub type SharedRom = Rc<RefCell<Rom>>;
pub type SharedController = Rc<RefCell<Controller>>;
pub type SharedMirroredRom = Rc<RefCell<MirroredRom>>;
