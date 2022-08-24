CHANGELOG
=========

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
