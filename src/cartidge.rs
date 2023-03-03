use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;

use log::debug;

use crate::processor::memory::{Ram, Rom};
use crate::utils::bv;

pub struct Cartidge {
    name: String,

    // Program memory (RAM)
    pub program_ram: Rc<RefCell<Ram>>,

    // Program memory (ROM)
    pub program_rom: Rc<RefCell<Rom>>,

    // Character memory, stores patterns and graphics for the PPU
    pub character_memory: Rc<RefCell<Ram>>,
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

        let mut header = [0; 16]; // 16 byte header
        file.read_exact(&mut header).unwrap();

        let cartidge_header = CartidgeHeader::parse(&header);
        debug!("Header: {cartidge_header:#?}");

        // Trainer content is ignored for now
        let _trainer = if cartidge_header.trainer {
            let mut buf = [0; 512]; // 512-byte trainer at 0x7000 - 0x71FF
            file.read_exact(&mut buf).unwrap();
            Some(buf)
        } else {
            None
        };

        let program_ram = Rc::new(RefCell::new(Ram::new(cartidge_header.pgr_ram_size)));

        let program_rom = Rc::new(RefCell::new(Rom::new(cartidge_header.pgr_rom_size)));
        let mut buf = vec![0; cartidge_header.pgr_rom_size];
        file.read_exact(&mut buf).unwrap();
        program_rom.borrow_mut().load(0, &buf);

        let character_memory = Rc::new(RefCell::new(Ram::new(cartidge_header.chr_rom_size)));
        let mut buf = vec![0; cartidge_header.chr_rom_size];
        file.read_exact(&mut buf).unwrap();
        character_memory.borrow_mut().load(0, &buf);

        let mut rest = Vec::new();
        file.read_to_end(&mut rest).unwrap();
        if !rest.is_empty() {
            panic!("This cartidge has more memory than expected!");
        }

        Self {
            name: game_name,
            program_ram,
            program_rom,
            character_memory,
        }
    }
}

impl std::fmt::Display for Cartidge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
enum Mirroring {
    /// Vertical arrangement (CIRAM A10 = PPU A11)
    Horizontal,

    /// Horizontal arrangement (CIRAM A10 = PPU A10)
    Vertical,
}

#[derive(Debug)]
struct CartidgeHeader {
    pub pgr_rom_size: usize,
    pub chr_rom_size: usize,
    pub mirroring: Mirroring,

    // 512-byte trainer at 0x7000-0x71FF (stored before PGR data)
    pub trainer: bool,

    // pub mapper: Box<dyn crate::mappers::Mapper>
    pub mapper: u8,

    pub pgr_ram_size: usize,
}

impl CartidgeHeader {
    fn parse(header: &[u8; 16]) -> Self {
        // (bytes 0-3) - NES cartidges started with ASCII "NES" and MS-DOS
        // end-of-file (0x1A)
        assert!(
            header[0..4] == [0x4E, 0x45, 0x53, 0x1A],
            "Invalid iNES header"
        );

        // (byte 4) - Size of PGR ROM in 16 KB units
        let pgr_rom_size = (header[4] as usize) * 16 * 1024;

        // (byte 5) - Size of CHR ROM in 8 KB units
        let chr_rom_size = (header[5] as usize) * 8 * 1024;

        // (byte 6) - Mapper, mirroring, battery, trainer
        let mirroring = if bv(header[6], 0) == 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };

        let trainer = bv(header[6], 2) != 0;

        let mapper_number = (header[7] & 0xF0) | ((header[6] & 0xF0) >> 4);
        // let mapper = crate::mappers::mapper_map(mapper_number);
        debug!("Cartidge mapper: {mapper_number}");

        // (byte 8) - PGR RAM size in 8 kB units (0 infers for 8 kB)
        let pgr_ram_size = if header[8] > 0 {
            (header[8] as usize) * 8 * 1024
        } else {
            8 * 1024
        };

        Self {
            pgr_rom_size,
            chr_rom_size,
            mirroring,
            trainer,
            mapper: mapper_number,
            pgr_ram_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interfaces::Memory;

    #[test]
    fn test_cartidge_new() {
        let cartidge = Cartidge::new("roms/Super Mario Bros. (World).nes");
        assert_eq!(cartidge.program_rom.borrow().size(), 32 * 1024);
        assert_eq!(cartidge.character_memory.borrow().size(), 8 * 1024);
    }
}
