use super::*;

use std::sync::Arc;

use crate::{sabi_types::MovePtr, test_utils::must_panic};

#[allow(clippy::redundant_allocation)]
fn _covariant_box<'a: 'b, 'b, T>(foo: Box<&'a T>) -> Box<&'b T> {
    foo
}

fn _covariant_rbox<'a: 'b, 'b, T>(foo: RBox<&'a T>) -> RBox<&'b T> {
    foo
}

#[test]
fn test_covariance() {
    struct F<T>(T);

    fn eq<'a, 'b, T>(left: &RBox<&'a T>, right: &RBox<&'b T>) -> bool
    where
        T: PartialEq,
    {
        left == right
    }

    let aaa = F(3);
    let bbb = F(5);

    let v0 = RBox::new(&aaa.0);
    let v1 = RBox::new(&bbb.0);

    assert!(!eq(&v0, &v1));
}

#[test]
fn new_and_drop() {
    let arc_a = Arc::new(100);

    let box_a = RBox::new(arc_a.clone());
    assert_eq!(&**box_a, &*arc_a);
    assert_eq!(Arc::strong_count(&arc_a), 2);
    drop(box_a);
    assert_eq!(Arc::strong_count(&arc_a), 1);
}

#[test]
fn from_to_box() {
    let arc_a = Arc::new(100);

    let box_a = Box::new(arc_a.clone()).piped(RBox::<Arc<i32>>::from);
    assert_eq!(&**box_a, &*arc_a);
    assert_eq!(Arc::strong_count(&arc_a), 2);
    let box_b = box_a.piped(RBox::into_box);
    assert_eq!(Arc::strong_count(&arc_a), 2);
    let mut box_c = box_b.piped(RBox::<Arc<i32>>::from);

    box_c.set_vtable_for_testing();
    let box_c_addr = (&*box_c) as *const _;
    let box_d = box_c.piped(RBox::into_box);
    let box_d_addr = (&*box_d) as *const _;
    assert_eq!(Arc::strong_count(&arc_a), 2);
    assert_ne!(box_c_addr, box_d_addr);
    println!("{}-{:p}-{:p}", line!(), box_c_addr, box_d_addr);
}

#[test]
fn from_elem_into_inner() {
    let arc_a = Arc::new(100);

    let box_a = RBox::new(arc_a.clone());
    assert_eq!(&**box_a, &*arc_a);
    assert_eq!(Arc::strong_count(&arc_a), 2);

    let _value = box_a.piped(RBox::into_inner);
    assert_eq!(Arc::strong_count(&arc_a), 2);
}

#[test]
fn clone() {
    let a = RBox::new(10);
    let a_addr = (&*a) as *const _;

    let b = a.clone();
    let b_addr = (&*b) as *const _;

    assert_eq!(a, b);
    assert_ne!(a_addr, b_addr);
}

#[test]
fn mutated() {
    let mut a = RBox::new(10);
    assert_eq!(*a, 10);

    *a = 1337;
    assert_eq!(*a, 1337);
}

#[test]
fn with_move_ptr_runs() {
    let rbox = ManuallyDrop::new(RBox::new(rvec![3]));

    must_panic(|| {
        OwnedPointer::with_move_ptr(rbox, |_| {
            panic!();
        });
    })
    .unwrap();

    let rbox = ManuallyDrop::new(RBox::new(rvec![5]));
    assert_eq!(OwnedPointer::with_move_ptr(rbox, |_| 10), 10);
}

#[test]
fn owned_pointer_trait() {
    let arc = Arc::new(10);

    {
        let cloned_arc = ManuallyDrop::new(RBox::new(arc.clone()));

        OwnedPointer::with_move_ptr(cloned_arc, |move_ptr| {
            assert_eq!(Arc::strong_count(&move_ptr), 2);

            let moved_arc = MovePtr::into_inner(move_ptr);
            assert_eq!(Arc::strong_count(&moved_arc), 2);
        });
    }
    assert_eq!(Arc::strong_count(&arc), 1);
    {
        let cloned_arc = ManuallyDrop::new(RBox::new(arc.clone()));

        OwnedPointer::with_move_ptr(cloned_arc, |move_ptr| {
            assert_eq!(Arc::strong_count(&move_ptr), 2);
        });
    }
    assert_eq!(Arc::strong_count(&arc), 1);
}
