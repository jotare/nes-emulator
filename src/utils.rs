/// Return the bit value for `value` at bit position `bit`
pub fn bv(value: u8, bit: u8) -> u8 {
    value.rotate_right(bit.into()) & 1
}

pub fn bv_16(value: u16, bit: u8) -> u16 {
    value.rotate_right(bit.into()) & 1
}

/// Return the value from `value` between bit positions `major_bit` and
/// `minor_bit`
pub fn bvs_8(value: u8, major_bit: u8, minor_bit: u8) -> u8 {
    (value >> minor_bit) & ((1 << (major_bit - minor_bit + 1)) - 1)
}

pub fn set_bit(byte: u8, bit: u8) -> u8 {
    byte | (1 << bit)
}

pub fn clear_bit(byte: u8, bit: u8) -> u8 {
    byte & (!(1 << bit))
}

/// Single or group of bits that represent some kind of flag or restricted set of
/// values. A group **must** be a consecutive group of 1s!
#[derive(Copy, Clone)]
pub struct BitGroup<T> {
    group: T,
}

impl BitGroup<u16> {
    pub fn new(value: u16) -> Self {
        Self { group: value }
    }

    pub fn get(&self, group: impl Into<BitGroup<u16>>) -> u16 {
        let group: u16 = group.into().into();
        (self.group & group) >> group.trailing_zeros()
    }

    pub fn set(&mut self, group: impl Into<BitGroup<u16>>, value: u16) {
        let group: u16 = group.into().into();
        self.clear(group);
        self.group |= value << group.trailing_zeros();
    }

    pub fn overflowing_add(&mut self, group: impl Into<BitGroup<u16>>, increment: u16) -> bool {
        let group: u16 = group.into().into();
        let value = self.get(group);

        let modulo = (group >> group.trailing_zeros()) + 1;
        let overflow = value + increment >= modulo;
        self.set(group, value + increment % modulo);

        overflow
    }

    pub fn toggle(&mut self, group: impl Into<BitGroup<u16>>) {
        let group: u16 = group.into().into();
        let value = self.get(group);
        let toggled = (!value) & (group >> group.trailing_zeros());
        self.set(group, toggled);
    }

    pub fn clear(&mut self, group: impl Into<BitGroup<u16>>) {
        let group: u16 = group.into().into();
        self.group = self.group & (!group);
    }
}

impl From<u16> for BitGroup<u16> {
    fn from(value: u16) -> Self {
        Self { group: value }
    }
}

impl From<BitGroup<u16>> for u16 {
    fn from(value: BitGroup<u16>) -> Self {
        value.group
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bv() {
        assert_eq!(bv(0b0000_0000, 0), 0);
        assert_eq!(bv(0b0000_0001, 0), 1);
        assert_eq!(bv(0b0001_0000, 4), 1);
        assert_eq!(bv(0b1110_1111, 4), 0);
    }

    #[test]
    fn test_bv_16() {
        assert_eq!(bv_16(0b0000_0000_0000_0000, 7), 0);
        assert_eq!(bv_16(0b0000_0000_1000_0000, 7), 1);
        assert_eq!(bv_16(0b1111_1111_0111_1111, 7), 0);
        assert_eq!(bv_16(0b1111_1111_1111_1111, 7), 1);
    }

    #[test]
    fn test_bvs() {
        assert_eq!(bvs_8(0b0000_0000, 1, 0), 0);
        assert_eq!(bvs_8(0b0000_0001, 1, 0), 1);
        assert_eq!(bvs_8(0b0000_0011, 1, 0), 3);
        assert_eq!(bvs_8(0b0001_0000, 4, 0), 16);
        assert_eq!(bvs_8(0b1110_1111, 4, 4), 0);
    }

    #[test]
    fn test_set_bit() {
        assert_eq!(set_bit(0b0000_0000, 0), 0b0000_0001);
        assert_eq!(set_bit(0b0000_0001, 7), 0b1000_0001);
        assert_eq!(set_bit(0b0000_0011, 5), 0b0010_0011);
    }

    #[test]
    fn test_clear_bit() {
        assert_eq!(clear_bit(0b0000_0001, 0), 0b0000_0000);
        assert_eq!(clear_bit(0b1001_0001, 7), 0b0001_0001);
        assert_eq!(clear_bit(0b0010_0011, 5), 0b0000_0011);
    }

    #[test]
    fn test_bit_group() {
        let mut g = BitGroup::new(0b1011_1010);

        assert_eq!(g.get(0b1111_1111), g.group);
        assert_eq!(g.get(0b0000_1111), 0b1010);
        assert_eq!(g.get(0b1111_0000), 0b1011);

        g.clear(0b1111_0000);
        assert_eq!(g.get(0b0000_1111), 0b1010);
        assert_eq!(g.get(0b1111_0000), 0b0000);

        g.set(0b1111_0000, 5);
        assert_eq!(g.get(0b1111_1111), 0b0101_1010);
        assert_eq!(g.get(0b0000_1111), 0b0000_1010);
        assert_eq!(g.get(0b1111_0000), 5);
    }

    #[test]
    fn test_bit_group_overflowing_add() {
        let mut g = BitGroup::new(0b1011_1010);
        let group = 0b1111_0000;

        assert_eq!(g.get(0b1111_1111), 0b1011_1010);

        let overflow = g.overflowing_add(group, 4);
        assert!(!overflow);
        assert_eq!(g.get(0b1111_1111), 0b1111_1010);

        let overflow = g.overflowing_add(group, 2);
        assert!(overflow);
        assert_eq!(g.get(0b1111_1111), 0b0001_1010);
    }

    #[test]
    fn test_bit_group_toggle() {
        let mut g = BitGroup::new(0b1011_1010);

        g.toggle(0b1111_0000);
        assert_eq!(g.get(0b1111_1111), 0b0100_1010);

        g.toggle(0b0010_0000);
        assert_eq!(g.get(0b1111_1111), 0b0110_1010);

        g.toggle(0b1111_1111);
        assert_eq!(g.get(0b1111_1111), 0b1001_0101);

        let mut g = BitGroup::new(0b0001_0000);
        let flag = 0b0001_0000;

        g.toggle(flag);
        assert_eq!(g.get(flag), 0);
        assert_eq!(g.get(0xFFFF), 0);

        g.toggle(flag);
        assert_eq!(g.get(flag), 1);
        assert_eq!(g.get(0xFFFF), flag);
    }
}
