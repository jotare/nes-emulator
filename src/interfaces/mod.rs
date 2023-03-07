use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct AddressRange {
    pub start: u16,
    pub end: u16,
}

pub trait Bus {
    /// Attach a new device to the bus to further read/write from
    /// it. Return an UUID to uniquely refer to `device`.
    fn attach(&mut self, device: Rc<RefCell<dyn Memory>>, addr_range: AddressRange) -> usize;

    /// Detach a device from the bus
    fn detach(&mut self, id: usize);

    /// Read a byte from the device attached to the specified
    /// `address`.
    ///
    /// Panics if an address doesn't correspond to any attached
    /// device.
    fn read(&self, address: u16) -> u8;

    /// Writes a byte to the device attached to the specified
    /// `address`.
    ///
    /// Panics if an address doesn't correspond to any attached
    /// device.
    fn write(&self, address: u16, data: u8);
}

pub trait Memory {
    /// Read a byte from the specified `address`
    fn read(&self, address: u16) -> u8;

    /// Write a byte of `data` to the specified `address`
    fn write(&mut self, address: u16, data: u8);

    /// Memory size in bytes
    fn size(&self) -> usize;
}
