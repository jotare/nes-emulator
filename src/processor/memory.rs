use crate::interfaces::Memory;

const RAM_SIZE: usize = 2 * 1024; // 2 kB RAM

#[derive(Debug, Clone)]
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
pub struct Rom {
    memory: Vec<u8>,
    /// How many times the ROM has been programmed
    count: usize,
}

impl Rom {
    pub fn new(size: usize) -> Self {
        Self {
            memory: vec![0; size],
            count: 0,
        }
    }

    /// Perform a memory load to the ROM. As it's intended to be
    /// read-only, this method can be used one time. Any other call
    /// will panic.
    pub fn load(&mut self, address: u16, contents: &[u8]) {
        if self.count > 0 {
            panic!("ROM memory can be written only once");
        }

        for (i, byte) in contents.iter().enumerate() {
            let i = i as u16;
            self.write(address + i, *byte);
        }
        self.count += 1;
    }
}

impl Memory for Rom {
    fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    /// Trying to write on any location will panic, as a ROM is a
    /// read-only memory.
    fn write(&mut self, _address: u16, _data: u8) {
        panic!("ROM is a read-only memory and can't be written!");
    }

    fn size(&self) -> usize {
        self.memory.len()
    }
}
