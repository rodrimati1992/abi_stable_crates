use std::cmp::Ordering;

/// Ffi-safe equivalent of ::std::cmp::Ordering.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(u8)]
#[derive(StableAbi)]
pub enum RCmpOrdering {
    Less,
    Equal,
    Greater,
}

impl_from_rust_repr! {
    impl From<Ordering> for RCmpOrdering {
        fn(this){
            match this {
                Ordering::Less=>RCmpOrdering::Less,
                Ordering::Equal=>RCmpOrdering::Equal,
                Ordering::Greater=>RCmpOrdering::Greater,
            }
        }
    }
}

impl_into_rust_repr! {
    impl Into<Ordering> for RCmpOrdering {
        fn(this){
            match this {
                RCmpOrdering::Less=>Ordering::Less,
                RCmpOrdering::Equal=>Ordering::Equal,
                RCmpOrdering::Greater=>Ordering::Greater,
            }
        }
    }
}
