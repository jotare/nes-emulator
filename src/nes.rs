/// Nintendo Entertainment System (NES) abstraction.
///
/// This module defines the higher level abstractions to run the NES
/// emulator. It defines the video game console `Nes` and `Cartidges`
/// representing games. To use it, create a Nes instance, create a
/// Cartidge from a ROM file, put the game on the machine and `run` to
/// start playing!
///
///
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct Nes {
    cartidge: Option<Cartidge>,
}

impl Nes {
    pub fn new() -> Self {
        Self { cartidge: None }
    }

    pub fn load_cartidge(&mut self, cartidge: Cartidge) {
        self.cartidge = Some(cartidge);
    }

    pub fn run(&self) {
        todo!()
    }
}

impl Default for Nes {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Cartidge {
    contents: Vec<u8>,
}

impl Cartidge {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let mut file = File::open(path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        Self { contents: buffer }
    }
}
