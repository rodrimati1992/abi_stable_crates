use super::immutable_ref;

use std::ptr::NonNull;

#[test]
fn teest_to_nonnull() {
    unsafe {
        let x = immutable_ref::to_nonnull(&3i32);
        assert_eq!(*x.cast::<u32>().as_ref(), 3u32);
    }
}

#[test]
fn teest_from_nonnull() {
    unsafe {
        let x = immutable_ref::from_nonnull::<&_>(NonNull::from(&5i32));
        assert_eq!(*x, 5i32);
    }
}

#[test]
fn teest_to_raw_ptr() {
    unsafe {
        let x = immutable_ref::to_raw_ptr(&8i32);
        assert_eq!(*x.cast::<u32>(), 8u32);
    }
}

#[test]
fn teest_from_raw_ptr() {
    unsafe {
        let x = immutable_ref::from_raw_ptr::<&_>(&13i32 as *const i32);
        assert_eq!(*x.unwrap(), 13i32);
    }
    unsafe {
        let x = immutable_ref::from_raw_ptr::<&_>(std::ptr::null::<i32>());
        assert!(x.is_none());
    }
}
