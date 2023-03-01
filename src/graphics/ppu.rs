/// PPU module
///
/// This module emulates the NES Picture Processing Unit (PPU)
///
/// NES PPU registers ($2000-$2007) are mirrored from $2008 to $3FFF. That's
/// because it's address is not completely decoded, that is, the chip ignores
/// one or more address lines. This allows a cheaper hardware (less address
/// lines) and a faster decoding at expense of unused address space.
///
///
///
use std::cell::RefCell;
use std::rc::Rc;

use crate::interfaces::Bus;

pub struct Ppu {}

impl Ppu {
    pub fn new(bus: Rc<RefCell<dyn Bus>>) -> Self {
        Self {}
    }
}
