/// UI module
///
/// This module abstract different UIs to render the NES output
pub mod gtk_ui;

pub const ORIGINAL_SCREEN_WIDTH: usize = 256;
pub const ORIGINAL_SCREEN_HEIGHT: usize = 240;
pub const PIXEL_SCALE_FACTOR: usize = 4;

#[derive(Copy, Clone)]
pub struct Pixel {
    red: f64,
    green: f64,
    blue: f64,
}

impl Pixel {
    pub fn new_rgb(red: f64, green: f64, blue: f64) -> Self {
        Self { red, green, blue }
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

// pub type Frame = [[(f64, f64, f64); SCREEN_WIDTH]; SCREEN_HEIGHT];
pub type Frame = Vec<[Pixel; ORIGINAL_SCREEN_WIDTH]>;

pub trait Ui {}
