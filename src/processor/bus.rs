use std::cell::RefCell;
use std::collections::HashMap;

use log::debug;

use crate::errors::BusError;
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
    ) -> Result<(), BusError> {
        if self.devices.borrow().contains_key(id) {
            return Err(BusError::AlreadyAttached {
                bus_id: self.id,
                device_id: id,
            });
        }

        // Check for overlapping address ranges
        for (registered_id, registered_device) in self.devices.borrow().iter() {
            let Device {
                addr_range: registered_addr_range,
                ..
            } = registered_device;

            let min_start =
                std::cmp::min(registered_addr_range.start as u32, addr_range.start as u32);
            let max_end =
                std::cmp::max(registered_addr_range.end as u32, addr_range.end as u32) + 1;

            let new_range = (addr_range.end - addr_range.start + 1) as u32;
            let registered_range =
                (registered_addr_range.end - registered_addr_range.start + 1) as u32;

            if (new_range + registered_range) > (max_end - min_start) {
                panic!("Device '{id}' (with address {addr_range:?}) overlapps with '{registered_id}' (with address {registered_addr_range:?})");
            }
        }

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
        self.try_read(address)
            .map_err(|error| error.to_string())
            .unwrap()
    }

    fn write(&self, address: u16, data: u8) {
        self.try_write(address, data)
            .map_err(|error| error.to_string())
            .unwrap();
    }
}

impl Bus {
    fn try_read(&self, address: u16) -> Result<u8, BusError> {
        for (device_id, Device { device, addr_range }) in self.devices.borrow().iter() {
            if address >= addr_range.start && address <= addr_range.end {
                let virtual_address = address - addr_range.start;
                let data = device.borrow().try_read(virtual_address).map_err(|error| {
                    BusError::BusReadError {
                        bus_id: self.id,
                        device_id,
                        address,
                        details: error.to_string(),
                    }
                })?;
                debug!(
                    "Bus ({0}) read from: {address:0>4X} <- {data:0>2X}",
                    self.id
                );
                return Ok(data);
            }
        }
        Err(BusError::MissingBusDevice {
            bus_id: self.id.to_string(),
            address,
        })
    }

    fn try_write(&self, address: u16, data: u8) -> Result<(), BusError> {
        debug!("Bus ({0}) write to: {address:0>4X} <- {data:0>2X}", self.id);
        for (device_id, Device { device, addr_range }) in self.devices.borrow_mut().iter_mut() {
            if address >= addr_range.start && address <= addr_range.end {
                let virtual_address = address - addr_range.start;
                device
                    .borrow_mut()
                    .try_write(virtual_address, data)
                    .map_err(|error| BusError::BusWriteError {
                        bus_id: self.id,
                        device_id,
                        address,
                        details: error.to_string(),
                    })?;
                return Ok(());
            }
        }
        Err(BusError::MissingBusDevice {
            bus_id: self.id.to_string(),
            address,
        })
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
