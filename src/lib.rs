/// NES emulator

mod catridge;
mod interfaces;
mod nes;
mod processor;

pub use nes::Nes;
pub use catridge::Catridge;
