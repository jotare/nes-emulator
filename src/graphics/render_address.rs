use crate::utils::BitGroup;

/// [`RenderAddress`] represents the loopy registers `v` and `t` (from NES
/// wiki), a VRAM address maintained internally by the PPU while rendering.
///
/// It's a 15-bit address used for both reading and writing PPU memory through
/// PPUDATA ($2007) register
#[derive(Copy, Clone)]
pub struct RenderAddress {
    value: BitGroup<u16>,
}

impl RenderAddress {
    pub const FINE_Y_SCROLL: u16 = 0b0111_0000_0000_0000;
    pub const NAMETABLES_SELECT: u16 = 0b0000_1100_0000_0000;
    pub const VERTICAL_NAMETABLE: u16 = 0b0000_1000_0000_0000;
    pub const HORIZONTAL_NAMETABLE: u16 = 0b0000_0100_0000_0000;
    pub const COARSE_Y_SCROLL: u16 = 0b0000_0011_1110_0000;
    pub const COARSE_X_SCROLL: u16 = 0b0000_0000_0001_1111;

    pub fn value(&self) -> u16 {
        self.value.into()
    }

    pub fn get(&self, group: u16) -> u16 {
        self.value.get(group)
    }

    pub fn set(&mut self, group: u16, value: u8) {
        self.value.set(group, value.into());
    }

    pub fn increment_x(&mut self) {
        if self.get(Self::COARSE_X_SCROLL) == 31 {
            self.set(Self::COARSE_X_SCROLL, 0);
            self.value.toggle(Self::HORIZONTAL_NAMETABLE);
        } else {
            self.value.overflowing_add(Self::COARSE_X_SCROLL, 1);
        }
    }

    pub fn increment_y(&mut self) {
        let fine_y = self.get(Self::FINE_Y_SCROLL);
        if fine_y != 7 {
            self.value.overflowing_add(Self::FINE_Y_SCROLL, 1);
        } else {
            self.set(Self::FINE_Y_SCROLL, 0);

            let coarse_y = self.get(Self::COARSE_Y_SCROLL);
            if coarse_y == 29 {
                self.set(Self::COARSE_Y_SCROLL, 0);
                self.value.toggle(Self::VERTICAL_NAMETABLE);
            } else if coarse_y == 31 {
                self.set(Self::COARSE_Y_SCROLL, 0);
            } else {
                self.value.overflowing_add(Self::COARSE_Y_SCROLL, 1);
            }
        }
    }
}

impl From<RenderAddress> for u16 {
    fn from(value: RenderAddress) -> Self {
        value.value.into()
    }
}

impl From<u16> for RenderAddress {
    fn from(value: u16) -> Self {
        Self {
            value: BitGroup::new(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value() {
        assert_eq!(RenderAddress::from(0).value(), 0);
        assert_eq!(RenderAddress::from(0xF1A2).value(), 0xF1A2);
    }

    mod test_get_and_set {
        use super::*;

        #[test]
        fn test_fine_y_scroll() {
            // Fine Y scroll
            let mut addr = RenderAddress::from(0);
            addr.set(RenderAddress::FINE_Y_SCROLL, 0b101);
            assert_eq!(addr.get(RenderAddress::FINE_Y_SCROLL), 0b101);
            assert_eq!(addr.value(), 0b0101_0000_0000_0000);
        }

        #[test]
        fn test_nametables_select() {
            // Nametables select
            let mut addr = RenderAddress::from(0);
            addr.set(RenderAddress::NAMETABLES_SELECT, 0b11);
            assert_eq!(addr.get(RenderAddress::NAMETABLES_SELECT), 0b11);
            assert_eq!(addr.value(), 0b0000_1100_0000_0000);

            let mut addr = RenderAddress::from(0);
            addr.set(RenderAddress::VERTICAL_NAMETABLE, 1);
            assert_eq!(addr.get(RenderAddress::VERTICAL_NAMETABLE), 1);
            assert_eq!(addr.value(), 0b0000_1000_0000_0000);

            let mut addr = RenderAddress::from(0);
            addr.set(RenderAddress::HORIZONTAL_NAMETABLE, 1);
            assert_eq!(addr.get(RenderAddress::HORIZONTAL_NAMETABLE), 1);
            assert_eq!(addr.value(), 0b0000_0100_0000_0000);
        }

        #[test]
        fn test_coarse_y() {
            // Coarse Y scroll
            let mut addr = RenderAddress::from(0);
            addr.set(RenderAddress::COARSE_Y_SCROLL, 0b10101);
            assert_eq!(addr.get(RenderAddress::COARSE_Y_SCROLL), 0b10101);
            assert_eq!(addr.value(), 0b0000_0010_1010_0000);
        }

        #[test]
        fn test_coarse_x() {
            // Coarse X scroll
            let mut addr = RenderAddress::from(0);
            addr.set(RenderAddress::COARSE_X_SCROLL, 0b10101);
            assert_eq!(addr.get(RenderAddress::COARSE_X_SCROLL), 0b10101);
            assert_eq!(addr.value(), 0b0000_0000_0001_0101);
        }
    }

    #[test]
    fn test_increment_x() {
        // See https://www.nesdev.org/wiki/PPU_scrolling#Coarse_X_increment for
        // further reference
        let mut addr = RenderAddress::from(0);

        addr.set(RenderAddress::COARSE_X_SCROLL, 0);
        addr.increment_x();
        assert_eq!(addr.get(RenderAddress::COARSE_X_SCROLL), 1);
        assert_eq!(addr.get(RenderAddress::HORIZONTAL_NAMETABLE), 0);

        addr.set(RenderAddress::COARSE_X_SCROLL, 31);
        addr.increment_x();
        assert_eq!(addr.get(RenderAddress::COARSE_X_SCROLL), 0);
        assert_eq!(addr.get(RenderAddress::HORIZONTAL_NAMETABLE), 1);

        addr.set(RenderAddress::COARSE_X_SCROLL, 31);
        addr.increment_x();
        assert_eq!(addr.get(RenderAddress::COARSE_X_SCROLL), 0);
        assert_eq!(addr.get(RenderAddress::HORIZONTAL_NAMETABLE), 0);
    }

    #[test]
    fn test_increment_y() {
        // See https://www.nesdev.org/wiki/PPU_scrolling#Y_increment for further
        // reference
        let mut addr = RenderAddress::from(0);

        // Normal fine Y increment without overflow
        addr.set(RenderAddress::FINE_Y_SCROLL, 0);
        addr.set(RenderAddress::COARSE_Y_SCROLL, 0);
        addr.increment_y();
        assert_eq!(addr.get(RenderAddress::FINE_Y_SCROLL), 1);
        assert_eq!(addr.get(RenderAddress::COARSE_Y_SCROLL), 0);
        assert_eq!(addr.get(RenderAddress::VERTICAL_NAMETABLE), 0);

        // Fine Y overflows to coarse Y
        addr.set(RenderAddress::FINE_Y_SCROLL, 7);
        addr.increment_y();
        assert_eq!(addr.get(RenderAddress::FINE_Y_SCROLL), 0);
        assert_eq!(addr.get(RenderAddress::COARSE_Y_SCROLL), 1);
        assert_eq!(addr.get(RenderAddress::VERTICAL_NAMETABLE), 0);

        // Coarse Y overflows to nametable toggle
        addr.set(RenderAddress::FINE_Y_SCROLL, 7);
        addr.set(RenderAddress::COARSE_Y_SCROLL, 29);
        addr.increment_y();
        assert_eq!(addr.get(RenderAddress::FINE_Y_SCROLL), 0);
        assert_eq!(addr.get(RenderAddress::COARSE_Y_SCROLL), 0);
        assert_eq!(addr.get(RenderAddress::VERTICAL_NAMETABLE), 1);

        addr.set(RenderAddress::FINE_Y_SCROLL, 7);
        addr.set(RenderAddress::COARSE_Y_SCROLL, 29);
        addr.increment_y();
        assert_eq!(addr.get(RenderAddress::FINE_Y_SCROLL), 0);
        assert_eq!(addr.get(RenderAddress::COARSE_Y_SCROLL), 0);
        assert_eq!(addr.get(RenderAddress::VERTICAL_NAMETABLE), 0);

        // Coarse Y out of bounds (> 29) increment and wrap around but don't
        // switch nametable
        addr.set(RenderAddress::FINE_Y_SCROLL, 7);
        addr.set(RenderAddress::COARSE_Y_SCROLL, 30);
        addr.increment_y();
        assert_eq!(addr.get(RenderAddress::FINE_Y_SCROLL), 0);
        assert_eq!(addr.get(RenderAddress::COARSE_Y_SCROLL), 31);
        assert_eq!(addr.get(RenderAddress::VERTICAL_NAMETABLE), 0);

        addr.set(RenderAddress::FINE_Y_SCROLL, 7);
        addr.increment_y();
        assert_eq!(addr.get(RenderAddress::COARSE_Y_SCROLL), 0);
        assert_eq!(addr.get(RenderAddress::VERTICAL_NAMETABLE), 0);
    }
}
