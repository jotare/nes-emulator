//! Load a game and run NES CPU forever.
//!
//! NES games might depend from the PPU to do some work, so this example can run
//! in an infinite loop executing the same instructions all over again.
//!

use std::thread;
use std::time::Duration;


use log::{error, LevelFilter};

use nes_emulator::{Cartidge, Nes};

// ATENTION! ROMs are not provided in this repository, you should download your
// owns and change this path.
const CARTIDGE_PATH: &str = "roms/Super Mario Bros. (World).nes";

fn main() {
    env_logger::builder()
        .filter(Some("nes_emulator::processor::cpu"), LevelFilter::Debug)
        .init();

    let mut nes = Nes::new();
    let cartidge = Cartidge::new(CARTIDGE_PATH);

    nes.load_cartidge(cartidge);

    loop {
        let result = nes.cpu.execute();
        if let Err(error) = result {
            error!("CPU execution error: {error}");
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
}
