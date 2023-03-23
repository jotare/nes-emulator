use std::collections::HashMap;

use crate::graphics::ui::Pixel;

pub struct Palette {
    palette: HashMap<u8, Pixel>,
}

impl Palette {
    /// Palette take from blargg's full palette demo:
    /// https://www.nesdev.org/wiki/PPU_palettes
    /// http://forums.nesdev.org/viewtopic.php?f=2&t=6484
    pub fn new() -> Self {
        let mut palette = HashMap::new();

        palette.insert(0x00, Pixel::new_rgb_byte(84, 84, 84));
        palette.insert(0x01, Pixel::new_rgb_byte(0, 30, 116));
        palette.insert(0x02, Pixel::new_rgb_byte(8, 16, 144));
        palette.insert(0x03, Pixel::new_rgb_byte(48, 0, 136));
        palette.insert(0x04, Pixel::new_rgb_byte(68, 0, 100));
        palette.insert(0x05, Pixel::new_rgb_byte(92, 0, 48));
        palette.insert(0x06, Pixel::new_rgb_byte(84, 4, 0));
        palette.insert(0x07, Pixel::new_rgb_byte(60, 24, 0));
        palette.insert(0x08, Pixel::new_rgb_byte(32, 42, 0));
        palette.insert(0x09, Pixel::new_rgb_byte(8, 58, 0));
        palette.insert(0x0A, Pixel::new_rgb_byte(0, 64, 0));
        palette.insert(0x0B, Pixel::new_rgb_byte(0, 60, 0));
        palette.insert(0x0C, Pixel::new_rgb_byte(0, 50, 60));
        palette.insert(0x0D, Pixel::new_rgb_byte(0, 0, 0));
        palette.insert(0x0E, Pixel::new_rgb_byte(0, 0, 0));
        palette.insert(0x0F, Pixel::new_rgb_byte(0, 0, 0));

        palette.insert(0x10, Pixel::new_rgb_byte(152, 150, 152));
        palette.insert(0x11, Pixel::new_rgb_byte(8, 76, 196));
        palette.insert(0x12, Pixel::new_rgb_byte(48, 50, 236));
        palette.insert(0x13, Pixel::new_rgb_byte(92, 30, 228));
        palette.insert(0x14, Pixel::new_rgb_byte(136, 20, 176));
        palette.insert(0x15, Pixel::new_rgb_byte(160, 20, 100));
        palette.insert(0x16, Pixel::new_rgb_byte(152, 34, 32));
        palette.insert(0x17, Pixel::new_rgb_byte(120, 60, 0));
        palette.insert(0x18, Pixel::new_rgb_byte(84, 90, 0));
        palette.insert(0x19, Pixel::new_rgb_byte(40, 114, 0));
        palette.insert(0x1A, Pixel::new_rgb_byte(8, 124, 0));
        palette.insert(0x1B, Pixel::new_rgb_byte(0, 118, 40));
        palette.insert(0x1C, Pixel::new_rgb_byte(0, 102, 120));
        palette.insert(0x1D, Pixel::new_rgb_byte(0, 0, 0));
        palette.insert(0x1E, Pixel::new_rgb_byte(0, 0, 0));
        palette.insert(0x1F, Pixel::new_rgb_byte(0, 0, 0));

        palette.insert(0x20, Pixel::new_rgb_byte(236, 238, 236));
        palette.insert(0x21, Pixel::new_rgb_byte(76, 154, 236));
        palette.insert(0x22, Pixel::new_rgb_byte(120, 124, 236));
        palette.insert(0x23, Pixel::new_rgb_byte(176, 98, 236));
        palette.insert(0x24, Pixel::new_rgb_byte(228, 84, 236));
        palette.insert(0x25, Pixel::new_rgb_byte(236, 88, 180));
        palette.insert(0x26, Pixel::new_rgb_byte(236, 106, 100));
        palette.insert(0x27, Pixel::new_rgb_byte(212, 136, 32));
        palette.insert(0x28, Pixel::new_rgb_byte(160, 170, 0));
        palette.insert(0x29, Pixel::new_rgb_byte(116, 196, 0));
        palette.insert(0x2A, Pixel::new_rgb_byte(76, 208, 32));
        palette.insert(0x2B, Pixel::new_rgb_byte(56, 204, 108));
        palette.insert(0x2C, Pixel::new_rgb_byte(56, 180, 204));
        palette.insert(0x2D, Pixel::new_rgb_byte(60, 60, 60));
        palette.insert(0x2E, Pixel::new_rgb_byte(0, 0, 0));
        palette.insert(0x2F, Pixel::new_rgb_byte(0, 0, 0));

        palette.insert(0x30, Pixel::new_rgb_byte(236, 238, 236));
        palette.insert(0x31, Pixel::new_rgb_byte(168, 204, 236));
        palette.insert(0x32, Pixel::new_rgb_byte(188, 188, 236));
        palette.insert(0x33, Pixel::new_rgb_byte(212, 178, 236));
        palette.insert(0x34, Pixel::new_rgb_byte(236, 174, 236));
        palette.insert(0x35, Pixel::new_rgb_byte(236, 174, 212));
        palette.insert(0x36, Pixel::new_rgb_byte(236, 180, 176));
        palette.insert(0x37, Pixel::new_rgb_byte(228, 196, 144));
        palette.insert(0x38, Pixel::new_rgb_byte(204, 210, 120));
        palette.insert(0x39, Pixel::new_rgb_byte(180, 222, 120));
        palette.insert(0x3A, Pixel::new_rgb_byte(168, 226, 144));
        palette.insert(0x3B, Pixel::new_rgb_byte(152, 226, 180));
        palette.insert(0x3C, Pixel::new_rgb_byte(160, 214, 228));
        palette.insert(0x3D, Pixel::new_rgb_byte(160, 162, 160));
        palette.insert(0x3E, Pixel::new_rgb_byte(0, 0, 0));
        palette.insert(0x3F, Pixel::new_rgb_byte(0, 0, 0));

        Self { palette }
    }

    pub fn decode_pixel(&self, color: u8) -> Pixel {
        *self.palette.get(&color).expect("Invalid color {color}")
    }
}
