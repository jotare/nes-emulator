/// Return the bit value for `value` at bit position `bit`
pub fn bv(value: u8, bit: u8) -> u8 {
    (value >> bit) & 1
}

/// Return the value from `value` between bit positions `major_bit` and
/// `minor_bit`
pub fn bvs(value: u8, major_bit: u8, minor_bit: u8) -> u8 {
    (value >> minor_bit) & ((1 << (major_bit - minor_bit + 1)) - 1)
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
        assert_eq!(bvs(0b0000_0000, 1, 0), 0);
        assert_eq!(bvs(0b0000_0001, 1, 0), 1);
        assert_eq!(bvs(0b0000_0011, 1, 0), 3);
        assert_eq!(bvs(0b0001_0000, 4, 0), 16);
        assert_eq!(bvs(0b1110_1111, 4, 4), 0);
    }
}
