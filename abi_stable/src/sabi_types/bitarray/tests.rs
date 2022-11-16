use super::{bool_to_enum, enum_to_bool, BitArray64, BooleanEnum};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Bool {
    ///
    False = 0,
    ///
    True = 1,
}

unsafe impl BooleanEnum for Bool {
    const FALSE: Self = Self::False;
    const TRUE: Self = Self::True;
}

#[test]
fn with_count() {
    for count in 0..=64 {
        let bits = BitArray64::<Bool>::with_count(count);

        for i in 0..64 {
            assert_eq!(
                bits.at(i) == Bool::True,
                i < count,
                "count={} bits={:?}",
                count,
                bits.bits()
            );
        }
    }
}

#[test]
fn set_bits() {
    let mut bits = BitArray64::with_count(8);
    assert_eq!(0b_1111_1111, bits.bits());

    {
        let mut bits = bits;

        bits = bits.set(0, Bool::False);
        assert_eq!(0b_1111_1110, bits.bits());

        bits = bits.set(2, Bool::False);
        assert_eq!(0b_1111_1010, bits.bits());

        bits = bits.set(1, Bool::False);
        assert_eq!(0b_1111_1000, bits.bits());
    }

    bits = bits.set(3, Bool::False);
    assert_eq!(0b_1111_0111, bits.bits());

    bits = bits.set(5, Bool::False);
    assert_eq!(0b_1101_0111, bits.bits());

    bits = bits.set(6, Bool::False);
    assert_eq!(0b_1001_0111, bits.bits());

    bits = bits.set(10, Bool::True);
    assert_eq!(0b_0100_1001_0111, bits.bits());

    bits = bits.set(63, Bool::True);
    assert_eq!((1 << 63) | 0b_0100_1001_0111, bits.bits());
}

#[test]
fn empty() {
    let bits = BitArray64::empty();

    for i in 0..64 {
        assert!(
            matches!(bits.at(i), Bool::False),
            "i={} bits={:b}",
            i,
            bits.bits()
        );
    }
}

#[test]
fn iter_test() {
    let iter = BitArray64::with_count(8)
        .set(1, Bool::False)
        .set(3, Bool::False)
        .iter()
        .take(10)
        .map(enum_to_bool);

    let expected = vec![
        true, false, true, false, true, true, true, true, false, false,
    ];
    let expected_rev = expected.iter().cloned().rev().collect::<Vec<bool>>();

    assert_eq!(iter.clone().collect::<Vec<bool>>(), expected);

    assert_eq!(iter.rev().collect::<Vec<bool>>(), expected_rev);
}

#[test]
fn enum_roundtrip() {
    assert_eq!(bool_to_enum::<Bool>(false), Bool::False);
    assert_eq!(bool_to_enum::<Bool>(true), Bool::True);

    assert_eq!(enum_to_bool(Bool::False), false);
    assert_eq!(enum_to_bool(Bool::True), true);
}

#[test]
fn bool_roundtrip() {
    assert_eq!(bool_to_enum::<bool>(false), false);
    assert_eq!(bool_to_enum::<bool>(true), true);

    assert_eq!(enum_to_bool(false), false);
    assert_eq!(enum_to_bool(true), true);
}
