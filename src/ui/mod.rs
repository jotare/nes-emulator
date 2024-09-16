//! UI module
//!
//! This module abstract different UIs to render the NES output

mod gtk_ui;

pub use gtk_ui::GtkUi;

use crate::errors::UiError;
use crate::graphics::Frame;

pub trait Ui {
    /// Start the UI. An unstarted UI won't render
    fn start(&mut self) -> Result<(), UiError>;

    /// Trigger a render of a `frame`
    fn render(&mut self, frame: Frame);

    /// Synchronously stop the UI
    fn stop(&mut self) -> Result<(), UiError>;
}
