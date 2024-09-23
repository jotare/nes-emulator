//! NES emulator
//!

#![allow(dead_code, unused_variables)]

mod cartidge;
mod controller;
mod dma;
pub mod errors;
pub mod events;
pub mod graphics;
pub mod hardware;
pub mod interfaces;
mod mappers;
mod metrics;
mod nes;
mod processor;
pub mod settings;
mod types;
pub mod ui;
pub mod utils;

pub use cartidge::Cartidge;
pub use controller::ControllerButtons;
pub use nes::Nes;
