use std::cell::RefCell;
use std::rc::Rc;

use crate::controller::Controller;
use crate::graphics::ppu::Ppu;
use crate::interfaces::Memory;
use crate::processor::bus::Bus;
use crate::processor::memory::{Ciram, MirroredMemory, Ram, Rom};

pub type SharedBus = Rc<RefCell<Bus>>;

pub type SharedMemory = Rc<RefCell<dyn Memory>>;
pub type SharedRam = Rc<RefCell<Ram>>;
pub type SharedCiram = Rc<RefCell<Ciram>>;
pub type SharedMirroredRom = Rc<RefCell<MirroredMemory<Rom>>>;

pub type SharedPpu = Rc<RefCell<Ppu>>;

pub type SharedController = Rc<RefCell<Controller>>;
