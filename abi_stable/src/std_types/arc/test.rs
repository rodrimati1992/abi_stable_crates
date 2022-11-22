use super::*;

use std::cell::Cell;

#[allow(clippy::redundant_allocation)]
fn _covariant_arc<'a: 'b, 'b, T>(foo: Arc<&'a T>) -> Arc<&'b T> {
    foo
}

fn _covariant_rarc<'a: 'b, 'b, T>(foo: RArc<&'a T>) -> RArc<&'b T> {
    foo
}

#[test]
fn test_covariance() {
    struct F<T>(T);

    fn eq<'a, 'b, T>(left: &RArc<&'a T>, right: &RArc<&'b T>) -> bool
    where
        T: PartialEq,
    {
        left == right
    }

    let aaa = F(3);
    let bbb = F(5);

    let v0 = RArc::new(&aaa.0);
    let v1 = RArc::new(&bbb.0);

    assert!(!eq(&v0, &v1));
}

fn refaddr<T>(ref_: &T) -> usize {
    ref_ as *const T as usize
}

#[test]
fn to_from_arc() {
    let orig_a = Arc::new(1000);
    let a_addr = (&*orig_a) as *const _ as usize;
    let mut reprc_a = orig_a.clone().piped(RArc::from);

    assert_eq!(a_addr, refaddr(&*reprc_a));

    assert_eq!(a_addr, reprc_a.clone().piped(|a| refaddr(&*a)));
    assert_eq!(
        a_addr,
        reprc_a
            .clone()
            .piped(RArc::into_arc)
            .piped(|a| refaddr(&*a))
    );

    reprc_a.set_vtable_for_testing();

    assert_eq!(a_addr, refaddr(&*reprc_a));
    assert_eq!(Arc::strong_count(&orig_a), 2);

    let back_to_a = reprc_a.piped(RArc::into_arc);
    assert_eq!(Arc::strong_count(&orig_a), 1);
    assert_ne!(a_addr, refaddr(&*back_to_a));
    drop(back_to_a);

    assert_eq!(Arc::strong_count(&orig_a), 1);
}

// testing that Arc<()> is valid
#[allow(clippy::unit_cmp)]
#[test]
fn default() {
    assert_eq!(*RArc::<String>::default(), "");
    assert_eq!(*RArc::<()>::default(), ());
    assert_eq!(*RArc::<u32>::default(), 0);
    assert_eq!(*RArc::<bool>::default(), false);
}

#[test]
fn new_test() {
    for elem in 0..100 {
        assert_eq!(*RArc::new(elem), elem);
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

#[test]
fn get_mut() {
    let mut conv = Arc::new(200).piped(RArc::from);

    {
        let _conv_clone = conv.clone();
        assert_eq!(RArc::get_mut(&mut conv), None);
    }
    assert_eq!(RArc::get_mut(&mut conv), Some(&mut 200));
}

#[test]
fn make_mut() {
    let count = Cell::new(1);
    let dod = DecrementOnDrop(&count);

    let mut arc = Arc::new(ValueAndDod {
        value: 'a',
        _dod: dod.clone(),
    })
    .piped(RArc::from);

    {
        assert_eq!(dod.count(), 2);
        let arc_clone = arc.clone();

        let mutref = RArc::make_mut(&mut arc);
        assert_eq!(dod.count(), 3);
        mutref.value = 'c';

        assert_eq!(arc_clone.value, 'a');
    }
    assert_eq!(dod.count(), 2);
    assert_eq!(arc.value, 'c');
}

/////////////////////////////////////////

#[derive(Clone)]
struct ValueAndDod<'a, T> {
    value: T,
    _dod: DecrementOnDrop<'a>,
}

/////////////////////////////////////////

struct DecrementOnDrop<'a>(&'a Cell<u32>);

impl<'a> DecrementOnDrop<'a> {
    fn count(&self) -> u32 {
        self.0.get()
    }
}

impl<'a> Clone for DecrementOnDrop<'a> {
    fn clone(&self) -> Self {
        self.0.set(self.0.get() + 1);
        DecrementOnDrop(self.0)
    }
}

impl<'a> Drop for DecrementOnDrop<'a> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}
