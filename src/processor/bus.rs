use std::cell::RefCell;

use crate::interfaces::Bus;
use crate::interfaces::AddressRange;
use crate::interfaces::Memory;

type Device = (usize, Box<dyn Memory>, AddressRange);

pub struct MainBus {
    devices: RefCell<Vec<Device>>,
    next_device_id: usize,
}

impl MainBus {
    pub fn new() -> Self {
        Self {
            devices: RefCell::new(Vec::new()),
            next_device_id: 0,
        }
    }
}

impl Bus for MainBus {
    fn attach(&mut self, device: Box<dyn Memory>, addr_range: AddressRange) -> usize {
        let device_id = self.next_device_id;
        self.next_device_id += 1;
        self.devices.borrow_mut().push((device_id, device, addr_range));
        device_id
    }

    fn detach(&mut self, id: usize) {
        let mut delete = None;

        for (i, (_id, _, _)) in self.devices.borrow().iter().enumerate() {
            if id == *_id {
                delete = Some(i);
            }
        }

        if let Some(i) = delete {
            self.devices.borrow_mut().remove(i);
        }
    }

    
    fn read(&self, address: u16) -> u8 {
        for (_, device, addr_range) in self.devices.borrow().iter() {
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
        for (_, device, addr_range) in self.devices.borrow_mut().iter_mut() {
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
        let bus = MainBus::new();

        bus.read(0x1234);
    }

    #[test]
    #[should_panic]
    fn test_bus_write_without_attached_devices() {
        let bus = MainBus::new();

        bus.write(0x1234, 0xf0);
    }
}
