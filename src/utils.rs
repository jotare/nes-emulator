/// Return the bit value for `value` at bit position `bit`
pub fn bv(value: u8, bit: u8) -> u8 {
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
}
