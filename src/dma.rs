//! The NES had a primitive DMA which allowed data transfer between CPU and PPU
//! OAM and CPU and APU.
//!
//! This module encapsulate the DMA logic in [`DmaController`]
//!

use crate::interfaces::Bus;
use crate::interfaces::Memory;
use crate::types::{SharedBus, SharedPpu};
use log::debug;

/// DMA controller is responsible to manage DMA. Once DMA starts,
/// [`DmaController`] is able to track the progress and indicate ending of DMA
/// process
pub struct DmaController {
    /// indicate whether DMA is active or not
    transfer: bool,

    /// indicates a dummy DMA cycle when it's synchronizing
    dummy: bool,

    /// high 8-bits of main bus address for OAM DMA transfer
    page: u8,

    /// low 8-bits of main bus address for OAM DMA transfer
    addr: u8,

    /// Byte of OAM data read from the CPU to write to the PPU
    data: u8,
}

#[derive(Default)]
pub enum DmaCycle {
    #[default]
    Read,
    Write,
}

impl DmaController {
    pub fn new() -> Self {
        Self {
            transfer: false,
            dummy: true,
            data: 0,
            page: 0,
            addr: 0,
        }
    }

    pub fn is_oam_dma_active(&self, clock: u64) -> bool {
        self.transfer
    }

    pub fn dma_cycle(&self, cpu_clock: u64) -> DmaCycle {
        if cpu_clock % 2 == 0 {
            DmaCycle::Read
        } else {
            DmaCycle::Write
        }
    }

    pub fn oam_dma_transfer(&mut self, cpu_clock: u64, main_bus: &SharedBus, ppu: &SharedPpu) {
        if self.dummy {
            if cpu_clock % 2 == 0 {
                self.dummy = false;
            }
        } else {
            match self.dma_cycle(cpu_clock) {
                DmaCycle::Read => {
                    self.oam_dma_read(main_bus);
                }
                DmaCycle::Write => {
                    self.oam_data_write(ppu);
                }
            }
        }
    }

    fn oam_dma_read(&mut self, main_bus: &SharedBus) {
        let oam_addr = ((self.page as u16) << 8) | self.addr as u16;
        self.data = main_bus.borrow().read(oam_addr);
    }

    fn oam_data_write(&mut self, ppu: &SharedPpu) {
        ppu.borrow_mut().oam_dma_write(self.addr, self.data);
        self.addr += 1;

        // once we wrap around, we've done 256 read-write cycles and filled the
        // OAM with data, we can now stop DMA
        let finish = self.addr == 0x00;
        self.transfer = !finish;

        if finish {
            debug!("OAM DMA finished");
        }
    }
}

impl Memory for DmaController {
    fn read(&self, address: u16) -> u8 {
        panic!("OAM DMA is a write only memory position!");
    }

    fn write(&mut self, address: u16, data: u8) {
        debug!("OAM DMA starts for page: ${data:0>2X}");
        self.transfer = true;
        self.page = data;
        self.addr = 0;
    }

    fn size(&self) -> usize {
        1
    }
}
