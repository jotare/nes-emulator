use mockall::mock;
use mockall::predicate::eq;

use super::*;

mock! {
    TestBus {}

    impl Bus for TestBus {
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
    let mock_bus = Rc::new(MockTestBus::new());
    let cpu = Cpu::new(mock_bus);

    cpu
}

// Get a CPU with mocked peripherials and a loaded program
fn test_cpu_with_program(program: Vec<u8>) -> Cpu {
    let mut mock_bus = MockTestBus::new();
    mock_bus.load_program(program);

    let mock_bus = Rc::new(mock_bus);
    let cpu = Cpu::new(mock_bus);

    cpu
}

//////////////////////////////////////////////////////////////////////
// TEST INSTRUCTION SET
//////////////////////////////////////////////////////////////////////

#[test]
#[allow(non_snake_case)]
fn test_AND_instruction() {
    let mut cpu = test_cpu();
    cpu.acc = 0xAC;

    // A AND 0xFF = A = 0xAC
    cpu.and(0xFF);
    assert_eq!(cpu.acc, 0xAC);
    assert!(!cpu.get_flag(Zero));
    assert!(cpu.get_flag(Negative));

    // A AND 0x0F = 0x0C
    cpu.and(0x0F);
    assert_eq!(cpu.acc, 0x0C);
    assert!(!cpu.get_flag(Zero));
    assert!(!cpu.get_flag(Negative));

    // A AND 0x00 = 0x00
    cpu.and(0x00);
    assert_eq!(cpu.acc, 0x00);
    assert!(cpu.get_flag(Zero));
    assert!(!cpu.get_flag(Negative));
}

#[test]
#[allow(non_snake_case)]
fn test_EOR_instruction() {
    let mut cpu = test_cpu();
    cpu.acc = 0xEF;

    // A EOR 0x88 = 0x67
    cpu.eor(0x88);
    assert_eq!(cpu.acc, 0x67);
    assert!(!cpu.get_flag(Zero));
    assert!(!cpu.get_flag(Negative));

    // A EOR 0x67 = 0x00
    cpu.eor(0x67);
    assert_eq!(cpu.acc, 0x00);
    assert!(cpu.get_flag(Zero));
    assert!(!cpu.get_flag(Negative));

    // A EOR 0x80 = 0x80
    cpu.eor(0x80);
    assert_eq!(cpu.acc, 0x80);
    assert!(!cpu.get_flag(Zero));
    assert!(cpu.get_flag(Negative));
}

#[test]
#[allow(non_snake_case)]
fn test_ORA_instruction() {
    let mut cpu = test_cpu();
    cpu.acc = 0x00;

    // A ORA 0x00 = 0x00
    cpu.ora(0x00);
    assert_eq!(cpu.acc, 0x00);
    assert!(cpu.get_flag(Zero));
    assert!(!cpu.get_flag(Negative));

    // A ORA 0xAB = 0xAB
    cpu.ora(0xAB);
    assert_eq!(cpu.acc, 0xAB);
    assert!(!cpu.get_flag(Zero));
    assert!(cpu.get_flag(Negative));

    // A ORA 0xCC = 0xEF
    cpu.ora(0xCC);
    assert_eq!(cpu.acc, 0xEF);
    assert!(!cpu.get_flag(Zero));
    assert!(cpu.get_flag(Negative));
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

#[test]
fn test_addressing_mode_zero_page() {
    let mut cpu = test_cpu_with_program(vec![
        // 
    ]);
}

//////////////////////////////////////////////////////////////////////
// TEST STATUS REGISTER
//////////////////////////////////////////////////////////////////////

#[test]
fn test_status_register() {
    let mut cpu = test_cpu();
    let flags = vec![
        StatusRegisterFlag::Carry,
        StatusRegisterFlag::Zero,
        StatusRegisterFlag::DisableInterrupts,
        StatusRegisterFlag::Break,
        StatusRegisterFlag::Overflow,
        StatusRegisterFlag::Negative,
    ];

    for flag in flags {
        assert!(!cpu.get_flag(flag));
        cpu.set_flag(flag, true);
        assert!(cpu.get_flag(flag));
        cpu.set_flag(flag, false);
        assert!(!cpu.get_flag(flag));
    }
}
