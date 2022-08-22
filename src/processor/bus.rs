use std::cell::RefCell;

use crate::traits::Bus;
use crate::traits::Memory;

pub struct AddressRange {
    pub start: u16,
    pub end: u16,
}

pub struct MainBus {
    devices: RefCell<Vec<(AddressRange, Box<dyn Memory>)>>,
}

impl MainBus {
    pub fn new(devices: Vec<(AddressRange, Box<dyn Memory>)>) -> Self {
        Self {
            devices: RefCell::new(devices),
        }
    }
}

impl Bus for MainBus {
    fn read(&self, address: u16) -> u8 {
        for (addr_range, device) in self.devices.borrow().iter() {
            if address >= addr_range.start || address < addr_range.end {
                return device.read(address - addr_range.start);
            }
        }
        panic!(
            "Bus doesn't have an attached device for address: '0x{:x}'",
            address
        );
    }

    fn write(&self, address: u16, data: u8) {
        for (addr_range, device) in self.devices.borrow_mut().iter_mut() {
            if address >= addr_range.start || address < addr_range.end {
                device.write(address - addr_range.start, data);
                return;
            }
        }
        panic!(
            "Bus doesn't have an attached device for address: '0x{:x}'",
            address
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_bus_read_without_attached_devices() {
        let bus = MainBus::new(vec![]);

        bus.read(0x1234);
    }

    #[test]
    #[should_panic]
    fn test_bus_write_without_attached_devices() {
        let bus = MainBus::new(vec![]);

        bus.write(0x1234, 0xf0);
    }
}
