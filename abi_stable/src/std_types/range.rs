//! Contains the ffi-safe equivalent of `std::ops::Range*` types.

use std::ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};

////////////////////////////////////////////////////////////////

macro_rules! impl_into_iterator {
    ( $from: ident, $to: ident ) => {
        impl<T> IntoIterator for $from<T>
        where
            $to<T>: Iterator<Item = T>,
        {
            type IntoIter = $to<T>;
            type Item = T;

            #[inline]
            fn into_iter(self) -> $to<T> {
                self.into()
            }
        }
    };
}

////////////////////////////////////////////////////////////////

/// Ffi-safe equivalent of `::std::ops::Range`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(C)]
#[derive(StableAbi)]
pub struct RRange<T> {
    pub start: T,
    pub end: T,
}

impl RRange<usize> {
    pub const fn from_std(v: Range<usize>) -> Self {
        Self {
            start: v.start,
            end: v.end,
        }
    }
}

impl_from_rust_repr! {
    impl[T] From<Range<T>> for RRange<T> {
        fn(v){
            Self {
                start: v.start,
                end: v.end,
            }
        }
    }
}

impl_into_rust_repr! {
    impl[T] Into<Range<T>> for RRange<T> {
        fn(this){
            Range {
                start: this.start,
                end: this.end,
            }
        }
    }
}

impl_into_iterator! { RRange, Range }

////////////////////////////////////////////////////////////////

/// Ffi-safe equivalent of `::std::ops::RangeInclusive`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(C)]
#[derive(StableAbi)]
pub struct RRangeInclusive<T> {
    pub start: T,
    pub end: T,
}

impl_from_rust_repr! {
    impl[T] From<RangeInclusive<T>> for RRangeInclusive<T> {
        fn(v){
            let (start, end) = v.into_inner();
            Self { start, end }
        }
    }
}

impl_into_rust_repr! {
    impl[T] Into<RangeInclusive<T>> for RRangeInclusive<T> {
        fn(this){
            RangeInclusive::new(this.start, this.end)
        }
    }
}

impl_into_iterator! { RRangeInclusive, RangeInclusive }

////////////////////////////////////////////////////////////////

/// Ffi-safe equivalent of `::std::ops::RangeFrom`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(C)]
#[derive(StableAbi)]
pub struct RRangeFrom<T> {
    pub start: T,
}

impl_from_rust_repr! {
    impl[T] From<RangeFrom<T>> for RRangeFrom<T> {
        fn(v){
            Self { start: v.start }
        }
    }
}

impl_into_rust_repr! {
    impl[T] Into<RangeFrom<T>> for RRangeFrom<T> {
        fn(this){
            this.start..
        }
    }
}

impl_into_iterator! { RRangeFrom, RangeFrom }

////////////////////////////////////////////////////////////////

/// Ffi-safe equivalent of `::std::ops::RangeTo`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(C)]
#[derive(StableAbi)]
pub struct RRangeTo<T> {
    pub end: T,
}

impl_from_rust_repr! {
    impl[T] From<RangeTo<T>> for RRangeTo<T> {
        fn(v){
            Self { end: v.end }
        }
    }
}

impl_into_rust_repr! {
    impl[T] Into<RangeTo<T>> for RRangeTo<T> {
        fn(this){
            ..this.end
        }
    }
}

////////////////////////////////////////////////////////////////

/// Ffi-safe equivalent of `::std::ops::RangeToInclusive`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(C)]
#[derive(StableAbi)]
pub struct RRangeToInclusive<T> {
    pub end: T,
}

impl_from_rust_repr! {
    impl[T] From<RangeToInclusive<T>> for RRangeToInclusive<T> {
        fn(v){
            Self { end: v.end }
        }
    }
}

impl_into_rust_repr! {
    impl[T] Into<RangeToInclusive<T>> for RRangeToInclusive<T> {
        fn(this){
             ..= this.end
        }
    }
}

////////////////////////////////////////////////////////////////
