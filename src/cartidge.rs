use std::fs::File;
use std::io::Read;
use std::path::Path;

use log::debug;

use crate::processor::memory::Ram;

pub struct Cartidge {
    name: String,

    // Program memory (RAM)
    pub program_ram: Ram,

    // Program memory (ROM)
    pub program_rom: Ram,

    // Character memory, stores patterns and graphics for the PPU
    character_memory: Ram,
}

impl Cartidge {
    /// Create a new cartidge loading the contents from a iNES file.
    ///
    /// Read more about iNES ROM file format in:
    /// https://www.nesdev.org/wiki/INES
    ///
    /// NES2.0 file format is not implemented.
    ///
    /// Header flags 8 to 10 are ignored.
    ///
    /// *Panic*
    ///
    /// iNES file format is expected and can panic if a different file format is
    /// used.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let game_name = path
            .as_ref()
            .file_name()
            .expect("Expected a .nes file, to a directory")
            .to_owned()
            .into_string()
            .unwrap();

        let mut file = File::open(path).unwrap();

        let mut header = vec![0; 16]; // 16 byte header
        file.read_exact(&mut header).unwrap();

        // Program and character ROM sizes
        let pgr_memory_size = (header[4] as usize) * 16 * 1024;
        let chr_memory_size = (header[5] as usize) * 8 * 1024;

        let trainer = (header[6] & 0x04) != 0;
        let mapper = (header[7] & 0xF0) | ((header[6] & 0xF0) >> 4);
        debug!("Cartidge mapper: {mapper}");

        // match mapper {
        //     0 => todo!(),
        //     _ => panic!("Mapper {} not implemented", mapper),
        // }

        // Trainer content is ignored for now
        if trainer {
            let mut buf = vec![0; 512]; // 512-byte trainer at 0x7000 - 0x71FF
            file.read_exact(&mut buf).unwrap();
            Some(buf)
        } else {
            None
        };

        let pgr_ram = Ram::new(0x7FFF - 0x6000 + 1);

        let mut pgr_memory = Ram::new(pgr_memory_size);
        let mut buf = vec![0; pgr_memory_size];
        file.read_exact(&mut buf).unwrap();
        pgr_memory.load(0, &buf);

        let mut chr_memory = Ram::new(chr_memory_size);
        let mut buf = vec![0; chr_memory_size];
        file.read_exact(&mut buf).unwrap();
        chr_memory.load(0, &buf);

        let mut rest = Vec::new();
        file.read_to_end(&mut rest).unwrap();
        if !rest.is_empty() {
            panic!("This cartidge has more memory than expected!");
        }

        Self {
            name: game_name,
            program_ram: pgr_ram,
            program_rom: pgr_memory,
            character_memory: chr_memory,
        }
    }
}


impl std::fmt::Display for Cartidge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}


#[cfg(test)]
mod tests {
    use crate::interfaces::Memory;
    use super::*;

    #[test]
    fn test_cartidge_new() {
        let cartidge = Cartidge::new("roms/Super Mario Bros. (World).nes");
        assert_eq!(cartidge.program_rom.size(), 32 * 1024);
        assert_eq!(cartidge.character_memory.size(), 8 * 1024);
    }
}
