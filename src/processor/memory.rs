use crate::interfaces::Memory;

const RAM_SIZE: usize = 2 * 1024; // 2 kB RAM

#[derive(Clone)]
pub struct Ram {
    memory: Vec<u8>,
}

impl Ram {
    pub fn new(size: usize) -> Self {
        Self {
            memory: vec![0; size],
        }
    }
}

impl Ram {
    /// Load `contents` array starting on `address`.
    pub fn load(&mut self, address: u16, contents: &[u8]) {
        for (i, byte) in contents.iter().enumerate() {
            let i = i as u16;
            self.write(address + i, *byte);
        }
    }
}

impl Memory for Ram {
    fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data;
    }

    fn size(&self) -> usize {
        self.memory.len()
    }
}

#[derive(Clone)]
pub struct MirroredRam {
    memory: Ram,
    mirrors: usize,
}

impl MirroredRam {
    pub fn new(size: usize, mirrors: usize) -> Self {
        Self {
            memory: Ram::new(size),
            mirrors,
        }
    }
}

impl Memory for MirroredRam {
    fn read(&self, address: u16) -> u8 {
        let address = ((address as usize) % self.memory.size()) as u16;
        self.memory.read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        let address = ((address as usize) % self.memory.size()) as u16;
        self.memory.write(address, data);
    }

    fn size(&self) -> usize {
        self.memory.size() * (self.mirrors + 1)
    }
}

/// ROM - Read-Only Memory
#[derive(Clone)]
pub struct Rom {
    memory: Vec<u8>,
    /// How many times the ROM has been programmed
    write_count: usize,
}

impl Rom {
    pub fn new(size: usize) -> Self {
        Self {
            memory: vec![0; size],
            write_count: 0,
        }
    }

    /// Perform a memory load to the ROM. As it's intended to be
    /// read-only, this method can be used one time. Any other call
    /// will panic.
    pub fn load(&mut self, address: u16, contents: &[u8]) {
        if self.write_count > 0 {
            panic!("ROM memory can be written only once");
        }

        for (i, byte) in contents.iter().enumerate() {
            let i = i as u16;
            self.memory[(address + i) as usize] = *byte;
        }
        self.write_count += 1;
    }
}

impl Memory for Rom {
    fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    /// Trying to write on any location will panic, as a ROM is a
    /// read-only memory.
    fn write(&mut self, address: u16, data: u8) {
        panic!(
            "ROM is a read-only memory and can't be written! Attempted to write 0x{data:0>2X} to 0x{address:0>4X}"
        );
    }

    fn size(&self) -> usize {
        self.memory.len()
    }
}

// Sprite memory
pub struct PatternMemory {
    // 0x0000 - 0x1FFF (ppu bus)
}

// 2D arrays which store the ids of which patterns to show in the
// backgraound
pub struct NameTable {
    // 0x2000 - 0x2FFF (ppu bus)
}

// Stores which colors should be displayed on the screen when you
// combine the sprites and the background
pub struct Palettes {
    // 0x3F00 - 0x3FFF (ppu bus)
}

// Program ROM
pub struct ProgramRom {
    // 0x4020 - 0xFFFF (main bus)
}
