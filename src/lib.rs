/// NES emulator

mod cartidge;
mod interfaces;
mod nes;
mod processor;
mod utils;

pub use nes::Nes;
pub use cartidge::Cartidge;
