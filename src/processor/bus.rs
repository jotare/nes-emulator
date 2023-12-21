use std::cell::RefCell;
use std::collections::HashMap;

use log::debug;

use crate::interfaces::AddressRange;
use crate::interfaces::Bus as BusTrait;
use crate::interfaces::DeviceId;
use crate::types::SharedMemory;

pub struct Bus {
    id: &'static str,
    devices: RefCell<HashMap<DeviceId, Device>>,
}

struct Device {
    device: SharedMemory,
    addr_range: AddressRange,
}

impl Bus {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            devices: RefCell::new(HashMap::new()),
        }
    }
}

impl BusTrait for Bus {
    fn attach(
        &mut self,
        id: DeviceId,
        memory: SharedMemory,
        addr_range: AddressRange,
    ) -> Result<(), String> {
        if self.devices.borrow().contains_key(id) {
            return Err(format!("Device '{id}' already exists on bus '{}'", self.id));
        }

        // TODO: return error on overlapping address ranges

        self.devices.borrow_mut().insert(
            id,
            Device {
                device: memory,
                addr_range,
            },
        );
        Ok(())
    }

    fn detach(&mut self, id: DeviceId) {
        self.devices.borrow_mut().remove(id);
    }

    fn read(&self, address: u16) -> u8 {
        for (device_id, Device { device, addr_range }) in self.devices.borrow().iter() {
            if address >= addr_range.start && address <= addr_range.end {
                let virtual_address = address - addr_range.start;
                let data = match device.borrow().try_read(virtual_address) {
                    Ok(data) => data,
                    Err(e) => {
                        let error = format!(
                            "Error while reading '{device_id}' on address ${address:0>4X}: {e}"
                        );
                        panic!("{}", error);
                    }
                };
                debug!(
                    "Bus ({0}) read from: {address:0>4X} <- {data:0>2X}",
                    self.id
                );
                return data;
            }
        }
        panic!(
            "Bus '{0}' doesn't have an attached device for address: '0x{address:x}'",
            self.id
        );
    }

    fn write(&self, address: u16, data: u8) {
        debug!("Bus ({0}) write to: {address:0>4X} <- {data:0>2X}", self.id);
        for (_device_id, Device { device, addr_range }) in self.devices.borrow_mut().iter_mut() {
            if address >= addr_range.start && address <= addr_range.end {
                let virtual_address = address - addr_range.start;
                device.borrow_mut().write(virtual_address, data);
                return;
            }
        }
        panic!(
            "Bus '{0}' doesn't have an attached device for address: '0x{address:x}'",
            self.id
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_bus_read_without_attached_devices() {
        let bus = Bus::new("test-bus");

        bus.read(0x1234);
    }

    #[test]
    #[should_panic]
    fn test_bus_write_without_attached_devices() {
        let bus = Bus::new("test-bus");

        bus.write(0x1234, 0xf0);
    }
}
