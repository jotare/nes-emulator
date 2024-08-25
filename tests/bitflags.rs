use bitflags::bitflags;

bitflags! {
    struct TestFlags: u8 {
        const A = 0b0000_0001;
        const B = 0b0000_1110;
        const C = 0b0111_0000;
        const D = 0b1000_0000;

        const ALL = Self::A.bits | Self::B.bits | Self::C.bits | Self::D.bits;
    }
}

#[test]
fn test_bitflags() {
    let mut flags = TestFlags::empty();
    assert!(!flags.contains(TestFlags::B));

    flags.insert(TestFlags::B);
    assert!(flags.contains(TestFlags::B));
}
