//! UI module
//!
//! This module abstract different UIs to render the NES output

mod gtk_ui;

pub use gtk_ui::GtkUi;

use crate::graphics::Frame;

pub const ORIGINAL_SCREEN_WIDTH: usize = crate::hardware::SCREEN_WIDTH;
pub const ORIGINAL_SCREEN_HEIGHT: usize = crate::hardware::SCREEN_HEIGHT;
pub const PIXEL_SCALE_FACTOR: usize = 4;

pub trait Ui {
    /// Trigger a render of a `frame`
    fn render(&self, frame: Frame);
}
