//! UI module
//!
//! This module abstract different UIs to render the NES output

mod gtk_ui;

pub use gtk_ui::GtkUi;

use crate::graphics::Frame;

pub trait Ui {
    /// Trigger a render of a `frame`
    fn render(&self, frame: Frame);
}
