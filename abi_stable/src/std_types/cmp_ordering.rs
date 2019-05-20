use std::cmp::Ordering;

/// Ffi-safe equivalent of ::std::cmp::Ordering.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(u8)]
#[derive(StableAbi)]
pub enum RCmpOrdering {
    #[serde(rename = "Less")]
    LessR,
    #[serde(rename = "Equal")]
    EqualR,
    #[serde(rename = "Greater")]
    GreaterR,
}

impl_from_rust_repr! {
    impl From<Ordering> for RCmpOrdering {
        fn(this){
            match this {
                Ordering::Less=>RCmpOrdering::LessR,
                Ordering::Equal=>RCmpOrdering::EqualR,
                Ordering::Greater=>RCmpOrdering::GreaterR,
            }
        }
    }
}

impl_into_rust_repr! {
    impl Into<Ordering> for RCmpOrdering {
        fn(this){
            match this {
                RCmpOrdering::LessR=>Ordering::Less,
                RCmpOrdering::EqualR=>Ordering::Equal,
                RCmpOrdering::GreaterR=>Ordering::Greater,
            }
        }
    }
}
