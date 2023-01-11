use super::*;

#[test]
fn cmp_and_variance() {
    fn eq_rcow<'a, 'b, T, U>(left: &RCowVal<'a, T>, right: &RCowVal<'b, U>) -> bool
    where
        T: Clone + PartialEq<U>,
        U: Clone,
    {
        RCow::eq(left, right)
    }

    fn cmp_rcow<'a, 'b, T, U>(left: &RCowVal<'a, T>, right: &RCowVal<'b, U>) -> Ordering
    where
        T: Clone + PartialOrd<U>,
        U: Clone,
    {
        RCow::partial_cmp(left, right).unwrap()
    }

    fn eq_rcow_str<'a, 'b>(left: &RCowStr<'a>, right: &RCowStr<'b>) -> bool {
        RCow::eq(left, right)
    }

    fn cmp_rcow_str<'a, 'b>(left: &RCowStr<'a>, right: &RCowStr<'b>) -> Ordering {
        RCow::cmp(left, right)
    }

    fn eq_rcow_slice<'a, 'b, T, U>(left: &RCowSlice<'a, T>, right: &RCowSlice<'b, U>) -> bool
    where
        T: Clone + PartialEq<U>,
        U: Clone,
    {
        RCow::eq(left, right)
    }

    // std doesn't have a `[T]: PartialCmp<[U]>` blanket impl.
    fn cmp_rcow_slice<'a, 'b, T>(left: &RCowSlice<'a, T>, right: &RCowSlice<'b, T>) -> Ordering
    where
        T: Clone + PartialOrd,
    {
        RCow::partial_cmp(left, right).unwrap()
    }

    {
        let bb = 3u8;

        let left = RCow::Owned(2u8);
        let middle = RCow::Borrowed(&bb);
        let right = RCow::Owned(5u8);
        assert!(eq_rcow(&left, &left));
        assert!(!eq_rcow(&left, &middle));
        assert!(!eq_rcow(&right, &middle));
        assert_eq!(cmp_rcow(&left, &left), Ordering::Equal);
        assert_eq!(cmp_rcow(&left, &middle), Ordering::Less);
        assert_eq!(cmp_rcow(&right, &middle), Ordering::Greater);
    }
    // polymorphic comparison
    {
        let left = RCowVal::Owned(rvec![3]);
        let right = RCowVal::Owned(&[3][..]);
        assert!(eq_rcow(&left, &right));
        assert_eq!(cmp_rcow(&left, &right), Ordering::Equal);
    }

    {
        let bb = "foo".to_string();
        let left = RCowStr::Borrowed(RStr::from_str("bar"));
        let middle = RCowStr::Borrowed(RStr::from_str(&bb));
        let right = RCowStr::Owned(RString::from("qux"));

        assert!(eq_rcow_str(&left, &left));
        assert!(!eq_rcow_str(&left, &middle));
        assert!(!eq_rcow_str(&right, &middle));
        assert_eq!(cmp_rcow_str(&left, &left), Ordering::Equal);
        assert_eq!(cmp_rcow_str(&left, &middle), Ordering::Less);
        assert_eq!(cmp_rcow_str(&right, &middle), Ordering::Greater);
    }

    {
        let aa = [13, 21, 34];
        let bb = aa.iter().collect::<RVec<&u8>>();
        let left = RCowSlice::Borrowed(RSlice::from_slice(&[&3u8, &5, &9]));
        let middle = RCowSlice::Owned(bb);
        let right = RCowSlice::Borrowed(RSlice::from_slice(&[&55u8, &88, &144]));

        assert!(eq_rcow_slice(&left, &left));
        assert!(!eq_rcow_slice(&left, &middle));
        assert!(!eq_rcow_slice(&right, &middle));
        assert_eq!(cmp_rcow_slice(&left, &left), Ordering::Equal);
        assert_eq!(cmp_rcow_slice(&left, &middle), Ordering::Less);
        assert_eq!(cmp_rcow_slice(&right, &middle), Ordering::Greater);
    }

    // polymorphic comparison
    {
        let left = &[vec![3]];
        let left = RCowSlice::Borrowed(RSlice::<Vec<u8>>::from_slice(left));
        let right = &[[3]];
        let right = RCowSlice::Borrowed(RSlice::<[u8; 1]>::from_slice(right));

        assert!(eq_rcow_slice(&left, &right));
    }
}

#[test]
fn rcow_from_str() {
    const RCSTR: &RCowStr<'_> = &RCow::from_str("bar");
    assert_eq!(RCSTR.as_str(), "bar");

    #[cfg(feature = "rust_1_64")]
    {
        const STR: &str = RCSTR.as_str();
        assert_eq!(STR, "bar");
    }
}

#[test]
fn rcow_from_slice() {
    const RCSLICE: &RCowSlice<'_, u8> = &RCow::from_slice(b"foo");
    assert_eq!(RCSLICE.as_slice(), b"foo");

    #[cfg(feature = "rust_1_64")]
    {
        const SLICE: &[u8] = RCSLICE.as_slice();
        assert_eq!(SLICE, b"foo");
    }
}

#[test]
fn rcow_from() {
    {
        const S: &str = "what the heck";
        let ref_owned: &String = &S.to_string();
        let ref_rowned: &RString = &S.to_string().into_c();
        assert_matches!(RCow::from(S), RCow::Borrowed(x @ RStr{..}) if x == S);
        assert_matches!(RCow::from(ref_owned), RCow::Borrowed(x @ RStr{..}) if x == S);
        assert_matches!(RCow::from(ref_rowned), RCow::Borrowed(x @ RStr{..}) if x == S);
        assert_matches!(
            RCow::from(Cow::from(S)),
            RCow::Borrowed(x @ RStr{..}) if x == S
        );
        assert_matches!(
            RCow::from(Cow::from(S.to_string())),
            RCow::Owned(ref x @ RString{..}) if x == S
        );
        assert_matches!(
            Cow::from(S).into_c(),
            RCow::Borrowed(x @ RStr{..}) if x == S
        );
        assert_matches!(
            Cow::from(S.to_string()).into_c(),
            RCow::Owned(ref x @ RString{..}) if x == S
        );
        assert_matches!(
            RCow::from(RString::from(S)),
            RCow::Owned(ref x @ RString{..}) if x == S
        );
        assert_eq!(RCow::from(Cow::from(S)), S);
        assert_eq!(RCow::from(S.to_string()), S);
        assert_eq!(RCow::from(RStr::from(S)), S);
        assert_eq!(RCow::from(RString::from(S)), S);
    }
    {
        const S: &[u8] = &[0, 1, 2, 3];
        let rref: RSlice<'_, u8> = S.into_c();
        let ref_owned: &Vec<u8> = &S.to_vec();
        let ref_rowned: &RVec<u8> = &S.to_vec().into_c();

        assert_matches!(RCow::from(S), RCow::Borrowed(x @ RSlice{..}) if x == S);
        assert_matches!(RCow::from(rref), RCow::Borrowed(x @ RSlice{..}) if x == S);
        assert_matches!(RCow::from(ref_owned), RCow::Borrowed(x @ RSlice{..}) if x == S);
        assert_matches!(RCow::from(ref_rowned), RCow::Borrowed(x @ RSlice{..}) if x == S);
        assert_matches!(
            RCow::from(Cow::from(S)),
            RCow::Borrowed(x @ RSlice{..}) if x == S
        );
        assert_matches!(
            RCow::from(Cow::from(S.to_vec())),
            RCow::Owned(ref x @ RVec{..}) if x == S
        );
        assert_matches!(
            Cow::from(S).into_c(),
            RCow::Borrowed(x @ RSlice{..}) if x == S
        );
        assert_matches!(
            Cow::from(S.to_vec()).into_c(),
            RCow::Owned(ref x @ RVec{..}) if x == S
        );
        assert_matches!(
            RCow::from(S.to_vec()),
            RCow::Owned(ref x @ RVec{..}) if x == S
        );
        assert_matches!(
            RCow::from(RVec::from(S)),
            RCow::Owned(ref x @ RVec{..}) if x == S
        );
    }
    {
        const S: &u32 = &1000u32;

        assert_eq!(*RCow::Borrowed(S), 1000);
        assert_eq!(*RCowVal::Owned(*S), 1000);

        assert_matches!(RCow::from(Cow::Borrowed(S)), RCow::Borrowed(&1000));
        assert_matches!(RCow::from(Cow::<u32>::Owned(*S)), RCow::Owned(1000));

        assert_matches!(Cow::Borrowed(S).into_c(), RCow::Borrowed(&1000));
        assert_matches!(Cow::<u32>::Owned(*S).into_c(), RCow::Owned(1000));
    }
}

#[test]
fn rcow_into() {
    {
        const S: &str = "what the heck";
        let bcow = || RCow::from(S);
        let ocow = || RCow::from(S.to_owned());

        assert_matches!(Cow::from(bcow()), Cow::Borrowed(S));
        assert_matches!(Cow::from(ocow()), Cow::Owned(ref x @ String{..}) if x == S);

        assert_matches!(bcow().into_rust(), Cow::Borrowed(S));
        assert_matches!(ocow().into_rust(), Cow::Owned(ref x @ String{..}) if x == S);
    }
    {
        const S: &[u8] = &[0, 1, 2, 3];
        let bcow = || RCow::from(S);
        let ocow = || RCow::from(S.to_owned());

        assert_matches!(Cow::from(bcow()), Cow::Borrowed(S));
        assert_matches!(Cow::from(ocow()), Cow::Owned(ref x @ Vec{..}) if x == S);

        assert_matches!(bcow().into_rust(), Cow::Borrowed(S));
        assert_matches!(ocow().into_rust(), Cow::Owned(ref x @ Vec{..}) if x == S);
    }
    {
        const S: u32 = 1234;
        let bcow = || RCowVal::Borrowed(&S);
        let ocow = || RCowVal::Owned(S);

        assert_matches!(Cow::from(bcow()), Cow::Borrowed(&S));
        assert_matches!(Cow::from(ocow()), Cow::Owned(S));

        assert_matches!(bcow().into_rust(), Cow::Borrowed(&S));
        assert_matches!(ocow().into_rust(), Cow::Owned(S));
    }
}

#[test]
fn to_mut() {
    {
        let mut value = RCow::<&u32, u32>::Borrowed(&100);
        assert_eq!(*value, 100);
        *value.to_mut() = 137;
        assert_eq!(*value, 137);
    }
    {
        let mut value = RCow::<RStr<'_>, RString>::Borrowed("what".into_c());
        assert_eq!(&*value, "what");

        *value.to_mut() = "the".piped(RString::from);
        assert_eq!(&*value, "the");
    }
    {
        let arr = [0, 1, 2, 3];

        let mut value = RCow::<RSlice<'_, u32>, RVec<u32>>::Borrowed((&arr[..]).into_c());
        assert_eq!(&*value, &arr[..]);
        *value.to_mut() = vec![99, 100, 101].into_c();
        assert_eq!(&*value, &[99, 100, 101][..]);
    }
}

#[test]
fn into_owned() {
    {
        let value = RCowVal::<'_, u32>::Borrowed(&100);
        let value: u32 = value.into_owned();
        assert_eq!(value, 100);
    }
    {
        let value = RCowStr::<'_>::Borrowed("what".into());
        let value: RString = value.into_owned();
        assert_eq!(&*value, "what");
    }
    {
        let arr = [0, 1, 2, 3];
        let value = RCowSlice::<'_, u32>::Borrowed((&arr[..]).into());
        let value: RVec<u32> = value.into_owned();
        assert_eq!(&*value, &arr[..]);
    }
}

#[test]
fn deserialize() {
    {
        // Borrowed string
        let json = r##" "what the hell" "##;
        let str_borr = "what the hell".piped(RStr::from);

        let what: BorrowingRCowStr<'_> = serde_json::from_str(json).unwrap();

        assert_eq!(what.cow.as_borrowed(), Some(str_borr),);
    }
    {
        // Owned string
        let json = r##" "what \nthe hell" "##;
        let str_owned = "what \nthe hell".piped(RString::from);

        let what: RCowStr<'_> = serde_json::from_str(json).unwrap();

        assert_eq!(what.as_owned(), Some(&str_owned),);
    }
    {
        // Owned list
        let json = r##" [0, 1, 2, 3] "##;

        let what: RCowSlice<'_, u8> = serde_json::from_str(json).unwrap();

        assert_eq!(what.as_owned(), Some(&vec![0, 1, 2, 3].into_c()),);
    }
    {
        // Borrowed list, using bincode.
        let list = [0u8, 1, 2, 3];
        let serialized = bincode::serialize(&list[..]).unwrap();

        let what: BorrowingRCowU8Slice<'_> = bincode::deserialize(&serialized[..]).unwrap();

        assert_eq!(what.cow.as_borrowed(), Some((&list[..]).into_c()),);
    }
    {
        // Owned value
        let json = r##" 1000 "##;

        let what: RCowVal<'_, u16> = serde_json::from_str(json).unwrap();

        assert_eq!(what.as_owned(), Some(&1000),);
    }
}
