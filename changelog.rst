CHANGELOG
=========

0.61.1
------
- Palettes are always interpreted as 6-bit colors
- PPU vertical blank supress and small fixes

0.61.0
------
- Always set status register bit 5
- Fix controllers. Maintain state between reads and implement proper key
  pressed/released logic

0.60.0
------
- Change SharedBus for plain Bus references in CPU instruction set

0.59.1
------
- Proper PPU odd frame cycle skip implementation

0.59.0
------
- Implement cartridge support for CHR RAM
- Refactor mappers for more flexiblity

0.58.4
------
- Fix PPU priority multiplexer
- Implement sprite 0 hit
- Fix pixel rendering according to flags on PPUMASK

0.58.3
------
- Fix OAMADDR autoincrement after OAMDATA write
- Add sprite 0 hit flag to PPU registers
- Move pixel producer back to PPU module

0.58.2
------
- Refactor DMA
- Fix dummy cycle (wasn't reset appropriately)

0.58.1
------
- Add page boundary cross extra clock adjustments to CPU
- Add some more docs in CPU clock function

0.58.0
------
- Use internal mutability inside PpuRegisters instead of using a RefCell in Ppu

0.57.0
------
- Added metric for clock frequency

0.56.0
------
- Pixel-by-pixel sprite rendering implementation

0.55.0
------
- Implement primitive metrics collector to compute FPS

0.54.0
------
- Sprite rendering v1, sprites by scan line

0.53.0
------
- Sprite rendering v0, sprites literally over background

0.52.0
------
- Sprite rendernig v0, use red tiles for sprites

0.51.0
------
- Add debug logs to DMA
- Use wrapping_add instead of relying on wrapping behaviour (as it panics for
  other than --release)

0.50.0
------
- Implement OAM struct with better API than raw RAM memory
- Add OAM debugging function

0.49.1
------
- Fix fake APU overwritting bus addresses
- Add overlapping address validation in bus

0.49.0
------
- Implement OAM DMA

0.48.0
------
- Extend UI trait
- Better error handling

0.47.0
------
- Implement an event bus for inter-component communications (NMI, frame
  ready...)
- Better encapsulation of keyboard channel between UI and controllers
- Add switch off event so the system can stop gracefully
- Add nes function to setup TV (GUI + audio when implemented)

0.46.0
------
- PPU background rendering with scrolling
- Add palette mirrors
- Improve controllers

0.45.0
------
- Bus specifies a unique id per attached device. Interface and log improvements

0.44.0
------
- Add quit functionality to GTK UI using C-q
0.43.0
------
- Implement controllers using crossbeam mspc channels

0.42.0
------
- Add a Bus id
- Add Bus debug logs

0.41.0
------
- Extend palette memory functionality

0.40.0
------
- Implement attribute table use on PPU rendering

0.39.0
------
- Partially implement PPU, CIRAM and nametable rendering

0.38.2
------
- Improve CPU logs

0.38.1
------
- Further implement and fix CPU interrupts

0.38.0
------
- Add CPU interruption capabilities

0.37.1
------
- Fix CPU instructions and addressing modes

0.37.0
------
- Add opcode field to CPU Instruction

0.36.1
------
- Fix CPU instructions

0.36.0
------
- Remove unneeded trait Processor
- Refactor CPU and split in simpler modules

0.35.0
------
- Add new bit utility functions to set and clear bits

0.34.0
------
- Support mappers on cartidge and implement mapper 0

0.33.0
------
- Allow Pixel creation using u8
- Add new Palette type with blargg's palette

0.32.2
------
- Fix bv shift with overflow

0.32.1
------
- Fix inversion of screen at GtkUi
- Allow arbitrary screen size

0.32.0
------
- Memories are now shared and Nes have it's ownership

0.31.0
------
- CPU execute error is now a String

0.30.0
------
- Implement CPU instruction limit for test purposes

0.29.0
------
- Processor execute returns a Result

0.28.0
------
- Add graphics module with empty PPU and GTK4 UI
- Add PPU and memories to NES module

0.27.0
------
- Add CartidgeHeader struct and improve header parsing

0.26.0
------
- Add logging
- Use interior mutability pattern for Nes bus
- Fix various CPU errors

0.25.0
------
- Add ROM implementation to memory module

0.24.0
------
- Rename MainBus to DataBus

0.23.0
------
- Add MirroredRam and use it as main memory

0.22.0
------
- Add attach and detach methods to Bus trait

0.21.1
------
- Bus hides address range to attached devices

0.21.0
------
- Move traits to separate folder (to share between modules)

0.20.4
------
- Test branch instructions

0.20.3
------
- Fix reset by starting on reset vector address

0.20.2
------
- Fix SR push and pull in BRK and RTI

0.20.1
------
- Fix PC increment on jumps and interrupts

0.20.0
------
- Implement BRK and RTI instructions

0.19.1
------
- Fix PC increment

0.19.0
------
- Add bytes to CPU instructions

0.18.0
------
- Add push and pull misc instructions
- Add branch misc instructions
- Add jump misc instruction
- Add call and return misc instructions
- Prepare interrupt instructions

0.17.0
------
- Add AbsoluteX, AbsoluteY and IndirectY load addressing modes

0.16.0
------
- Complete instruction set (missing implementation for some
  instructions)

0.15.0
------
- Add BIT instruction

0.14.0
------
- Add branch instructions (wo/ tests)

0.13.0
------
- Add more addressing modes to load and store operations (wo/ tests)
- Add reset to CPU

0.12.0
------
- Add CMP, CPX and CPY comparaison instructions

0.11.0
------
- Add ADC and SBC arithmetic instructions
- Add ASL and LSR shift instructions
- Add ROR and ROL rotate instructions

0.10.0
------
- Remove ExecutableInstruction and split instruction depending on
  memory access
- Improve internal CPU instruction execution model
- Add STA, STX, STY store instructions
- Add DEC, INC instructions
- Add NOP instruction

0.9.0
-----
- Add DEX, DEY, INX, INY instructions

0.8.0
-----
- Add CLC, CLD, CLI, CLV, SEC, SED, SEI flag instructions

0.7.0
-----
- Add TAX, TAY, TSX, TXA, TXS, TYA transfer instructions

0.6.0
-----
- Add LDA, LDX and LDY load instructions

0.5.0
-----
- Convert Bus into a trait and rename struct to MainBus
- Move CPU tests to a separate file
- Reorder CPU module
- Update Nes with new cpu-bus architecture

0.4.0
-----
- Add macro to easily write CPU instructions
- Implement EOR and ORA instructions with immediate addressing

0.3.0
-----
- Start implementing the MOS 6502 processor
- Implement RAM
- Add CPU skeleton
- Implement AND instruction with immediate addressing

0.2.0
-----
- Add Nes and Cartidge abstractions and a dummy main program

0.1.0
-----
- Start NES emulator project
