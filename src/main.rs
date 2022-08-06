use std::path::Path;

use nes_emulator::{Nes, Cartidge};

fn main() {
    let mut nes = Nes::default();
    let cartidge = Cartidge::new(Path::new("/path/to/cartidge"));

    nes.load_cartidge(cartidge);
    nes.run();
}
