use std::convert::From;

use crate::utils;

// Bring local enum variants to scope
use StatusRegisterFlag::*;

#[derive(Copy, Clone, Default)]
pub struct StatusRegister {
    sr: u8,
}

impl StatusRegister {
    pub fn new() -> Self {
        Self { sr: 0 }
    }

    pub fn reset(&mut self) {
        self.sr = 0;
    }

    pub fn get(&self, flag: StatusRegisterFlag) -> bool {
        utils::bv(self.sr, flag as u8) > 0
    }

    pub fn set(&mut self, flag: StatusRegisterFlag) {
        self.sr = utils::set_bit(self.sr, flag as u8);
    }

    pub fn clear(&mut self, flag: StatusRegisterFlag) {
        self.sr = utils::clear_bit(self.sr, flag as u8);
    }

    pub fn set_value(&mut self, flag: StatusRegisterFlag, condition: bool) {
        match condition {
            true => self.set(flag),
            false => self.clear(flag),
        }
    }

    pub fn auto_set(&mut self, flag: StatusRegisterFlag, value: u8) {
        let condition = match flag {
            Zero => value == 0,
            Negative => (value as i8) < 0,
            _ => panic!("Auto set flag {flag:?} not implemented"),
        };

        self.set_value(flag, condition);
    }
}

impl From<u8> for StatusRegister {
    fn from(value: u8) -> Self {
        Self { sr: value }
    }
}

impl From<StatusRegister> for u8 {
    fn from(value: StatusRegister) -> Self {
        value.sr
    }
}

#[derive(Copy, Clone, Debug)]
pub enum StatusRegisterFlag {
    Negative = 7,
    Overflow = 6,
    // bit 5 is unused and is always 1
    Break = 4,
    Decimal = 3, // unused in the NES
    InterruptDisable = 2,
    Zero = 1,
    Carry = 0,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_register_all() {
        let mut sr = StatusRegister::default();

        let flags = vec![
            Carry,
            Zero,
            InterruptDisable,
            Decimal,
            Break,
            Overflow,
            Negative,
        ];

        for flag in flags {
            assert!(!sr.get(flag));
            sr.set(flag);
            assert!(sr.get(flag));
            sr.clear(flag);
            assert!(!sr.get(flag));
        }
    }

    #[test]
    fn test_status_register_get() {
        let sr = StatusRegister {
            sr: (1 << Negative as u8) | (1 << Zero as u8),
        };

        assert!(sr.get(Negative));
        assert!(sr.get(Zero));
        assert!(!sr.get(Overflow));
    }

    #[test]
    fn test_status_register_set() {
        let mut sr = StatusRegister::default();
        assert!(!sr.get(InterruptDisable));

        sr.set(InterruptDisable);
        assert!(sr.get(InterruptDisable));
    }

    #[test]
    fn test_status_register_clear() {
        let mut sr = StatusRegister {
            sr: (1 << Carry as u8),
        };
        assert!(sr.get(Carry));

        sr.clear(Carry);
        assert!(!sr.get(Carry));
    }
}
