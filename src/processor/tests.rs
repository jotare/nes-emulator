#![allow(non_snake_case)]

use crate::processor::instruction_set;
use crate::processor::instruction_set::*;
use crate::processor::internal_cpu::*;
use crate::processor::status_register::*;
use StatusRegisterFlag::*;

#[test]
fn test_load_instruction_LDA() {
    let mut cpu = InternalCpu::default();

    instruction_set::lda(&mut cpu, 0);
    assert_eq!(cpu.acc, 0);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));

    instruction_set::lda(&mut cpu, 0x95);
    assert_eq!(cpu.acc, 0x95);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));
}

#[test]
fn test_load_instruction_LDX() {
    let mut cpu = InternalCpu::default();

    instruction_set::ldx(&mut cpu, 0);
    assert_eq!(cpu.x_reg, 0);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));

    instruction_set::ldx(&mut cpu, 0x95);
    assert_eq!(cpu.x_reg, 0x95);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));
}

#[test]
fn test_load_instruction_LDY() {
    let mut cpu = InternalCpu::default();

    instruction_set::ldy(&mut cpu, 0);
    assert_eq!(cpu.y_reg, 0);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));

    instruction_set::ldy(&mut cpu, 0x95);
    assert_eq!(cpu.y_reg, 0x95);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));
}

#[test]
fn test_store_instruction_STA() {
    let mut cpu = InternalCpu {
        acc: 0x95,
        ..Default::default()
    };
    assert_eq!(instruction_set::sta(&mut cpu), 0x95);
}

#[test]
fn test_store_instruction_STX() {
    let mut cpu = InternalCpu {
        x_reg: 0x95,
        ..Default::default()
    };
    assert_eq!(instruction_set::stx(&mut cpu), 0x95);
}

#[test]
fn test_store_instruction_STY() {
    let mut cpu = InternalCpu {
        y_reg: 0x95,
        ..Default::default()
    };
    assert_eq!(instruction_set::sty(&mut cpu), 0x95);
}

#[test]
fn test_transfer_instruction_TAX() {
    let mut cpu = InternalCpu::default();

    cpu.acc = 0x82;
    assert_ne!(cpu.acc, cpu.x_reg);
    instruction_set::tax(&mut cpu);
    assert_eq!(cpu.acc, cpu.x_reg);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));

    cpu.acc = 0;
    instruction_set::tax(&mut cpu);
    assert_eq!(cpu.acc, cpu.x_reg);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
}

#[test]
fn test_transfer_instruction_TAY() {
    let mut cpu = InternalCpu::default();

    cpu.acc = 0x82;
    assert_ne!(cpu.acc, cpu.y_reg);
    instruction_set::tay(&mut cpu);
    assert_eq!(cpu.acc, cpu.y_reg);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));

    cpu.acc = 0;
    instruction_set::tay(&mut cpu);
    assert_eq!(cpu.acc, cpu.y_reg);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
}

#[test]
fn test_transfer_instruction_TSX() {
    let mut cpu = InternalCpu::default();

    cpu.sp = 0x82;
    assert_ne!(cpu.sp, cpu.x_reg);
    instruction_set::tsx(&mut cpu);
    assert_eq!(cpu.sp, cpu.x_reg);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));

    cpu.sp = 0;
    instruction_set::tsx(&mut cpu);
    assert_eq!(cpu.sp, cpu.x_reg);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
}

#[test]
fn test_transfer_instruction_TXA() {
    let mut cpu = InternalCpu::default();

    cpu.x_reg = 0x82;
    assert_ne!(cpu.x_reg, cpu.acc);
    instruction_set::txa(&mut cpu);
    assert_eq!(cpu.x_reg, cpu.acc);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));

    cpu.x_reg = 0;
    instruction_set::txa(&mut cpu);
    assert_eq!(cpu.x_reg, cpu.acc);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
}

#[test]
fn test_transfer_instruction_TXS() {
    let mut cpu = InternalCpu::default();

    cpu.x_reg = 0x82;
    assert_ne!(cpu.x_reg, cpu.sp);
    instruction_set::txs(&mut cpu);
    assert_eq!(cpu.x_reg, cpu.sp);
    assert!(!cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));

    cpu.x_reg = 0;
    instruction_set::txs(&mut cpu);
    assert_eq!(cpu.x_reg, cpu.sp);
    assert!(!cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
}

#[test]
fn test_transfer_instruction_TYA() {
    let mut cpu = InternalCpu::default();

    cpu.y_reg = 0x82;
    assert_ne!(cpu.y_reg, cpu.acc);
    instruction_set::tya(&mut cpu);
    assert_eq!(cpu.y_reg, cpu.acc);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));

    cpu.y_reg = 0;
    instruction_set::tya(&mut cpu);
    assert_eq!(cpu.y_reg, cpu.acc);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
}

#[test]
fn test_decrement_instruction_DEC() {
    let mut cpu = InternalCpu::default();

    assert_eq!(dec(&mut cpu, 0x82), 0x81);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));

    assert_eq!(instruction_set::dec(&mut cpu, 1), 0);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));

    instruction_set::dex(&mut cpu);
    assert_eq!(cpu.x_reg, 0xFF);
}

#[test]
fn test_decrement_instruction_DEX() {
    let mut cpu = InternalCpu::default();

    cpu.x_reg = 0x82;
    instruction_set::dex(&mut cpu);
    assert_eq!(cpu.x_reg, 0x81);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));

    cpu.x_reg = 1;
    instruction_set::dex(&mut cpu);
    assert_eq!(cpu.x_reg, 0);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));

    instruction_set::dex(&mut cpu);
    assert_eq!(cpu.x_reg, 0xFF);
}

#[test]
fn test_decrement_instruction_DEY() {
    let mut cpu = InternalCpu::default();

    cpu.y_reg = 0x82;
    instruction_set::dey(&mut cpu);
    assert_eq!(cpu.y_reg, 0x81);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));

    cpu.y_reg = 1;
    instruction_set::dey(&mut cpu);
    assert_eq!(cpu.y_reg, 0);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));

    instruction_set::dey(&mut cpu);
    assert_eq!(cpu.y_reg, 0xFF);
}

#[test]
fn test_load_instruction_INC() {
    let mut cpu = InternalCpu::default();

    assert_eq!(instruction_set::inc(&mut cpu, 0x82), 0x83);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));

    assert_eq!(instruction_set::inc(&mut cpu, 0xFF), 0);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
}

#[test]
fn test_load_instruction_INX() {
    let mut cpu = InternalCpu::default();

    cpu.x_reg = 0x82;
    instruction_set::inx(&mut cpu);
    assert_eq!(cpu.x_reg, 0x83);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));

    cpu.x_reg = 0xFF;
    instruction_set::inx(&mut cpu);
    assert_eq!(cpu.x_reg, 0);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
}

#[test]
fn test_load_instruction_INY() {
    let mut cpu = InternalCpu::default();

    cpu.y_reg = 0x82;
    instruction_set::iny(&mut cpu);
    assert_eq!(cpu.y_reg, 0x83);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));

    cpu.y_reg = 0xFF;
    instruction_set::iny(&mut cpu);
    assert_eq!(cpu.y_reg, 0);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
}

#[test]
fn test_arithmetic_instruction_ADC() {
    let mut cpu = InternalCpu::default();

    cpu.acc = 5;
    cpu.sr.clear(Carry);
    instruction_set::adc(&mut cpu, 2);
    assert_eq!(cpu.acc, 7);
    assert!(!cpu.sr.get(Negative));
    assert!(!cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Carry));
    assert!(!cpu.sr.get(Overflow));

    cpu.acc = 5;
    cpu.sr.set(Carry);
    instruction_set::adc(&mut cpu, 2);
    assert_eq!(cpu.acc, 8);
    assert!(!cpu.sr.get(Negative));
    assert!(!cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Carry));
    assert!(!cpu.sr.get(Overflow));

    cpu.acc = 0xFF;
    cpu.sr.clear(Carry);
    instruction_set::adc(&mut cpu, 1);
    assert_eq!(cpu.acc, 0);
    assert!(!cpu.sr.get(Negative));
    assert!(cpu.sr.get(Zero));
    assert!(cpu.sr.get(Carry));
    assert!(!cpu.sr.get(Overflow));

    cpu.acc = 0xFF;
    cpu.sr.set(Carry);
    instruction_set::adc(&mut cpu, 0xFF);
    assert_eq!(cpu.acc, 0xFF);
    assert!(cpu.sr.get(Negative));
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Carry));
    assert!(!cpu.sr.get(Overflow));

    cpu.acc = 0x80;
    cpu.sr.clear(Carry);
    instruction_set::adc(&mut cpu, 0x80);
    assert!(cpu.sr.get(Overflow));
}

#[test]
fn test_arithmetic_instruction_SBC() {
    let mut cpu = InternalCpu::default();

    cpu.sr.set(Carry);
    instruction_set::sbc(&mut cpu, 0);
    assert_eq!(cpu.acc, 0);
    assert!(!cpu.sr.get(Negative));
    assert!(cpu.sr.get(Zero));
    assert!(cpu.sr.get(Carry));
    assert!(!cpu.sr.get(Overflow));

    cpu.sr.clear(Carry);
    instruction_set::sbc(&mut cpu, 0);
    assert_eq!(cpu.acc, 0xFF);
    assert!(cpu.sr.get(Negative));
    assert!(!cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Carry));
    assert!(!cpu.sr.get(Overflow));

    // // C = 0; 5 - 4 - (1 - C) = 0
    // cpu.acc = 5;
    // cpu.sr.clear(Carry);
    // instruction_set::sbc(&mut cpu, 4);
    // assert_eq!(cpu.acc, 0);
    // assert!(!cpu.sr.get(Negative));
    // assert!(cpu.sr.get(Zero));
    // assert!(!cpu.sr.get(Carry));
    // assert!(!cpu.sr.get(Overflow));

    // // C = 1; 5 - 2 - (1 - C) = 3
    // cpu.acc = 5;
    // cpu.sr.set(Carry);
    // instruction_set::sbc(&mut cpu, 2);
    // assert_eq!(cpu.acc, 3);
    // assert!(!cpu.sr.get(Negative));
    // assert!(!cpu.sr.get(Zero));
    // assert!(!cpu.sr.get(Carry));
    // assert!(!cpu.sr.get(Overflow));

    // // C = 0; 0 - 1 - (1 - C) = -2 = 0xFE
    // cpu.acc = 0;
    // cpu.sr.clear(Carry);
    // instruction_set::sbc(&mut cpu, 1);
    // assert_eq!(cpu.acc, 0xFE);
    // assert!(cpu.sr.get(Negative));
    // assert!(!cpu.sr.get(Zero));
    // assert!(cpu.sr.get(Carry));
    // assert!(cpu.sr.get(Overflow));

    // // C = 1; 0 - 1 - (1 - C) = -1 = 0xFF
    // cpu.acc = 0;
    // cpu.sr.set(Carry);
    // instruction_set::sbc(&mut cpu, 1);
    // assert_eq!(cpu.acc, 0xFF);
    // assert!(cpu.sr.get(Negative));
    // assert!(!cpu.sr.get(Zero));
    // assert!(cpu.sr.get(Carry));
    // assert!(cpu.sr.get(Overflow));
}

#[test]
fn test_logical_instruction_AND() {
    let mut cpu = InternalCpu::default();
    cpu.acc = 0xAC;

    // A AND 0xFF = A = 0xAC
    instruction_set::and(&mut cpu, 0xFF);
    assert_eq!(cpu.acc, 0xAC);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));

    // A AND 0x0F = 0x0C
    instruction_set::and(&mut cpu, 0x0F);
    assert_eq!(cpu.acc, 0x0C);
    assert!(!cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));

    // A AND 0x00 = 0x00
    instruction_set::and(&mut cpu, 0x00);
    assert_eq!(cpu.acc, 0x00);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
}

#[test]
fn test_logical_instruction_EOR() {
    let mut cpu = InternalCpu::default();
    cpu.acc = 0xEF;

    // A EOR 0x88 = 0x67
    instruction_set::eor(&mut cpu, 0x88);
    assert_eq!(cpu.acc, 0x67);
    assert!(!cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));

    // A EOR 0x67 = 0x00
    instruction_set::eor(&mut cpu, 0x67);
    assert_eq!(cpu.acc, 0x00);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));

    // A EOR 0x80 = 0x80
    instruction_set::eor(&mut cpu, 0x80);
    assert_eq!(cpu.acc, 0x80);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));
}

#[test]
fn test_logical_instruction_ORA() {
    let mut cpu = InternalCpu::default();
    cpu.acc = 0x00;

    // A ORA 0x00 = 0x00
    instruction_set::ora(&mut cpu, 0x00);
    assert_eq!(cpu.acc, 0x00);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));

    // A ORA 0xAB = 0xAB
    instruction_set::ora(&mut cpu, 0xAB);
    assert_eq!(cpu.acc, 0xAB);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));

    // A ORA 0xCC = 0xEF
    instruction_set::ora(&mut cpu, 0xCC);
    assert_eq!(cpu.acc, 0xEF);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));
}

#[test]
fn test_shift_instruction_ASL_ACC() {
    let mut cpu = InternalCpu::default();

    cpu.acc = 2;
    instruction_set::asl_acc(&mut cpu);
    assert_eq!(cpu.acc, 4);
    assert!(!cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
    assert!(!cpu.sr.get(Carry));

    cpu.acc = 0x40;
    instruction_set::asl_acc(&mut cpu);
    assert_eq!(cpu.acc, 0x80);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));
    assert!(!cpu.sr.get(Carry));

    cpu.acc = 0x80;
    instruction_set::asl_acc(&mut cpu);
    assert_eq!(cpu.acc, 0);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
    assert!(cpu.sr.get(Carry));
}

#[test]
fn test_shift_instruction_LSR_ACC() {
    let mut cpu = InternalCpu::default();

    cpu.acc = 1;
    instruction_set::lsr_acc(&mut cpu);
    assert_eq!(cpu.acc, 0);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
    assert!(cpu.sr.get(Carry));

    cpu.acc = 0x40;
    instruction_set::lsr_acc(&mut cpu);
    assert_eq!(cpu.acc, 0x20);
    assert!(!cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
    assert!(!cpu.sr.get(Carry));
}

#[test]
fn test_rotate_instruction_ROL_ACC() {
    let mut cpu = InternalCpu::default();

    cpu.acc = 0b1111_0000;
    cpu.sr.set(Carry);
    instruction_set::rol_acc(&mut cpu);
    assert_eq!(cpu.acc, 0b1110_0001);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));
    assert!(cpu.sr.get(Carry));

    cpu.acc = 0b1000_0000;
    cpu.sr.clear(Carry);
    instruction_set::rol_acc(&mut cpu);
    assert_eq!(cpu.acc, 0);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
    assert!(cpu.sr.get(Carry));

    cpu.acc = 0b0000_1000;
    cpu.sr.clear(Carry);
    instruction_set::rol_acc(&mut cpu);
    assert_eq!(cpu.acc, 0b0001_0000);
    assert!(!cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
    assert!(!cpu.sr.get(Carry));
}

#[test]
fn test_rotate_instruction_ROR_ACC() {
    let mut cpu = InternalCpu::default();

    cpu.acc = 0b0000_1111;
    cpu.sr.set(Carry);
    instruction_set::ror_acc(&mut cpu);
    assert_eq!(cpu.acc, 0b1000_0111);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));
    assert!(cpu.sr.get(Carry));

    cpu.acc = 0b0000_0001;
    cpu.sr.clear(Carry);
    instruction_set::ror_acc(&mut cpu);
    assert_eq!(cpu.acc, 0);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
    assert!(cpu.sr.get(Carry));

    cpu.acc = 0b0001_0000;
    cpu.sr.clear(Carry);
    instruction_set::ror_acc(&mut cpu);
    assert_eq!(cpu.acc, 0b0000_1000);
    assert!(!cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
    assert!(!cpu.sr.get(Carry));
}

#[test]
fn test_flag_instruction_CLC() {
    let mut cpu = InternalCpu::default();

    cpu.sr.set(Carry);
    instruction_set::clc(&mut cpu);
    assert!(!cpu.sr.get(Carry));
}

#[test]
fn test_flag_instruction_CLD() {
    let mut cpu = InternalCpu::default();

    cpu.sr.set(Decimal);
    instruction_set::cld(&mut cpu);
    assert!(!cpu.sr.get(Decimal));
}

#[test]
fn test_flag_instruction_CLI() {
    let mut cpu = InternalCpu::default();

    cpu.sr.set(InterruptDisable);
    instruction_set::cli(&mut cpu);
    assert!(!cpu.sr.get(InterruptDisable));
}

#[test]
fn test_flag_instruction_CLV() {
    let mut cpu = InternalCpu::default();

    cpu.sr.set(Overflow);
    instruction_set::clv(&mut cpu);
    assert!(!cpu.sr.get(Overflow));
}

#[test]
fn test_flag_instruction_SEC() {
    let mut cpu = InternalCpu::default();

    cpu.sr.clear(Carry);
    instruction_set::sec(&mut cpu);
    assert!(cpu.sr.get(Carry));
}

#[test]
fn test_flag_instruction_SED() {
    let mut cpu = InternalCpu::default();

    cpu.sr.clear(Decimal);
    instruction_set::sed(&mut cpu);
    assert!(cpu.sr.get(Decimal));
}

#[test]
fn test_flag_instruction_SEI() {
    let mut cpu = InternalCpu::default();

    cpu.sr.clear(InterruptDisable);
    instruction_set::sei(&mut cpu);
    assert!(cpu.sr.get(InterruptDisable));
}

#[test]
fn test_comparaison_instruction_CMP() {
    let mut cpu = InternalCpu::default();

    cpu.acc = 10;
    instruction_set::cmp(&mut cpu, 5);
    assert!(!cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
    assert!(cpu.sr.get(Carry));

    cpu.acc = 5;
    instruction_set::cmp(&mut cpu, 5);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
    assert!(cpu.sr.get(Carry));

    cpu.acc = 0x80;
    instruction_set::cmp(&mut cpu, 0xA0);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));
    assert!(!cpu.sr.get(Carry));
}

#[test]
fn test_comparaison_instruction_CPX() {
    let mut cpu = InternalCpu::default();
    cpu.x_reg = 10;
    instruction_set::cpx(&mut cpu, 5);
    assert!(!cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
    assert!(cpu.sr.get(Carry));

    cpu.x_reg = 5;
    instruction_set::cpx(&mut cpu, 5);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
    assert!(cpu.sr.get(Carry));

    cpu.x_reg = 0x80;
    instruction_set::cpx(&mut cpu, 0xA0);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));
    assert!(!cpu.sr.get(Carry));
}

#[test]
fn test_comparaison_instruction_CPY() {
    let mut cpu = InternalCpu::default();

    cpu.y_reg = 10;
    instruction_set::cpy(&mut cpu, 5);
    assert!(!cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
    assert!(cpu.sr.get(Carry));

    cpu.y_reg = 5;
    instruction_set::cpy(&mut cpu, 5);
    assert!(cpu.sr.get(Zero));
    assert!(!cpu.sr.get(Negative));
    assert!(cpu.sr.get(Carry));

    cpu.y_reg = 0x80;
    instruction_set::cpy(&mut cpu, 0xA0);
    assert!(!cpu.sr.get(Zero));
    assert!(cpu.sr.get(Negative));
    assert!(!cpu.sr.get(Carry));
}

#[test]
fn test_branch_instruction_BCC() {
    let mut cpu = InternalCpu::default();
    let pc = cpu.pc;

    cpu.sr.set(Carry);
    instruction_set::bcc(&mut cpu, 10);
    assert_eq!(cpu.pc, pc);

    cpu.sr.clear(Carry);
    instruction_set::bcc(&mut cpu, 10);
    assert_eq!(cpu.pc, pc + 10);
}

#[test]
fn test_branch_instruction_BCS() {
    let mut cpu = InternalCpu::default();
    let pc = cpu.pc;

    cpu.sr.clear(Carry);
    instruction_set::bcs(&mut cpu, 10);
    assert_eq!(cpu.pc, pc);

    cpu.sr.set(Carry);
    instruction_set::bcs(&mut cpu, 10);
    assert_eq!(cpu.pc, pc + 10);
}

#[test]
fn test_branch_instruction_BEQ() {
    let mut cpu = InternalCpu::default();
    let pc = cpu.pc;

    cpu.sr.clear(Zero);
    instruction_set::beq(&mut cpu, 10);
    assert_eq!(cpu.pc, pc);

    cpu.sr.set(Zero);
    instruction_set::beq(&mut cpu, 10);
    assert_eq!(cpu.pc, pc + 10);
}

#[test]
fn test_branch_instruction_BMI() {
    let mut cpu = InternalCpu::default();
    let pc = cpu.pc;

    cpu.sr.clear(Negative);
    instruction_set::bmi(&mut cpu, 10);
    assert_eq!(cpu.pc, pc);

    cpu.sr.set(Negative);
    instruction_set::bmi(&mut cpu, 10);
    assert_eq!(cpu.pc, pc + 10);
}

#[test]
fn test_branch_instruction_BNE() {
    let mut cpu = InternalCpu::default();
    let pc = cpu.pc;

    cpu.sr.set(Zero);
    instruction_set::bne(&mut cpu, 10);
    assert_eq!(cpu.pc, pc);

    cpu.sr.clear(Zero);
    instruction_set::bne(&mut cpu, 10);
    assert_eq!(cpu.pc, pc + 10);
}

#[test]
fn test_branch_instruction_BPL() {
    let mut cpu = InternalCpu::default();
    let pc = cpu.pc;

    cpu.sr.set(Negative);
    instruction_set::bpl(&mut cpu, 10);
    assert_eq!(cpu.pc, pc);

    cpu.sr.clear(Negative);
    instruction_set::bpl(&mut cpu, 10);
    assert_eq!(cpu.pc, pc + 10);
}

#[test]
fn test_branch_instruction_BVC() {
    let mut cpu = InternalCpu::default();
    let pc = cpu.pc;

    cpu.sr.set(Overflow);
    instruction_set::bvc(&mut cpu, 10);
    assert_eq!(cpu.pc, pc);

    cpu.sr.clear(Overflow);
    instruction_set::bvc(&mut cpu, 10);
    assert_eq!(cpu.pc, pc + 10);
}

#[test]
fn test_branch_instruction_BVS() {
    let mut cpu = InternalCpu::default();
    let pc = cpu.pc;

    cpu.sr.clear(Overflow);
    instruction_set::bvs(&mut cpu, 10);
    assert_eq!(cpu.pc, pc);

    cpu.sr.set(Overflow);
    instruction_set::bvs(&mut cpu, 10);
    assert_eq!(cpu.pc, pc + 10);
}
