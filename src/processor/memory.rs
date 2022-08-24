use crate::interfaces::Memory;

const RAM_SIZE: usize = 2 * 1024; // 2 kB RAM

pub struct Ram {
    memory: Vec<u8>,
}

impl Ram {
    pub fn new(size: usize) -> Self {
        Self { memory: vec![0; size]}
    }

    pub fn size(&self) -> usize {
        self.memory.len()
    }
}

impl Memory for Ram {
    fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data;
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
}
