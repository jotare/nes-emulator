#![allow(non_snake_case)]

use mockall::mock;
use mockall::predicate::eq;

use super::*;
use crate::interfaces::{Bus, Memory, AddressRange};

mock! {
    TestBus {}

    impl Bus for TestBus {
        fn attach(&mut self, device: Box<dyn Memory>, addr_range: AddressRange) -> usize;
        fn detach(&mut self, id: usize);
        fn read(&self, address: u16) -> u8;
        fn write(&self, address: u16, data: u8);
    }
}

impl MockTestBus {
    fn load_program(&mut self, program: Vec<u8>) {
        for (addr, value) in program.iter().enumerate() {
            let (addr, value) = (addr as u16, *value as u8);
            self.expect_read().with(eq(addr)).return_const(value);
        }
    }
}

fn test_cpu() -> Cpu {
    let mut mock_bus = MockTestBus::new();

    // reset will jump to start address
    mock_bus.expect_read().with(eq(0xFFFC)).return_const(0);
    mock_bus.expect_read().with(eq(0xFFFD)).return_const(0);

    let mock_bus = Rc::new(RefCell::new(mock_bus));
    let mock_bus_ptr = Rc::clone(&mock_bus);
    let cpu = Cpu::new(mock_bus_ptr);

    cpu
}

// Get a CPU with mocked peripherials and a loaded program
fn test_cpu_with_program(program: Vec<u8>) -> Cpu {
    let mut mock_bus = MockTestBus::new();

    // reset will jump to start address
    mock_bus.expect_read().with(eq(0xFFFC)).return_const(0);
    mock_bus.expect_read().with(eq(0xFFFD)).return_const(0);

    mock_bus.load_program(program);

    let mock_bus = Rc::new(RefCell::new(mock_bus));
    let mock_bus_ptr = Rc::clone(&mock_bus);
    let cpu = Cpu::new(mock_bus);

    cpu
}

//////////////////////////////////////////////////////////////////////
// TEST INSTRUCTION SET
//////////////////////////////////////////////////////////////////////

#[test]
fn test_load_instruction_LDA() {
    let mut cpu = test_cpu();

    cpu.lda(0);
    assert_eq!(cpu.acc, 0);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));

    cpu.lda(0x95);
    assert_eq!(cpu.acc, 0x95);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));
}

#[test]
fn test_load_instruction_LDX() {
    let mut cpu = test_cpu();

    cpu.ldx(0);
    assert_eq!(cpu.x_reg, 0);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));

    cpu.ldx(0x95);
    assert_eq!(cpu.x_reg, 0x95);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));
}

#[test]
fn test_load_instruction_LDY() {
    let mut cpu = test_cpu();

    cpu.ldy(0);
    assert_eq!(cpu.y_reg, 0);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));

    cpu.ldy(0x95);
    assert_eq!(cpu.y_reg, 0x95);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));
}

#[test]
fn test_store_instruction_STA() {
    let mut cpu = test_cpu();

    cpu.acc = 0;
    assert_eq!(cpu.sta(), 0);

    cpu.acc = 0x95;
    assert_eq!(cpu.sta(), 0x95);
}

#[test]
fn test_store_instruction_STX() {
    let mut cpu = test_cpu();

    cpu.x_reg = 0;
    assert_eq!(cpu.stx(), 0);

    cpu.x_reg = 0x95;
    assert_eq!(cpu.stx(), 0x95);
}

#[test]
fn test_store_instruction_STY() {
    let mut cpu = test_cpu();

    cpu.y_reg = 0;
    assert_eq!(cpu.sty(), 0);

    cpu.y_reg = 0x95;
    assert_eq!(cpu.sty(), 0x95);
}

#[test]
fn test_transfer_instruction_TAX() {
    let mut cpu = test_cpu();

    cpu.acc = 0x82;
    assert_ne!(cpu.acc, cpu.x_reg);
    cpu.tax();
    assert_eq!(cpu.acc, cpu.x_reg);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));

    cpu.acc = 0;
    cpu.tax();
    assert_eq!(cpu.acc, cpu.x_reg);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
}

#[test]
fn test_transfer_instruction_TAY() {
    let mut cpu = test_cpu();

    cpu.acc = 0x82;
    assert_ne!(cpu.acc, cpu.y_reg);
    cpu.tay();
    assert_eq!(cpu.acc, cpu.y_reg);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));

    cpu.acc = 0;
    cpu.tay();
    assert_eq!(cpu.acc, cpu.y_reg);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
}

#[test]
fn test_transfer_instruction_TSX() {
    let mut cpu = test_cpu();

    cpu.sp = 0x82;
    assert_ne!(cpu.sp, cpu.x_reg);
    cpu.tsx();
    assert_eq!(cpu.sp, cpu.x_reg);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));

    cpu.sp = 0;
    cpu.tsx();
    assert_eq!(cpu.sp, cpu.x_reg);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
}

#[test]
fn test_transfer_instruction_TXA() {
    let mut cpu = test_cpu();

    cpu.x_reg = 0x82;
    assert_ne!(cpu.x_reg, cpu.acc);
    cpu.txa();
    assert_eq!(cpu.x_reg, cpu.acc);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));

    cpu.x_reg = 0;
    cpu.txa();
    assert_eq!(cpu.x_reg, cpu.acc);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
}

#[test]
fn test_transfer_instruction_TXS() {
    let mut cpu = test_cpu();

    cpu.x_reg = 0x82;
    assert_ne!(cpu.x_reg, cpu.sp);
    cpu.txs();
    assert_eq!(cpu.x_reg, cpu.sp);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));

    cpu.x_reg = 0;
    cpu.txs();
    assert_eq!(cpu.x_reg, cpu.sp);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
}

#[test]
fn test_transfer_instruction_TYA() {
    let mut cpu = test_cpu();

    cpu.y_reg = 0x82;
    assert_ne!(cpu.y_reg, cpu.acc);
    cpu.tya();
    assert_eq!(cpu.y_reg, cpu.acc);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));

    cpu.y_reg = 0;
    cpu.tya();
    assert_eq!(cpu.y_reg, cpu.acc);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
}

#[test]
fn test_decrement_instruction_DEC() {
    let mut cpu = test_cpu();

    assert_eq!(cpu.dec(0x82), 0x81);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));

    assert_eq!(cpu.dec(1), 0);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));

    cpu.dex();
    assert_eq!(cpu.x_reg, 0xFF);
}

#[test]
fn test_decrement_instruction_DEX() {
    let mut cpu = test_cpu();

    cpu.x_reg = 0x82;
    cpu.dex();
    assert_eq!(cpu.x_reg, 0x81);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));

    cpu.x_reg = 1;
    cpu.dex();
    assert_eq!(cpu.x_reg, 0);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));

    cpu.dex();
    assert_eq!(cpu.x_reg, 0xFF);
}

#[test]
fn test_decrement_instruction_DEY() {
    let mut cpu = test_cpu();

    cpu.y_reg = 0x82;
    cpu.dey();
    assert_eq!(cpu.y_reg, 0x81);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));

    cpu.y_reg = 1;
    cpu.dey();
    assert_eq!(cpu.y_reg, 0);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));

    cpu.dey();
    assert_eq!(cpu.y_reg, 0xFF);
}

#[test]
fn test_load_instruction_INC() {
    let mut cpu = test_cpu();

    assert_eq!(cpu.inc(0x82), 0x83);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));

    assert_eq!(cpu.inc(0xFF), 0);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
}

#[test]
fn test_load_instruction_INX() {
    let mut cpu = test_cpu();

    cpu.x_reg = 0x82;
    cpu.inx();
    assert_eq!(cpu.x_reg, 0x83);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));

    cpu.x_reg = 0xFF;
    cpu.inx();
    assert_eq!(cpu.x_reg, 0);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
}

#[test]
fn test_load_instruction_INY() {
    let mut cpu = test_cpu();

    cpu.y_reg = 0x82;
    cpu.iny();
    assert_eq!(cpu.y_reg, 0x83);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));

    cpu.y_reg = 0xFF;
    cpu.iny();
    assert_eq!(cpu.y_reg, 0);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
}

#[test]
fn test_arithmetic_instruction_ADC() {
    let mut cpu = test_cpu();

    cpu.acc = 5;
    cpu.set_flag(Carry, false);
    cpu.adc(2);
    assert_eq!(cpu.acc, 7);
    assert!(!cpu.flag(Negative));
    assert!(!cpu.flag(Zero));
    assert!(!cpu.flag(Carry));
    assert!(!cpu.flag(Overflow));

    cpu.acc = 5;
    cpu.set_flag(Carry, true);
    cpu.adc(2);
    assert_eq!(cpu.acc, 8);
    assert!(!cpu.flag(Negative));
    assert!(!cpu.flag(Zero));
    assert!(!cpu.flag(Carry));
    assert!(!cpu.flag(Overflow));

    cpu.acc = 0xFF;
    cpu.set_flag(Carry, false);
    cpu.adc(1);
    assert_eq!(cpu.acc, 0);
    assert!(!cpu.flag(Negative));
    assert!(cpu.flag(Zero));
    assert!(cpu.flag(Carry));
    assert!(!cpu.flag(Overflow));

    cpu.acc = 0xFF;
    cpu.set_flag(Carry, true);
    cpu.adc(0xFF);
    assert_eq!(cpu.acc, 0xFF);
    assert!(cpu.flag(Negative));
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Carry));
    assert!(!cpu.flag(Overflow));

    cpu.acc = 0x80;
    cpu.set_flag(Carry, false);
    cpu.adc(0x80);
    assert!(cpu.flag(Overflow));
}

#[test]
fn test_arithmetic_instruction_SBC() {
    let mut cpu = test_cpu();

    cpu.acc = 5;
    cpu.set_flag(Carry, false);
    cpu.sbc(2);
    assert_eq!(cpu.acc, 3);
    assert!(!cpu.flag(Negative));
    assert!(!cpu.flag(Zero));
    assert!(!cpu.flag(Carry));
    assert!(!cpu.flag(Overflow));

    cpu.acc = 5;
    cpu.set_flag(Carry, true);
    cpu.sbc(4);
    assert_eq!(cpu.acc, 0);
    assert!(!cpu.flag(Negative));
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Carry));
    assert!(!cpu.flag(Overflow));

    cpu.acc = 0;
    cpu.set_flag(Carry, false);
    cpu.sbc(1);
    assert_eq!(cpu.acc, 0xFF);
    assert!(cpu.flag(Negative));
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Carry));
    assert!(cpu.flag(Overflow));

    cpu.acc = 0;
    cpu.set_flag(Carry, true);
    cpu.sbc(1);
    assert_eq!(cpu.acc, 0xFE);
    assert!(cpu.flag(Negative));
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Carry));
    assert!(cpu.flag(Overflow));
}

#[test]
fn test_logical_instruction_AND() {
    let mut cpu = test_cpu();
    cpu.acc = 0xAC;

    // A AND 0xFF = A = 0xAC
    cpu.and(0xFF);
    assert_eq!(cpu.acc, 0xAC);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));

    // A AND 0x0F = 0x0C
    cpu.and(0x0F);
    assert_eq!(cpu.acc, 0x0C);
    assert!(!cpu.flag(Zero));
    assert!(!cpu.flag(Negative));

    // A AND 0x00 = 0x00
    cpu.and(0x00);
    assert_eq!(cpu.acc, 0x00);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
}

#[test]
fn test_logical_instruction_EOR() {
    let mut cpu = test_cpu();
    cpu.acc = 0xEF;

    // A EOR 0x88 = 0x67
    cpu.eor(0x88);
    assert_eq!(cpu.acc, 0x67);
    assert!(!cpu.flag(Zero));
    assert!(!cpu.flag(Negative));

    // A EOR 0x67 = 0x00
    cpu.eor(0x67);
    assert_eq!(cpu.acc, 0x00);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));

    // A EOR 0x80 = 0x80
    cpu.eor(0x80);
    assert_eq!(cpu.acc, 0x80);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));
}

#[test]
fn test_logical_instruction_ORA() {
    let mut cpu = test_cpu();
    cpu.acc = 0x00;

    // A ORA 0x00 = 0x00
    cpu.ora(0x00);
    assert_eq!(cpu.acc, 0x00);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));

    // A ORA 0xAB = 0xAB
    cpu.ora(0xAB);
    assert_eq!(cpu.acc, 0xAB);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));

    // A ORA 0xCC = 0xEF
    cpu.ora(0xCC);
    assert_eq!(cpu.acc, 0xEF);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));
}

#[test]
fn test_shift_instruction_ASL() {
    let mut cpu = test_cpu();

    cpu.acc = 2;
    cpu.asl();
    assert_eq!(cpu.acc, 4);
    assert!(!cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
    assert!(!cpu.flag(Carry));

    cpu.acc = 0x40;
    cpu.asl();
    assert_eq!(cpu.acc, 0x80);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));
    assert!(!cpu.flag(Carry));

    cpu.acc = 0x80;
    cpu.asl();
    assert_eq!(cpu.acc, 0);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
    assert!(cpu.flag(Carry));
}

#[test]
fn test_shift_instruction_LSR() {
    let mut cpu = test_cpu();

    cpu.acc = 1;
    cpu.lsr();
    assert_eq!(cpu.acc, 0);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
    assert!(cpu.flag(Carry));

    cpu.acc = 0x40;
    cpu.lsr();
    assert_eq!(cpu.acc, 0x20);
    assert!(!cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
    assert!(!cpu.flag(Carry));
}

#[test]
fn test_rotate_instruction_ROL() {
    let mut cpu = test_cpu();

    cpu.acc = 0b1111_0000;
    cpu.set_flag(Carry, true);
    cpu.rol();
    assert_eq!(cpu.acc, 0b1110_0001);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));
    assert!(cpu.flag(Carry));

    cpu.acc = 0b1000_0000;
    cpu.set_flag(Carry, false);
    cpu.rol();
    assert_eq!(cpu.acc, 0);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
    assert!(cpu.flag(Carry));

    cpu.acc = 0b0000_1000;
    cpu.set_flag(Carry, false);
    cpu.rol();
    assert_eq!(cpu.acc, 0b0001_0000);
    assert!(!cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
    assert!(!cpu.flag(Carry));
}

#[test]
fn test_rotate_instruction_ROR() {
    let mut cpu = test_cpu();

    cpu.acc = 0b0000_1111;
    cpu.set_flag(Carry, true);
    cpu.ror();
    assert_eq!(cpu.acc, 0b1000_0111);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));
    assert!(cpu.flag(Carry));

    cpu.acc = 0b0000_0001;
    cpu.set_flag(Carry, false);
    cpu.ror();
    assert_eq!(cpu.acc, 0);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
    assert!(cpu.flag(Carry));

    cpu.acc = 0b0001_0000;
    cpu.set_flag(Carry, false);
    cpu.ror();
    assert_eq!(cpu.acc, 0b0000_1000);
    assert!(!cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
    assert!(!cpu.flag(Carry));
}

#[test]
fn test_flag_instruction_CLC() {
    let mut cpu = test_cpu();

    cpu.set_flag(Carry, true);
    cpu.clc();
    assert!(!cpu.flag(Carry));
}

#[test]
fn test_flag_instruction_CLD() {
    let mut cpu = test_cpu();

    cpu.set_flag(Decimal, true);
    cpu.cld();
    assert!(!cpu.flag(Decimal));
}

#[test]
fn test_flag_instruction_CLI() {
    let mut cpu = test_cpu();

    cpu.set_flag(InterruptDisable, true);
    cpu.cli();
    assert!(!cpu.flag(InterruptDisable));
}

#[test]
fn test_flag_instruction_CLV() {
    let mut cpu = test_cpu();

    cpu.set_flag(Overflow, true);
    cpu.clv();
    assert!(!cpu.flag(Overflow));
}

#[test]
fn test_flag_instruction_SEC() {
    let mut cpu = test_cpu();

    cpu.set_flag(Carry, false);
    cpu.sec();
    assert!(cpu.flag(Carry));
}

#[test]
fn test_flag_instruction_SED() {
    let mut cpu = test_cpu();

    cpu.set_flag(Decimal, false);
    cpu.sed();
    assert!(cpu.flag(Decimal));
}

#[test]
fn test_flag_instruction_SEI() {
    let mut cpu = test_cpu();

    cpu.set_flag(InterruptDisable, false);
    cpu.sei();
    assert!(cpu.flag(InterruptDisable));
}

#[test]
fn test_comparaison_instruction_CMP() {
    let mut cpu = test_cpu();

    cpu.acc = 10;
    cpu.cmp(5);
    assert!(!cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
    assert!(cpu.flag(Carry));

    cpu.acc = 5;
    cpu.cmp(5);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
    assert!(cpu.flag(Carry));

    cpu.acc = 0x80;
    cpu.cmp(0xA0);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));
    assert!(!cpu.flag(Carry));
}

#[test]
fn test_comparaison_instruction_CPX() {
    let mut cpu = test_cpu();
    cpu.x_reg = 10;
    cpu.cpx(5);
    assert!(!cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
    assert!(cpu.flag(Carry));

    cpu.x_reg = 5;
    cpu.cpx(5);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
    assert!(cpu.flag(Carry));

    cpu.x_reg = 0x80;
    cpu.cpx(0xA0);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));
    assert!(!cpu.flag(Carry));
}

#[test]
fn test_comparaison_instruction_CPY() {
    let mut cpu = test_cpu();

    cpu.y_reg = 10;
    cpu.cpy(5);
    assert!(!cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
    assert!(cpu.flag(Carry));

    cpu.y_reg = 5;
    cpu.cpy(5);
    assert!(cpu.flag(Zero));
    assert!(!cpu.flag(Negative));
    assert!(cpu.flag(Carry));

    cpu.y_reg = 0x80;
    cpu.cpy(0xA0);
    assert!(!cpu.flag(Zero));
    assert!(cpu.flag(Negative));
    assert!(!cpu.flag(Carry));
}

#[test]
fn test_branch_instruction_BCC() {
    let mut cpu = test_cpu();
    let pc = cpu.pc;

    cpu.set_flag(Carry, true);
    cpu.bcc(10);
    assert_eq!(cpu.pc, pc);

    cpu.set_flag(Carry, false);
    cpu.bcc(10);
    assert_eq!(cpu.pc, pc + 10);
}

#[test]
fn test_branch_instruction_BCS() {
    let mut cpu = test_cpu();
    let pc = cpu.pc;

    cpu.set_flag(Carry, false);
    cpu.bcs(10);
    assert_eq!(cpu.pc, pc);

    cpu.set_flag(Carry, true);
    cpu.bcs(10);
    assert_eq!(cpu.pc, pc + 10 + 1);
}

#[test]
fn test_branch_instruction_BEQ() {
    let mut cpu = test_cpu();
    let pc = cpu.pc;

    cpu.set_flag(Zero, false);
    cpu.beq(10);
    assert_eq!(cpu.pc, pc);

    cpu.set_flag(Zero, true);
    cpu.beq(10);
    assert_eq!(cpu.pc, pc + 10);
}

#[test]
fn test_branch_instruction_BMI() {
    let mut cpu = test_cpu();
    let pc = cpu.pc;

    cpu.set_flag(Negative, false);
    cpu.bmi(10);
    assert_eq!(cpu.pc, pc);

    cpu.set_flag(Negative, true);
    cpu.bmi(10);
    assert_eq!(cpu.pc, pc + 10);
}

#[test]
fn test_branch_instruction_BNE() {
    let mut cpu = test_cpu();
    let pc = cpu.pc;

    cpu.set_flag(Zero, true);
    cpu.bne(10);
    assert_eq!(cpu.pc, pc);

    cpu.set_flag(Zero, false);
    cpu.bne(10);
    assert_eq!(cpu.pc, pc + 10);
}

#[test]
fn test_branch_instruction_BPL() {
    let mut cpu = test_cpu();
    let pc = cpu.pc;

    cpu.set_flag(Negative, true);
    cpu.bpl(10);
    assert_eq!(cpu.pc, pc);

    cpu.set_flag(Negative, false);
    cpu.bpl(10);
    assert_eq!(cpu.pc, pc + 10);
}

#[test]
fn test_branch_instruction_BVC() {
    let mut cpu = test_cpu();
    let pc = cpu.pc;

    cpu.set_flag(Overflow, true);
    cpu.bvc(10);
    assert_eq!(cpu.pc, pc);

    cpu.set_flag(Overflow, false);
    cpu.bvc(10);
    assert_eq!(cpu.pc, pc + 10);
}

#[test]
fn test_branch_instruction_BVS() {
    env_logger::builder().is_test(true).try_init();

    let mut cpu = test_cpu();
    let pc = cpu.pc;

    cpu.set_flag(Overflow, false);
    cpu.bvs(10);
    assert_eq!(cpu.pc, pc);

    cpu.set_flag(Overflow, true);
    cpu.bvs(10);
    assert_eq!(cpu.pc, pc + 10);
}

//////////////////////////////////////////////////////////////////////
// TEST ADDRESSING MODES
//////////////////////////////////////////////////////////////////////

#[test]
fn test_addressing_mode_immediate() {
    let mut cpu = test_cpu_with_program(vec![
        // immediate AND: A (0x72) AND 0xAB = 0x22
        0x29, 0xAB,
    ]);
    cpu.acc = 0x72;

    cpu.execute();
    assert_eq!(cpu.acc, 0x22);
}

// TODO: test all addressing modes

//////////////////////////////////////////////////////////////////////
// TEST INSTRUCTION KINDS
//////////////////////////////////////////////////////////////////////

// TODO: test instruction kinds

//////////////////////////////////////////////////////////////////////
// TEST STATUS REGISTER
//////////////////////////////////////////////////////////////////////

#[test]
fn test_status_register() {
    let mut cpu = test_cpu();
    let flags = vec![
        StatusRegisterFlag::Carry,
        StatusRegisterFlag::Zero,
        StatusRegisterFlag::InterruptDisable,
        StatusRegisterFlag::Break,
        StatusRegisterFlag::Overflow,
        StatusRegisterFlag::Negative,
    ];

    for flag in flags {
        assert!(!cpu.flag(flag));
        cpu.set_flag(flag, true);
        assert!(cpu.flag(flag));
        cpu.set_flag(flag, false);
        assert!(!cpu.flag(flag));
    }
}
