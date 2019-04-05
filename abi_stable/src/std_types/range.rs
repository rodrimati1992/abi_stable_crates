use std::ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};

////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
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

////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
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

////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
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

////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
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
            ..=this.end
        }
    }
}

////////////////////////////////////////////////////////////////
