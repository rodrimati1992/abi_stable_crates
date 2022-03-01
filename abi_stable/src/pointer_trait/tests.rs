#[cfg(feature = "rust_1_56")]
use super::{immutable_ref, GetPointerKind, IsReference};

#[cfg(feature = "rust_1_56")]
use std::ptr::NonNull;

#[cfg(feature = "rust_1_56")]
#[test]
fn teest_to_nonnull() {
    unsafe {
        let x = immutable_ref::to_nonnull(&3i32, IsReference::NEW);
        assert_eq!(*x.cast::<u32>().as_ref(), 3u32);
    }
}

#[cfg(feature = "rust_1_56")]
#[test]
fn teest_from_nonnull() {
    unsafe {
        let x = immutable_ref::from_nonnull(NonNull::from(&5i32), <&_>::IS_PTR);
        assert_eq!(*x, 5i32);
    }
}

#[cfg(feature = "rust_1_56")]
#[test]
fn teest_to_raw_ptr() {
    unsafe {
        let x = immutable_ref::to_raw_ptr(&8i32, IsReference::NEW);
        assert_eq!(*x.cast::<u32>(), 8u32);
    }
}

#[cfg(feature = "rust_1_56")]
#[test]
fn teest_from_raw_ptr() {
    unsafe {
        let x = immutable_ref::from_raw_ptr(&13i32 as *const i32, <&_>::IS_PTR);
        assert_eq!(*x.unwrap(), 13i32);
    }
    unsafe {
        let x = immutable_ref::from_raw_ptr(std::ptr::null::<i32>(), <&_>::IS_PTR);
        assert!(x.is_none());
    }
}
