use std::cell::RefCell;
use std::rc::Rc;

use crate::types::SharedMemory;


#[derive(Debug)]
pub struct AddressRange {
    pub start: u16,
    pub end: u16,
}

pub type DeviceId = &'static str;

pub trait Bus {
    /// Attach a new device to the bus to further read/write from
    /// it. Return an UUID to uniquely refer to `device`.
    fn attach(&mut self, id: DeviceId, device: SharedMemory, addr_range: AddressRange) -> Result<(), String>;

    /// Detach a device from the bus
    fn detach(&mut self, id: DeviceId);

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

    fn try_read(&self, address: u16) -> Result<u8, String> {
        let size = self.size() as u16;
        if address > size {
            return Err(format!("Read address out of bounds, index is ${address:0>4X} but memory size is ${size:0>4X}"));
        }
        Ok(self.read(address))
    }

    /// Write a byte of `data` to the specified `address`
    fn write(&mut self, address: u16, data: u8);

    fn try_write(&mut self, address: u16, data: u8) -> Result<(), String> {
        let size = self.size() as u16;
        if address > size {
            return Err(format!("Write address out of bounds, index is ${address:0>4X} but memory size is ${size:0>4X}"));
        }
        self.write(address, data);
        Ok(())
    }

    /// Memory size in bytes
    fn size(&self) -> usize;
}
