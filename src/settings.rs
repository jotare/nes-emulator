/// NES configuration options
pub struct NesSettings {
    /// UI setting: scale factor applied to screen pixels to increase image
    /// size. A scale factor of 2 will make original pixels render as 2x2 pixel
    /// squares
    pub pixel_scale_factor: usize,

    pub ui_kind: UiKind,
}

pub const DEFAULT_PIXEL_SCALE_FACTOR: usize = 4;

pub enum UiKind {
    None,
    Gtk,
}

impl Default for NesSettings {
    fn default() -> Self {
        Self {
            pixel_scale_factor: DEFAULT_PIXEL_SCALE_FACTOR,
            ui_kind: UiKind::Gtk,
        }
    }
}
