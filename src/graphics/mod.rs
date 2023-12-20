//! NES graphics hardware emulation

pub mod palette;
pub mod palette_memory;
pub mod ppu;

use crate::hardware::{SCREEN_HEIGHT, SCREEN_WIDTH};

#[derive(Copy, Clone, Debug)]
pub struct Pixel {
    red: f64,
    green: f64,
    blue: f64,
}

impl Pixel {
    pub const BLACK: Pixel = Pixel {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };
    pub const RED: Pixel = Pixel {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
    };
    pub const GREEN: Pixel = Pixel {
        red: 0.0,
        green: 1.0,
        blue: 0.0,
    };
    pub const BLUE: Pixel = Pixel {
        red: 0.0,
        green: 0.0,
        blue: 1.0,
    };
    pub const WHITE: Pixel = Pixel {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
    };

    pub fn new_rgb(red: f64, green: f64, blue: f64) -> Self {
        Self { red, green, blue }
    }

    pub fn new_rgb_byte(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red: red as f64 / u8::MAX as f64,
            green: green as f64 / u8::MAX as f64,
            blue: blue as f64 / u8::MAX as f64,
        }
    }

    pub fn red(&self) -> f64 {
        self.red
    }

    pub fn green(&self) -> f64 {
        self.green
    }

    pub fn blue(&self) -> f64 {
        self.blue
    }
}

/// Representation of a pixel in a Frame
pub struct FramePixel {
    pub row: usize,
    pub col: usize,
}

/// NES screen frame representation. It sizes are the same as the NES screen
/// (see hardware module)
pub struct Frame {
    pub inner: InnerFrame,
}

type InnerFrame = Vec<Vec<Pixel>>;

impl Frame {
    pub fn new(color: Pixel) -> Self {
        Self {
            inner: vec![vec![color; SCREEN_WIDTH]; SCREEN_HEIGHT],
        }
    }

    pub fn black() -> Self {
        Self::new(Pixel::BLACK)
    }

    pub fn set_pixel(&mut self, pixel: Pixel, position: FramePixel) {
        self.inner[position.row][position.col] = pixel;
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::black()
    }
}

impl std::ops::Deref for Frame {
    type Target = InnerFrame;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
