use std::fs::File;
use std::io::Read;
use std::path::Path;

use log::debug;

use crate::mappers::mapper_map;
use crate::mappers::{Mapper, MapperSpecs};
use crate::processor::memory::Mirroring;
use crate::utils::bv;

pub struct Cartidge {
    name: String,
    pub mapper: Box<dyn Mapper>,
    header: CartidgeHeader,
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
    /// - iNES file format is expected and can panic if a different file format
    /// is used.
    /// - It can also panics if an invalid path is provided
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        if !path.as_ref().exists() {
            panic!(
                "Game {:?} not found. Make sure the path is correct",
                path.as_ref().as_os_str()
            );
        }

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

        let mapper_specs = MapperSpecs {
            program_ram_capacity: cartidge_header.pgr_ram_size,
            program_rom_capacity: cartidge_header.pgr_rom_size,
            character_rom_capacity: cartidge_header.chr_rom_size,
            character_ram: cartidge_header.chr_ram,
        };
        let mut mapper = mapper_map(cartidge_header.mapper, mapper_specs);

        let mut buf = vec![0; cartidge_header.pgr_rom_size];
        file.read_exact(&mut buf).unwrap();
        mapper.load_program_memory(buf);

        let mut buf = vec![0; cartidge_header.chr_rom_size];
        file.read_exact(&mut buf).unwrap();
        mapper.load_character_memory(buf);

        let mut rest = Vec::new();
        file.read_to_end(&mut rest).unwrap();
        if !rest.is_empty() {
            panic!("This cartidge has more memory than expected!");
        }

        Self {
            name: game_name,
            mapper,
            header: cartidge_header,
        }
    }

    pub fn mirroring(&self) -> Mirroring {
        self.header.mirroring
    }
}

impl std::fmt::Display for Cartidge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
struct CartidgeHeader {
    pub pgr_rom_size: usize,
    pub chr_rom_size: usize,
    pub mirroring: Mirroring,

    pub chr_ram: bool,

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

        // (byte 5) - Size of CHR ROM in 8 KB units (or usage of CHR RAM)
        let chr_ram = header[5] == 0;
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
            chr_ram,
            mirroring,
            trainer,
            mapper: mapper_number,
            pgr_ram_size,
        }
    }
}

// Doesn't make sense, as the mapper doesn't expose it's memories anymore

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_cartidge_new() {
//         let cartidge = Cartidge::new("roms/Super Mario Bros. (World).nes");

//         assert_eq!(cartidge.mapper.program_rom_ref().borrow().size(), 32 * 1024);
//         assert_eq!(
//             cartidge.mapper.character_memory_ref().borrow().size(),
//             8 * 1024
//         );
//     }
// }
