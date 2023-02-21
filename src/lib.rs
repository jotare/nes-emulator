/// NES emulator

mod cartidge;
mod interfaces;
mod nes;
mod processor;

pub use nes::Nes;
pub use cartidge::Cartidge;
