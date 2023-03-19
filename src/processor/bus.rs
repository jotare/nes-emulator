use std::cell::RefCell;
use std::rc::Rc;

use log::debug;

use crate::interfaces::AddressRange;
use crate::interfaces::Bus as BusTrait;
use crate::interfaces::Memory;

type Device = (usize, Rc<RefCell<dyn Memory>>, AddressRange);

pub struct Bus {
    id: &'static str,
    devices: RefCell<Vec<Device>>,
    next_device_id: usize,
}

impl Bus {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            devices: RefCell::new(Vec::new()),
            next_device_id: 0,
        }
    }
}

impl BusTrait for Bus {
    fn attach(&mut self, device: Rc<RefCell<dyn Memory>>, addr_range: AddressRange) -> usize {
        let device_id = self.next_device_id;
        self.next_device_id += 1;
        self.devices
            .borrow_mut()
            .push((device_id, device, addr_range));
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
            if address >= addr_range.start && address <= addr_range.end {
                let virtual_address = address - addr_range.start;
                let data = device.borrow().read(virtual_address);
                debug!("Bus ({0}) read from: {address:0>4X} <- {data:0>2X}", self.id);
                return data;
            }
        }
        panic!("Bus doesn't have an attached device for address: '0x{address:x}'");
    }

    fn write(&self, address: u16, data: u8) {
        debug!("Bus ({0}) write to: {address:0>4X} <- {data:0>2X}", self.id);
        for (_, device, addr_range) in self.devices.borrow_mut().iter_mut() {
            if address >= addr_range.start && address <= addr_range.end {
                let virtual_address = address - addr_range.start;
                device.borrow_mut().write(virtual_address, data);
                return;
            }
        }
        panic!("Bus doesn't have an attached device for address: '0x{address:x}'");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_bus_read_without_attached_devices() {
        let bus = Bus::new();

        bus.read(0x1234);
    }

    #[test]
    #[should_panic]
    fn test_bus_write_without_attached_devices() {
        let bus = Bus::new();

        bus.write(0x1234, 0xf0);
    }
}
