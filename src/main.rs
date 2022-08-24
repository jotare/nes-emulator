use std::path::Path;

use nes_emulator::{Nes, Catridge};

fn main() {
    let mut nes = Nes::default();
    let cartidge = Catridge::new(Path::new("/path/to/cartidge"));

    nes.load_cartidge(cartidge);
    nes.run();
}
