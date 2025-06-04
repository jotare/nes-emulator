use crate::graphics::Pixel;

impl From<u8> for Pixel {
    /// Convert a color to it's RGB representation using NTSC video encoding
    ///
    /// Palette taken from blargg's full palette demo:
    /// https://www.nesdev.org/wiki/PPU_palettes
    /// http://forums.nesdev.org/viewtopic.php?f=2&t=6484
    fn from(color: u8) -> Self {
        match color & 0x3F {
            0x00 => Self::new_rgb_byte(84, 84, 84),
            0x01 => Self::new_rgb_byte(0, 30, 116),
            0x02 => Self::new_rgb_byte(8, 16, 144),
            0x03 => Self::new_rgb_byte(48, 0, 136),
            0x04 => Self::new_rgb_byte(68, 0, 100),
            0x05 => Self::new_rgb_byte(92, 0, 48),
            0x06 => Self::new_rgb_byte(84, 4, 0),
            0x07 => Self::new_rgb_byte(60, 24, 0),
            0x08 => Self::new_rgb_byte(32, 42, 0),
            0x09 => Self::new_rgb_byte(8, 58, 0),
            0x0A => Self::new_rgb_byte(0, 64, 0),
            0x0B => Self::new_rgb_byte(0, 60, 0),
            0x0C => Self::new_rgb_byte(0, 50, 60),
            0x0D => Self::new_rgb_byte(0, 0, 0),
            0x0E => Self::new_rgb_byte(0, 0, 0),
            0x0F => Self::new_rgb_byte(0, 0, 0),

            0x10 => Self::new_rgb_byte(152, 150, 152),
            0x11 => Self::new_rgb_byte(8, 76, 196),
            0x12 => Self::new_rgb_byte(48, 50, 236),
            0x13 => Self::new_rgb_byte(92, 30, 228),
            0x14 => Self::new_rgb_byte(136, 20, 176),
            0x15 => Self::new_rgb_byte(160, 20, 100),
            0x16 => Self::new_rgb_byte(152, 34, 32),
            0x17 => Self::new_rgb_byte(120, 60, 0),
            0x18 => Self::new_rgb_byte(84, 90, 0),
            0x19 => Self::new_rgb_byte(40, 114, 0),
            0x1A => Self::new_rgb_byte(8, 124, 0),
            0x1B => Self::new_rgb_byte(0, 118, 40),
            0x1C => Self::new_rgb_byte(0, 102, 120),
            0x1D => Self::new_rgb_byte(0, 0, 0),
            0x1E => Self::new_rgb_byte(0, 0, 0),
            0x1F => Self::new_rgb_byte(0, 0, 0),

            0x20 => Self::new_rgb_byte(236, 238, 236),
            0x21 => Self::new_rgb_byte(76, 154, 236),
            0x22 => Self::new_rgb_byte(120, 124, 236),
            0x23 => Self::new_rgb_byte(176, 98, 236),
            0x24 => Self::new_rgb_byte(228, 84, 236),
            0x25 => Self::new_rgb_byte(236, 88, 180),
            0x26 => Self::new_rgb_byte(236, 106, 100),
            0x27 => Self::new_rgb_byte(212, 136, 32),
            0x28 => Self::new_rgb_byte(160, 170, 0),
            0x29 => Self::new_rgb_byte(116, 196, 0),
            0x2A => Self::new_rgb_byte(76, 208, 32),
            0x2B => Self::new_rgb_byte(56, 204, 108),
            0x2C => Self::new_rgb_byte(56, 180, 204),
            0x2D => Self::new_rgb_byte(60, 60, 60),
            0x2E => Self::new_rgb_byte(0, 0, 0),
            0x2F => Self::new_rgb_byte(0, 0, 0),

            0x30 => Self::new_rgb_byte(236, 238, 236),
            0x31 => Self::new_rgb_byte(168, 204, 236),
            0x32 => Self::new_rgb_byte(188, 188, 236),
            0x33 => Self::new_rgb_byte(212, 178, 236),
            0x34 => Self::new_rgb_byte(236, 174, 236),
            0x35 => Self::new_rgb_byte(236, 174, 212),
            0x36 => Self::new_rgb_byte(236, 180, 176),
            0x37 => Self::new_rgb_byte(228, 196, 144),
            0x38 => Self::new_rgb_byte(204, 210, 120),
            0x39 => Self::new_rgb_byte(180, 222, 120),
            0x3A => Self::new_rgb_byte(168, 226, 144),
            0x3B => Self::new_rgb_byte(152, 226, 180),
            0x3C => Self::new_rgb_byte(160, 214, 228),
            0x3D => Self::new_rgb_byte(160, 162, 160),
            0x3E => Self::new_rgb_byte(0, 0, 0),
            0x3F => Self::new_rgb_byte(0, 0, 0),

            _ => unreachable!("Invalid color {color}"),
        }
    }
}
