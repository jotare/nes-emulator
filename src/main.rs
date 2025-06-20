// use std::path::Path;

use nes_emulator::{Cartidge, ControllerButtons, Nes};

fn main() {
    env_logger::init();

    let mut nes = Nes::default();
    nes.setup_tv();
    nes.connect_controller_one(ControllerButtons::default());
    nes.connect_controller_two(ControllerButtons::default());
    // let cartidge = Cartidge::new(Path::new("/path/to/cartidge"));
    let cartidge = Cartidge::new("roms/Super Mario Bros. (World).nes");
    // let cartidge = Cartidge::new("roms/Galaga - Demons of Death (USA).nes");

    nes.load_cartidge(cartidge);
    nes.run().unwrap();
}
