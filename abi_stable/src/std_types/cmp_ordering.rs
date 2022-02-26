//! Contains the ffi-safe equivalent of `std::cmp::Ordering`.

use std::cmp::Ordering;

/// Ffi-safe equivalent of `std::cmp::Ordering`.
///
/// # Example
///
/// This defines an extern function, which compares a slice to another.
///
/// ```rust
///
/// use abi_stable::{
///     sabi_extern_fn,
///     std_types::{RCmpOrdering, RSlice},
/// };
/// use std::cmp::Ord;
///
/// #[sabi_extern_fn]
/// pub fn compare_slices<T>(l: RSlice<'_, T>, r: RSlice<'_, T>) -> RCmpOrdering
/// where
///     T: Ord,
/// {
///     l.cmp(&r).into()
/// }
///
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[repr(u8)]
#[derive(StableAbi)]
pub enum RCmpOrdering {
    Less,
    Equal,
    Greater,
}

impl RCmpOrdering {
    /// Converts this `RCmpOrdering` into a `std::cmp::Ordering`;
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RCmpOrdering;
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(RCmpOrdering::Less.to_ordering(), Ordering::Less);
    /// assert_eq!(RCmpOrdering::Equal.to_ordering(), Ordering::Equal);
    /// assert_eq!(RCmpOrdering::Greater.to_ordering(), Ordering::Greater);
    ///
    /// ```
    #[inline]
    pub const fn to_ordering(self) -> Ordering {
        match self {
            RCmpOrdering::Less => Ordering::Less,
            RCmpOrdering::Equal => Ordering::Equal,
            RCmpOrdering::Greater => Ordering::Greater,
        }
    }

    /// Converts a [`std::cmp::Ordering`] to [`RCmpOrdering`];
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RCmpOrdering;
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(RCmpOrdering::from_ordering(Ordering::Less), RCmpOrdering::Less);
    /// assert_eq!(RCmpOrdering::from_ordering(Ordering::Equal), RCmpOrdering::Equal);
    /// assert_eq!(RCmpOrdering::from_ordering(Ordering::Greater), RCmpOrdering::Greater);
    ///
    /// ```
    #[inline]
    pub const fn from_ordering(ordering: Ordering) -> RCmpOrdering {
        match ordering {
            Ordering::Less => RCmpOrdering::Less,
            Ordering::Equal => RCmpOrdering::Equal,
            Ordering::Greater => RCmpOrdering::Greater,
        }
    }
}

impl_from_rust_repr! {
    impl From<Ordering> for RCmpOrdering {
        fn(this){
            RCmpOrdering::from_ordering(this)
        }
    }
}

impl_into_rust_repr! {
    impl Into<Ordering> for RCmpOrdering {
        fn(this){
            this.to_ordering()
        }
    }
}
