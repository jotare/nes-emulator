use crate::traits::Memory;

const RAM_SIZE: usize = 2 * 1024; // 2 kB RAM

pub struct Ram {
    memory: Vec<u8>,
}

impl Ram {
    pub fn new() -> Self {
        Self { memory: vec![0; RAM_SIZE] }
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
