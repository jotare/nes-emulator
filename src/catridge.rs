use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::processor::memory::Ram;

pub struct Catridge {
    contents: Vec<u8>,
    // Program memory
    pgr_memory: Ram,
    // Character memory, stores patterns and graphics for the PPU
    chr_memory: Ram,
}

impl Catridge {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let mut file = File::open(path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        Self { contents: buffer }
    }
}

pub struct Mapper {}

impl Mapper {
    pub fn new() -> Self {
        Self {}
    }
}
