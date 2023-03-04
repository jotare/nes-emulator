# NES Emulator

Nintendo Entertainment System (NES) Emulator writen in Rust

This project is under development and it's still far to be a full emulator.
Although, any contribution is welcome.


## Run nes-emulator

*nes-emulator* can be run with:
``` bash
cargo run
```

For the moment, it'll run a binary with a hardcoded path to a cartidge. Open
*src/main.rs* and change the path to load you NES game.

NES games must be in iNES file format.


## Test nes-emulator

To run *nes-emulator* tests, execute:

``` bash
cargo test
```


## Features

### Done

- CPU (MOS6502) emulated
- Memories (RAM, ROM and mirrored RAMs)
- Buses
- GTK4 GUI capable to render frames
- Partial Maper-0 support

### Work in progress

- PPU (Picture Processing Unit)
- Mapper-0 suppot
- Support more mappers

### TODOs and ideas

- APU (Audio Processing Unit)
- Web interface (compiling to web assembly)

## Dependencies

### GTK-4 Graphical interface

When using GTK-4 desktop application to render NES screen, GTK-4 library is
needed. To install the necessary dependencies, refer to [GTK-4 Rust bindings
documentation](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation.html).
For Linux users based on Debian and derivates, install the following libraries:
- libgtk-4-dev
- build-essential
