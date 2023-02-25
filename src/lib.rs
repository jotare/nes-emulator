/// NES emulator

mod cartidge;
mod interfaces;
mod nes;
mod processor;
mod utils;

pub use cartidge::Cartidge;
pub use nes::Nes;
