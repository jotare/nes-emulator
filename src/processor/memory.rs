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

#[derive(Clone)]
pub struct MirroredRom {
    memory: Rom,
    mirrors: usize,
}

impl MirroredRom {
    pub fn new(size: usize, mirrors: usize) -> Self {
        Self {
            memory: Rom::new(size),
            mirrors,
        }
    }
    pub fn load(&mut self, address: u16, contents: &[u8]) {
        self.memory.load(address, contents);
    }
}

impl Memory for MirroredRom {
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

#[derive(Copy, Clone, Debug)]
pub enum Mirroring {
    /// Vertical arrangement (CIRAM A10 = PPU A11)
    Horizontal,

    /// Horizontal arrangement (CIRAM A10 = PPU A10)
    Vertical,
}

/// CIRAM memory is divided in 4 logical cells where the half is a mirror of the
/// other half.
#[derive(Clone)]
pub struct Ciram {
    memory: Ram,
    mirroring: Mirroring,
    cell_size: usize,
}

impl Ciram {
    pub fn new(cell_size: usize) -> Self {
        Self {
            memory: Ram::new(cell_size * 2),
            mirroring: Mirroring::Horizontal,
            cell_size,
        }
    }

    pub fn set_mirroring(&mut self, mirroring: Mirroring) {
        self.mirroring = mirroring;
    }

    fn compute_offset(&self, address: u16) -> u16 {
        // Nametables
        // (0,0)     (256,0)     (511,0)
        //        +-----------+-----------+
        //        |           |           |
        //        |           |           |
        //        |   $2000   |   $2400   |
        //        |           |           |
        //        |         0 | 1         |
        // (0,240)+-----------+-----------+(511,240)
        //        |         2 | 3         |
        //        |           |           |
        //        |   $2800   |   $2C00   |
        //        |           |           |
        //        |           |           |
        //        +-----------+-----------+
        //      (0,479)   (256,479)   (511,479)

        let cell_size = self.cell_size as u16;
        let cell = match address as usize {
            a if a < self.cell_size => 0,
            a if self.cell_size <= a && a < 2 * self.cell_size => 1,
            a if self.cell_size * 2 <= a && a < 3 * self.cell_size => 2,
            a if self.cell_size * 3 <= a && a < 4 * self.cell_size => 3,
            _ => panic!("Impossible CIRAM address {}", address),
        };

        let offset = match (cell, self.mirroring) {
            // Horizontal
            // +---+---+
            // | A | A |
            // +---+---+
            // | B | B |
            // +---+---+
            (0, Mirroring::Horizontal) => 0,
            (1, Mirroring::Horizontal) => cell_size,
            (2, Mirroring::Horizontal) => cell_size,
            (3, Mirroring::Horizontal) => 2 * cell_size,

            // Vertical
            // +---+---+
            // | A | B |
            // +---+---+
            // | A | B |
            // +---+---+
            (0, Mirroring::Vertical) => 0,
            (1, Mirroring::Vertical) => 0,
            (2, Mirroring::Vertical) => 2 * cell_size,
            (3, Mirroring::Vertical) => 2 * cell_size,

            _ => panic!(
                "Impossible CIRAM cell-mirroring combination: {} {:?}",
                cell, self.mirroring
            ),
        };

        offset
    }
}

impl Memory for Ciram {
    fn read(&self, address: u16) -> u8 {
        let offset = self.compute_offset(address);
        self.memory.read(address - offset)
    }

    fn write(&mut self, address: u16, data: u8) {
        let offset = self.compute_offset(address);
        println!("Writing to  {address:0>4X} - {offset:0>4X} -> {data:0>4X}");
        self.memory.write(address - offset, data);
    }

    fn size(&self) -> usize {
        self.cell_size * 4
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
