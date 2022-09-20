use crate::sabi_types::{NulStr, NulStrError};

use abi_stable_shared::test_utils::must_panic;

use std::cmp::{Ord, Ordering, PartialOrd};

fn from_str_with_constructor(func: fn(&str) -> NulStr<'_>) {
    let pairs = [("fob\0", "fob"), ("fo\0", "fo"), ("f\0", "f"), ("\0", "")];
    for (strwn, str) in pairs.iter().copied() {
        dbg!(strwn);
        let this = func(strwn);
        assert_eq!(this.as_ptr(), strwn.as_ptr());
        assert_eq!(this.to_str(), str);
        assert_eq!(this.to_rstr(), str);
        assert_eq!(this.to_str_with_nul(), strwn);
        assert_eq!(this.to_rstr_with_nul(), strwn);
        #[cfg(feature = "rust_1_64")]
        {
            assert_eq!(this.const_to_str(), str);
            assert_eq!(this.const_to_str_with_nul(), strwn);
        }
    }

    for &strwn in &["foo\0", "foo\0bar\0"] {
        dbg!(strwn);
        let this = func(strwn);
        assert_eq!(this.as_ptr(), strwn.as_ptr());
        assert_eq!(this.to_str(), "foo");
        assert_eq!(this.to_rstr(), "foo");
        assert_eq!(this.to_str_with_nul(), "foo\0");
        assert_eq!(this.to_rstr_with_nul(), "foo\0");

        #[cfg(feature = "rust_1_64")]
        {
            assert_eq!(this.const_to_str(), "foo");
            assert_eq!(this.const_to_str_with_nul(), "foo\0");
        }
    }
}

const NS1: NulStr<'_> = NulStr::from_str("hello\0");
const NS2: NulStr<'_> = NulStr::from_str("world\0foo\0");

#[test]
fn nulstr_from_str_tests() {
    must_panic(|| NulStr::from_str("foo\0bar")).unwrap();
    must_panic(|| NulStr::from_str("foo")).unwrap();
    must_panic(|| NulStr::from_str("")).unwrap();

    assert_eq!(NS1, "hello");
    assert_eq!(NS2, "world");

    from_str_with_constructor(|s| NulStr::from_str(s));
    from_str_with_constructor(|s| unsafe { NulStr::from_ptr(s.as_ptr()) });
}

#[test]
#[cfg(feature = "rust_1_64")]
fn const_to_str_tests() {
    macro_rules! assert_cs {
        ($lhs:expr, $($rem:tt)*) => ({
            const __S: &str = $lhs;
            assert_eq!(__S, $($rem)*);
        });
    }

    assert_cs!(NS1.const_to_str(), "hello");
    assert_cs!(NS1.const_to_str_with_nul(), "hello\0");

    assert_cs!(NS2.const_to_str(), "world");
    assert_cs!(NS2.const_to_str_with_nul(), "world\0");
}

#[test]
fn nulstr_try_from_str_tests() {
    let pairs = [("foobar\0", "foobar"), ("f\0", "f"), ("\0", "")];
    for (strwn, str) in pairs.iter().copied() {
        let nuls = [
            NulStr::try_from_str(strwn).unwrap(),
            NulStr::__try_from_str_unwrapping(strwn),
        ];
        for &nul in &nuls {
            assert_eq!(nul.to_str(), str);
            assert_eq!(nul.to_str_with_nul(), strwn);
        }
    }

    let err_pairs = vec![
        ("foo\0bar\0", NulStrError::InnerNul { pos: 3 }),
        ("fo\0\0", NulStrError::InnerNul { pos: 2 }),
        ("f\0\0", NulStrError::InnerNul { pos: 1 }),
        ("\0\0", NulStrError::InnerNul { pos: 0 }),
        ("foobar", NulStrError::NoNulTerminator),
        ("", NulStrError::NoNulTerminator),
    ];

    for (strwn, err) in err_pairs {
        dbg!(strwn);
        assert_eq!(NulStr::try_from_str(strwn), Err(err));

        must_panic(|| NulStr::__try_from_str_unwrapping(strwn)).unwrap();
    }
}

#[test]
fn nulstr_cmp_test() {
    let strings = [
        "\0", "f\0", "fo\0", "foc\0", "foca\0", "focal\0", "foo\0", "fooo\0", "foooo\0", "bar\0",
        "barr\0", "barrr\0", "baz\0", "bazz\0", "bazzz\0",
    ]
    .iter()
    .map(|&x| (x, NulStr::from_str(x)))
    .collect::<Vec<(&str, NulStr<'_>)>>();

    for &(leftwn, left) in &strings {
        assert_eq!(left, left);

        assert_eq!(left.cmp(&left), Ordering::Equal);
        assert_eq!(left.partial_cmp(&left), Some(Ordering::Equal));

        {
            let x = leftwn.trim_end_matches('\0');

            assert_eq!(left, x);
            assert_eq!(left, *x);
            assert_eq!(x, left);
            assert_eq!(*x, left);
            assert_eq!(left.to_str_with_nul(), leftwn);
        }

        // making sure that NulStr doesn't just do pointer equality
        let left_copywn = leftwn.to_string();
        let left_copy = NulStr::from_str(&left_copywn);
        assert_eq!(left, left_copy);
        assert_eq!(left_copy, left);

        assert_eq!(left.cmp(&left_copy), Ordering::Equal);
        assert_eq!(left_copy.cmp(&left), Ordering::Equal);
        assert_eq!(left.partial_cmp(&left_copy), Some(Ordering::Equal));
        assert_eq!(left_copy.partial_cmp(&left), Some(Ordering::Equal));

        for &(_, right) in &strings {
            assert_eq!(
                left == right,
                left.to_str() == right.to_str(),
                "\n left: {:?}\nright: {:?}\n",
                left,
                right,
            );

            assert_eq!(left.cmp(&right), left.to_str().cmp(right.to_str()));
            assert_eq!(
                left.partial_cmp(&right),
                left.to_str().partial_cmp(right.to_str())
            );
        }
    }
}
