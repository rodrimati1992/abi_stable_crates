use super::*;

#[test]
fn to_from_arc() {
    let orig_a = Arc::new(1000);
    let a_addr = (&*orig_a) as *const _;
    let mut reprc_a = orig_a.clone().piped(RArc::from);

    assert_eq!(a_addr, &*reprc_a);

    assert_eq!(a_addr, &*reprc_a.clone());
    assert_eq!(a_addr, &*reprc_a.clone().piped(RArc::into_arc));

    reprc_a.set_vtable_for_testing();

    assert_eq!(a_addr, &*reprc_a);
    assert_eq!(Arc::strong_count(&orig_a), 2);

    let back_to_a = reprc_a.piped(RArc::into_arc);
    assert_eq!(Arc::strong_count(&orig_a), 1);
    assert_ne!(a_addr, &*back_to_a);
    drop(back_to_a);

    assert_eq!(Arc::strong_count(&orig_a), 1);
}

#[test]
fn default() {
    assert_eq!(*RArc::<String>::default(), "");
    assert_eq!(*RArc::<()>::default(), ());
    assert_eq!(*RArc::<u32>::default(), 0);
    assert_eq!(*RArc::<bool>::default(), false);
}

#[test]
fn from_elem() {
    for elem in 0..100 {
        assert_eq!(*RArc::from_elem(elem), elem);
    }
}

#[test]
fn into_raw() {
    let orig_a = Arc::new(200);
    let reprc_a = orig_a.clone().piped(RArc::from);
    let raw_a = reprc_a.into_raw();
    assert_eq!(Arc::strong_count(&orig_a), 2);
    unsafe {
        Arc::from_raw(raw_a);
    }
    assert_eq!(Arc::strong_count(&orig_a), 1);
}
