use crate::utils::BitGroup;

/// A pattern table address points to a specific pattern table section (left or
/// right), a tile number and a row inside the tile. The column to use is
/// decided by users of this abstraction. Scrolling behavior is implemented
/// manipulating which column of a tile should be rendered
#[derive(Clone, Copy)]
pub struct PatternTableAddress {
    value: BitGroup<u16>,
}

impl PatternTableAddress {
    /// Select with pattern table to use (left or right one)
    pub const PATTERN_TABLE: u16 = 0b0001_0000_0000_0000;

    pub const TILE_NUMBER: u16 = 0b0000_1111_1111_0000;

    pub const BIT_PLANE: u16 = 0b0000_0000_0000_1000;

    /// Row number inside a tile
    pub const FINE_Y_OFFSET: u16 = 0b0000_0000_0000_0111;

    pub fn new(pattern_table: u8) -> Self {
        let mut value = BitGroup::new(0);
        value.set(Self::PATTERN_TABLE, pattern_table.into());
        Self { value }
    }

    pub fn set(&mut self, group: u16, value: u8) {
        self.value.set(group, value.into());
    }
}

impl From<PatternTableAddress> for u16 {
    fn from(value: PatternTableAddress) -> Self {
        value.value.into()
    }
}
