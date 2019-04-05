use super::*;


#[allow(unused_imports)]
use core_extensions::prelude::*;


static TEST_STR: &str =
    "hello_world.cáscara.ñ.🎊🍕👏😊😀😄😉😉😛😮🙁🙂💔👻😎.";

#[test]
fn from_utf8() {
    let rstr = RString::from_utf8(TEST_STR.as_bytes().to_vec()).unwrap();
    assert_eq!(&*rstr, TEST_STR);
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
fn push_str() {
    let mut rstr = RString::new();

    let iter = TEST_STR.split_while(|c| c == '.').map(|v| v.str);

    for s in iter {
        let end = s.offset_inside_of(TEST_STR) + s.len();
        rstr.push_str(s);
        assert_eq!(&*rstr, &TEST_STR[..end]);
    }
}

#[test]
fn drain() {
    let rstr = TEST_STR.into_(RString::T);
    assert_eq!(&*rstr, TEST_STR);
    assert_eq!(&*rstr.into_iter().collect::<RString>(), TEST_STR);
}
