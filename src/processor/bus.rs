use std::cell::RefCell;
use std::collections::HashMap;

use log::debug;

use crate::errors::BusError;
use crate::graphics::palette_memory::PaletteMemory;
use crate::hardware::CARTIDGE_WEIRD_UNUSED_REGION_END;
use crate::hardware::NAMETABLES_START;
use crate::hardware::PALETTE_MEMORY_MIRRORS_END;
use crate::hardware::PALETTE_MEMORY_START;
use crate::hardware::PALETTE_MIRRORS;
use crate::hardware::PATTERN_TABLES_END;
use crate::hardware::PATTERN_TABLES_START;
use crate::hardware::RAM_END;
use crate::hardware::RAM_MIRRORS;
use crate::hardware::RAM_SIZE;
use crate::hardware::RAM_START;
use crate::interfaces::AddressRange;
use crate::interfaces::Bus as BusTrait;
use crate::interfaces::DeviceId;
use crate::interfaces::Memory;
use crate::processor::memory::Ciram;
use crate::types::SharedMemory;

use super::memory::MirroredMemory;
use super::memory::Ram;

pub struct Bus {
    id: &'static str,
    devices: RefCell<HashMap<DeviceId, Device>>,

    ram: MirroredMemory<Ram>,
}

struct Device {
    device: SharedMemory,
    addr_range: AddressRange,
}

impl Bus {
    pub fn new(id: &'static str) -> Self {
        let ram = MirroredMemory::new(
            Ram::new((RAM_SIZE / (RAM_MIRRORS + 1)).into()),
            RAM_MIRRORS.into(),
        );

        Self {
            id,
            devices: RefCell::new(HashMap::new()),

            ram,
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

    fn write(&mut self, address: u16, data: u8) {
        self.try_write(address, data)
            .map_err(|error| error.to_string())
            .unwrap();
    }
}

impl Bus {
    fn try_read(&self, address: u16) -> Result<u8, BusError> {
        let Some((device_id, virtual_address, device)) = (match address {
            RAM_START..=RAM_END => {
                const DEVICE_ID: &str = "RAM";
                let virtual_address = address - RAM_START;
                let device: &dyn Memory = &self.ram;
                Some((DEVICE_ID, virtual_address, device))
            }
            _ => None,
        }) else {
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
            return Err(BusError::MissingBusDevice {
                bus_id: self.id.to_string(),
                address,
            });
        };

        let data = device
            .try_read(virtual_address)
            .map_err(|error| BusError::BusReadError {
                bus_id: "CPU",
                device_id,
                address,
                details: error.to_string(),
            })?;
        debug!("Bus (CPU) read from: {address:0>4X} <- {data:0>2X}");
        return Ok(data);
    }

    fn try_write(&mut self, address: u16, data: u8) -> Result<(), BusError> {
        debug!("Bus ({0}) write to: {address:0>4X} <- {data:0>2X}", self.id);

        let Some((device_id, virtual_address, device)) = (match address {
            RAM_START..=RAM_END => {
                const DEVICE_ID: &str = "RAM";
                let virtual_address = address - RAM_START;
                let device: &mut dyn Memory = &mut self.ram;
                Some((DEVICE_ID, virtual_address, device))
            }
            _ => None,
        }) else {
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
            return Err(BusError::MissingBusDevice {
                bus_id: self.id.to_string(),
                address,
            });
        };

        device
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

pub type MainBus = Bus;

/// Graphics Bus
///
/// See https://www.nesdev.org/wiki/PPU_memory_map for further reference
pub struct GraphicsBus {
    pattern_tables: Option<SharedMemory>,
    pub nametables: MirroredMemory<Ciram>,
    palettes: MirroredMemory<PaletteMemory>,
}

impl GraphicsBus {
    pub fn new() -> Self {
        let nametables = MirroredMemory::new(Ciram::new(0x0400), 2); // 2 kB mirrored

        let palette_memory = MirroredMemory::new(PaletteMemory::new(), PALETTE_MIRRORS.into());

        Self {
            nametables,
            palettes: palette_memory,
            pattern_tables: None,
        }
    }
}

impl BusTrait for GraphicsBus {
    fn attach(
        &mut self,
        id: DeviceId,
        device: SharedMemory,
        addr_range: AddressRange,
    ) -> Result<(), BusError> {
        unimplemented!("Not expecting to use this method anymore");
    }

    fn detach(&mut self, id: DeviceId) {
        unimplemented!("Not expecting to use this method anymore");
    }

    fn read(&self, address: u16) -> u8 {
        self.try_read(address)
            .map_err(|error| error.to_string())
            .unwrap()
    }

    fn write(&mut self, address: u16, data: u8) {
        self.try_write(address, data)
            .map_err(|error| error.to_string())
            .unwrap();
    }
}

impl GraphicsBus {
    pub fn connect_cartridge(&mut self, device: SharedMemory, addr_range: AddressRange) {
        assert!(
            addr_range.start == PATTERN_TABLES_START && addr_range.end == PATTERN_TABLES_END,
            "unexpected address range for cartridge CHR memory"
        );
        self.pattern_tables.replace(device);
    }

    fn try_read(&self, address: u16) -> Result<u8, BusError> {
        // PPU address are 14-bit long
        let address = address & 0x3FFF;

        let (device_id, virtual_address, device) = match address {
            PATTERN_TABLES_START..=PATTERN_TABLES_END => {
                const DEVICE_ID: &str = "Pattern tables (cartridge CHR memory)";
                let virtual_address = address - PATTERN_TABLES_START;
                let device: &dyn Memory =
                    self.pattern_tables
                        .as_ref()
                        .ok_or_else(|| BusError::MissingBusDevice {
                            bus_id: "PPU".to_string(),
                            address,
                        })?;
                (DEVICE_ID, virtual_address, device)
            }

            NAMETABLES_START..=CARTIDGE_WEIRD_UNUSED_REGION_END => {
                const DEVICE_ID: &str = "Nametables";
                let virtual_address = address - NAMETABLES_START;
                let device: &dyn Memory = &self.nametables;
                (DEVICE_ID, virtual_address, device)
            }

            PALETTE_MEMORY_START..=PALETTE_MEMORY_MIRRORS_END => {
                const DEVICE_ID: &str = "Palettes";
                let virtual_address = address - PALETTE_MEMORY_START;
                let device: &dyn Memory = &self.palettes;
                (DEVICE_ID, virtual_address, device)
            }

            _ => {
                return Err(BusError::MissingBusDevice {
                    bus_id: "PPU".to_string(),
                    address,
                });
            }
        };

        let data = device
            .try_read(virtual_address)
            .map_err(|error| BusError::BusReadError {
                bus_id: "PPU",
                device_id,
                address,
                details: error.to_string(),
            })?;

        debug!("Bus (PPU) read from: {address:0>4X} <- {data:0>2X}");
        Ok(data)
    }

    fn try_write(&mut self, address: u16, data: u8) -> Result<(), BusError> {
        debug!("Bus (PPU) write to: {address:0>4X} <- {data:0>2X}");

        // PPU address are 14-bit long
        let address = address & 0x3FFF;

        let (device_id, virtual_address, device) = match address {
            PATTERN_TABLES_START..=PATTERN_TABLES_END => {
                const DEVICE_ID: &str = "Pattern tables (cartridge CHR memory)";
                let virtual_address = address - PATTERN_TABLES_START;
                let device: &mut dyn Memory =
                    self.pattern_tables
                        .as_mut()
                        .ok_or_else(|| BusError::MissingBusDevice {
                            bus_id: "PPU".to_string(),
                            address,
                        })?;
                (DEVICE_ID, virtual_address, device)
            }

            NAMETABLES_START..=CARTIDGE_WEIRD_UNUSED_REGION_END => {
                const DEVICE_ID: &str = "Nametables";
                let virtual_address = address - NAMETABLES_START;
                let device: &mut dyn Memory = &mut self.nametables;
                (DEVICE_ID, virtual_address, device)
            }

            PALETTE_MEMORY_START..=PALETTE_MEMORY_MIRRORS_END => {
                const DEVICE_ID: &str = "Palettes";
                let virtual_address = address - PALETTE_MEMORY_START;
                let device: &mut dyn Memory = &mut self.palettes;
                (DEVICE_ID, virtual_address, device)
            }

            _ => {
                return Err(BusError::MissingBusDevice {
                    bus_id: "PPU".to_string(),
                    address,
                })
            }
        };

        device
            .try_write(virtual_address, data)
            .map_err(|error| BusError::BusWriteError {
                bus_id: "PPU",
                device_id,
                address,
                details: error.to_string(),
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
        let mut bus = Bus::new("test-bus");

        bus.write(0x1234, 0xf0);
    }
}
