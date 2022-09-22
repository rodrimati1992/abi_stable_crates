use super::{data_structures::ArrayLen, *};

use std::ops::{Deref, Index};

abi_stable_shared::declare_tl_lifetime_types! {
    repr=u8,
    attrs=[
        derive(StableAbi),
    ]
}

////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

impl Display for LifetimeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            LifetimeIndex::STATIC => f.write_str("'static"),
            LifetimeIndex::ANONYMOUS => f.write_str("'_"),
            LifetimeIndex::NONE => f.write_str("'NONE"),
            LifetimeIndex { bits: n } => write!(f, "'{}", n - Self::START_OF_LIFETIMES),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl LifetimeRange {
    /// Expands this `LifetimeRange` into a `LifetimeArrayOrSlice`
    pub fn slicing(self, lifetime_indices: &[LifetimeIndexPair]) -> LifetimeArrayOrSlice<'_> {
        let len = (self.len() + 1) / 2;
        if self.is_range() {
            let start = (self.bits & Self::START_MASK) as usize;
            let end = start + len;
            let x = RSlice::from_slice(&lifetime_indices[start..end]);
            LifetimeArrayOrSlice::Slice(x)
        } else {
            LifetimeArrayOrSlice::Array(ArrayLen {
                len: len as u16,
                array: LifetimeIndexArray::from_u20(self.bits).to_array(),
            })
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Either an array of 3 `LifetimeIndexPair`,or a slice of `LifetimeIndexPair`.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub enum LifetimeArrayOrSlice<'a> {
    ///
    Slice(RSlice<'a, LifetimeIndexPair>),
    ///
    Array(ArrayLen<[LifetimeIndexPair; 3]>),
}

impl<'a> LifetimeArrayOrSlice<'a> {
    /// An empty `LifetimeArrayOrSlice`.
    pub const EMPTY: Self = LifetimeArrayOrSlice::Array(ArrayLen {
        len: 0,
        array: [
            LifetimeIndexPair::NONE,
            LifetimeIndexPair::NONE,
            LifetimeIndexPair::NONE,
        ],
    });

    /// Gets a slice of the `LifetimeIndexPair` this contains.
    pub fn as_slice(&self) -> &[LifetimeIndexPair] {
        match self {
            LifetimeArrayOrSlice::Slice(slice) => slice.as_slice(),
            LifetimeArrayOrSlice::Array(arraylen) => &arraylen.array[..arraylen.len as usize],
        }
    }
}

impl<'a> Deref for LifetimeArrayOrSlice<'a> {
    type Target = [LifetimeIndexPair];

    #[inline]
    fn deref(&self) -> &[LifetimeIndexPair] {
        self.as_slice()
    }
}

impl<'a, I, Output: ?Sized> Index<I> for LifetimeArrayOrSlice<'a>
where
    [LifetimeIndexPair]: Index<I, Output = Output>,
{
    type Output = Output;

    #[inline]
    fn index(&self, i: I) -> &Output {
        &self.as_slice()[i]
    }
}

////////////////////////////////////////////////////////////////////////////////
