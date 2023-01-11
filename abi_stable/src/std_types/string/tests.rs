use super::*;

use crate::test_utils::must_panic;

#[allow(unused_imports)]
use core_extensions::{SelfOps, SliceExt};

static TEST_STR: &str = "hello_world.cáscara.ñ.🎊🍕👏😊😀😄😉😉😛😮🙁🙂💔👻😎.";

#[test]
fn from_to_string() {
    let orig = "hello, world!";
    let orig_owned = orig.to_string();
    let orig_cap = orig_owned.capacity();

    // Converted to an RString
    let copy = orig.into_::<RString>();

    assert_eq!(orig, &copy[..]);

    assert_eq!(copy.capacity(), orig_cap);

    // Converted back to the original
    let orig_back = copy.into_::<String>();

    assert_eq!(&orig_back[..], orig);
    assert_eq!(orig_back.capacity(), orig_cap);
}

#[test]
fn from_utf8() {
    let rstr = RString::from_utf8(TEST_STR.as_bytes().to_vec()).unwrap();
    assert_eq!(&*rstr, TEST_STR);
}

#[cfg(feature = "rust_1_64")]
#[test]
fn const_as_str() {
    const RS: &RString = &RString::new();
    const S: &str = RS.as_str();

    assert_eq!(S, "");
}

#[test]
fn push() {
    let mut rstr = RString::new();

    assert_eq!(&*rstr, "");

    for (i, c) in TEST_STR.char_indices() {
        let end = i + c.len_utf8();
        rstr.push(c);
        assert_eq!(&rstr[..end], &TEST_STR[..end]);
    }
}

#[test]
fn insert_str() {
    // '💔' is 4 bytes long
    let test_str = "💔love💔is💔";
    let rstr = test_str.into_::<RString>();

    {
        let mut rstr = rstr.clone();
        must_panic(|| rstr.insert_str(1, "foo")).unwrap();
        must_panic(|| rstr.insert_str(2, "foo")).unwrap();
        must_panic(|| rstr.insert_str(3, "foo")).unwrap();

        must_panic(|| rstr.insert_str(9, "foo")).unwrap();
        must_panic(|| rstr.insert_str(10, "foo")).unwrap();
        must_panic(|| rstr.insert_str(11, "foo")).unwrap();

        must_panic(|| rstr.insert_str(15, "foo")).unwrap();
        must_panic(|| rstr.insert_str(16, "foo")).unwrap();
        must_panic(|| rstr.insert_str(17, "foo")).unwrap();
    }
    {
        // insert at the end
        let mut rstr = rstr.clone();
        rstr.insert_str(18, "💔love💔is💔foo");
    }

    {
        // insert at the start
        let mut rstr = rstr.clone();
        rstr.insert_str(0, "foo💔love💔is💔");
    }

    {
        // insert in the middle
        let mut rstr = rstr.clone();
        rstr.insert_str(12, "💔love💔foois💔");
    }
    {
        // insert in the middle 2
        let mut rstr = rstr;
        rstr.insert_str(14, "💔love💔isfoo💔");
    }
}

#[test]
fn remove() {
    // '💔' is 4 bytes long
    let test_str = "💔love💔is💔💔love💔is💔";
    let test_str_nohearts = test_str.chars().filter(|&c| c != '💔').collect::<String>();
    let mut rstr = test_str.into_::<RString>();

    must_panic(|| rstr.remove(1)).unwrap();
    must_panic(|| rstr.remove(9)).unwrap();
    must_panic(|| rstr.remove(10)).unwrap();
    must_panic(|| rstr.remove(11)).unwrap();
    must_panic(|| rstr.remove(15)).unwrap();
    must_panic(|| rstr.remove(16)).unwrap();
    must_panic(|| rstr.remove(17)).unwrap();
    must_panic(|| rstr.remove(test_str.len() - 3)).unwrap();
    must_panic(|| rstr.remove(test_str.len() - 2)).unwrap();
    must_panic(|| rstr.remove(test_str.len() - 1)).unwrap();
    must_panic(|| rstr.remove(test_str.len())).unwrap();

    assert_eq!(rstr.remove(32), '💔');
    assert_eq!(rstr.remove(26), '💔');
    assert_eq!(rstr.remove(18), '💔');
    assert_eq!(rstr.remove(14), '💔');
    assert_eq!(rstr.remove(8), '💔');
    assert_eq!(rstr.remove(0), '💔');

    assert_eq!(&*rstr, &*test_str_nohearts);

    {
        // Removing from the end
        let mut rstr = rstr.clone();

        for i in (0..rstr.len()).rev() {
            assert_eq!(
                rstr.remove(i),
                test_str_nohearts[i..].chars().next().unwrap()
            );
        }
    }

    {
        // Removing from the start
        let mut rstr = rstr.clone();

        for i in 0..rstr.len() {
            assert_eq!(
                rstr.remove(0),
                test_str_nohearts[i..].chars().next().unwrap()
            );
        }
    }
}

#[test]
fn push_str() {
    let mut rstr = RString::new();

    let iter = TEST_STR.split_while(|c| c == '.').map(|v| v.str);

    for s in iter {
        let end = TEST_STR.offset_of_slice(s) + s.len();
        rstr.push_str(s);
        assert_eq!(&*rstr, &TEST_STR[..end]);
    }
}

#[test]
fn retain() {
    let retain_test_str = "abcd💔01💔efg💔23";
    let rstr = retain_test_str.into_::<RString>();

    {
        let mut rstr = rstr.clone();
        rstr.retain(|c| c.is_alphabetic());
        assert_eq!(&*rstr, "abcdefg");
    }
    {
        let mut rstr = rstr.clone();
        rstr.retain(|c| !c.is_alphabetic());
        assert_eq!(&*rstr, "💔01💔💔23");
    }
    {
        let mut rstr = rstr.clone();
        rstr.retain(|c| c.is_numeric());
        assert_eq!(&*rstr, "0123");
    }
    {
        let mut rstr = rstr.clone();
        rstr.retain(|c| c == '💔');
        assert_eq!(&*rstr, "💔💔💔");
    }
    {
        let mut rstr = rstr.clone();
        rstr.retain(|c| c != '💔');
        assert_eq!(&*rstr, "abcd01efg23");
    }
    {
        let mut i = 0;
        let closure = move |_| {
            let cond = i % 2 == 0;
            i += 1;
            cond
        };

        let mut rstr = rstr;
        rstr.retain(closure);

        let mut string = retain_test_str.to_string();
        string.retain(closure);

        assert_eq!(&*rstr, &*string);
    }
    {
        // Copied from:
        // https://github.com/rust-lang/rust/blob/48c4afbf9c29880dd946067d1c9aee1e7f75834a/library/alloc/tests/string.rs#L383
        let mut s = RString::from("0è0");
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut count = 0;
            s.retain(|_| {
                count += 1;
                match count {
                    1 => false,
                    2 => true,
                    _ => panic!(),
                }
            });
        }));
        assert!(std::str::from_utf8(s.as_bytes()).is_ok());
    }
}

#[test]
fn into_iter() {
    static TEST_STR: &str = "hello_world.cáscara.ñ.🎊🍕👏😊😀😄😉😉😛😮🙁🙂💔👻😎.";

    let rstr = TEST_STR.into_::<RString>();

    assert_eq!(&*rstr, TEST_STR);
    assert_eq!(&*rstr.clone().into_iter().collect::<String>(), TEST_STR);

    let mut iter = rstr.into_iter();

    fn compare_str_iter(expecting: &str, iter: &mut IntoIter) {
        assert_eq!(&iter.as_str()[..expecting.len()], expecting);
        assert_eq!(
            &*iter.take(expecting.chars().count()).collect::<String>(),
            expecting,
        );
    }

    compare_str_iter("hello_world", &mut iter);
    assert_eq!(iter.next(), Some('.'));

    compare_str_iter("cáscara", &mut iter);
    assert_eq!(iter.next(), Some('.'));

    compare_str_iter("ñ", &mut iter);
    assert_eq!(iter.next(), Some('.'));

    compare_str_iter("🎊🍕👏😊😀😄😉😉😛😮🙁🙂💔👻", &mut iter);
    assert_eq!(iter.next(), Some('😎'));
    assert_eq!(iter.next(), Some('.'));

    assert_eq!(iter.next(), None);
}

#[test]
fn drain() {
    let mut rstr = TEST_STR.into_::<RString>();
    let rstr_cap = rstr.capacity();

    // Using this to test that trying to drain in the middle of a character does not work
    let broken_heart_pos = TEST_STR.char_indices().find(|(_, c)| '💔' == *c).unwrap().0;

    must_panic(|| rstr.drain(..TEST_STR.len() + 1)).unwrap();
    must_panic(|| rstr.drain(..broken_heart_pos + 1)).unwrap();
    must_panic(|| rstr.drain(broken_heart_pos..broken_heart_pos + 1)).unwrap();
    must_panic(|| rstr.drain(broken_heart_pos + 1..broken_heart_pos + 2)).unwrap();
    must_panic(|| rstr.drain(broken_heart_pos + 1..broken_heart_pos + 3)).unwrap();
    must_panic(|| rstr.drain(broken_heart_pos + 1..)).unwrap();

    assert_eq!(&rstr.drain(11..).collect::<String>(), &TEST_STR[11..]);
    assert_eq!(&rstr[..], "hello_world");
    assert_eq!(rstr.len(), 11);
    assert_eq!(rstr.capacity(), rstr_cap);

    rstr.drain(4..8);
    assert_eq!(&rstr[..], "hellrld");
    assert_eq!(rstr.len(), 7);
    assert_eq!(rstr.capacity(), rstr_cap);

    rstr.drain(..6);
    assert_eq!(&rstr[..], "d");
    assert_eq!(rstr.len(), 1);
    assert_eq!(rstr.capacity(), rstr_cap);

    rstr.drain(..1);
    assert_eq!(&rstr[..], "");
    assert_eq!(rstr.len(), 0);
    assert_eq!(rstr.capacity(), rstr_cap);
}
