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
