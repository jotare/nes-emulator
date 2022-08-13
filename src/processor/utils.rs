/// Return the bit value for `value` at bit position `bit`
pub fn bv(value: u8, bit: u8) -> u8 {
    value & (1 << bit)
}
