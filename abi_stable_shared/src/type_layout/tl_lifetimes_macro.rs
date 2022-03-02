#[doc(hidden)]
#[macro_export]
macro_rules! declare_tl_lifetime_types {(
    repr=$repr:ty,
    attrs=[ $($extra_attrs:meta),* $(,)* ]
) => (

    /// Which lifetime is being referenced by a field.
    /// Allows lifetimes to be renamed,so long as the "same" lifetime is being referenced.
    #[repr(transparent)]
    #[derive(Copy, Clone, PartialEq, Eq)]
    $(#[ $extra_attrs ])*
    pub struct LifetimeIndex{
        bits:$repr
    }

    impl LifetimeIndex {
        /// A sentinel value to represent the absence of a lifetime.
        ///
        /// Making this be all zeroes allows using `u32::leading_zeros`
        /// to calculate the length of LifetimeIndexArray .
        pub const NONE: Self = LifetimeIndex{bits:0};
        /// A lifetime parameter in a function pointer that is only used once,
        /// and does not appear in the return type.
        pub const ANONYMOUS: Self = LifetimeIndex{bits:1};
        /// A static lifetime.
        pub const STATIC: Self = LifetimeIndex{bits:2};

        const START_OF_LIFETIMES:$repr=3;
        /// The maximum number of lifetime parameters.
        pub const MAX_LIFETIME_PARAM:$repr=15-Self::START_OF_LIFETIMES;

        /// Constructs a LifetimeIndex to the nth lifetime parameter of a type.
        #[inline]
        pub const fn Param(index: $repr) -> LifetimeIndex {
            LifetimeIndex{
                bits:index + Self::START_OF_LIFETIMES,
            }
        }

        /// Gets which lifetiem parameter this is.
        /// Returns None if it's not a lifetime parameter.
        #[inline]
        pub const fn to_param(self)->Option<$repr>{
            if self.bits < Self::START_OF_LIFETIMES {
                None
            }else{
                Some(self.bits-Self::START_OF_LIFETIMES)
            }
        }

        /// Constructs a `LifetimeIndex` from its representation.
        #[inline]
        pub const fn from_u4(bits: u8) -> Self {
            LifetimeIndex{
                bits:(bits & 0b1111)as _
            }
        }

        /// Converts a `LifetimeIndex` into its representation.
        #[inline]
        pub const fn to_u4(self) -> u8 {
            (self.bits & 0b1111)as _
        }
    }

    mod lifetime_index_impls{
        use super::*;
        use std::fmt::{self,Debug};

        impl Debug for LifetimeIndex{
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
                match *self {
                    Self::NONE=>f.debug_struct("NONE").finish(),
                    Self::ANONYMOUS=>f.debug_struct("ANONYMOUS").finish(),
                    Self::STATIC=>f.debug_struct("STATIC").finish(),
                    Self{bits}=>
                        f.debug_tuple("Param")
                         .field(&(bits-Self::START_OF_LIFETIMES))
                         .finish(),
                }
            }
        }
    }


    /////////////////////////////////////////////////////

    /// A `LifetimeIndex::NONE` terminated array of 5 lifetime indices.
    ///
    /// This is represented as `4 bits x 5` inside a u32.
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    $(#[ $extra_attrs ])*
    pub struct LifetimeIndexArray {
        /// (4 bits per LifetimeIndex)*5
        bits: u32,
    }

    impl LifetimeIndexArray {
        /// An array with no lifetimes.
        pub const EMPTY: Self = Self { bits: 0 };

        /// Constructs this LifetimeIndexArray from an array.
        #[inline]
        pub const fn from_array(array: [LifetimeIndex;5]) -> Self {
            let bits= array[0].bits as u32 | ((array[1].bits as u32) << 4)
                | ((array[2].bits as u32) << 8) | ((array[3].bits as u32) << 12)
                | ((array[4].bits as u32) << 16);
            Self {bits}
        }

        /// Converts this LifetimeIndexArray into an array.
        #[inline]
        pub const fn to_array(self) -> [LifetimeIndexPair; 3] {
            [
                LifetimeIndexPair::new(
                    LifetimeIndex{ bits: (self.bits & 0b1111)as $repr },
                    LifetimeIndex{ bits: ((self.bits >> 4) & 0b1111)as $repr },
                ),
                LifetimeIndexPair::new(
                    LifetimeIndex{ bits: ((self.bits >> 8) & 0b1111)as $repr },
                    LifetimeIndex{ bits: ((self.bits >> 12) & 0b1111)as $repr },
                ),
                LifetimeIndexPair::new(
                    LifetimeIndex{ bits: ((self.bits >> 16) & 0b1111)as $repr },
                    LifetimeIndex::NONE,
                )
            ]
        }

        /// Converts this array into its representation.
        #[inline]
        pub const fn to_u20(self) -> u32 {
            self.bits&0xF_FF_FF
        }

        /// Constructs this array from its representation.
        #[inline]
        pub const fn from_u20(bits: u32) -> Self {
            Self {
                bits:bits & 0xF_FF_FF
            }
        }

        /// Gets the length of this array.
        #[inline]
        pub const fn len(mut self) -> usize{
            (8-(self.bits.leading_zeros() >> 2))as usize
        }

        /// Gets whether the array is empty.
        #[inline]
        pub const fn is_empty(self) -> bool{
            self.len() == 0
        }
    }

    /////////////////////////////////////////////////////

    /// Either a `LifetimeArray` or a range into a slice of `LifetimePair`s.
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
    $(#[ $extra_attrs ])*
    pub struct LifetimeRange{
        /// 21 bits:
        ///      (13 for the start index | 12 for the LifetimeIndexArray)
        ///     +8 bits for the length
        bits:u32,
    }

    impl LifetimeRange{
        /// An empty `LifetimeRange`.
        pub const EMPTY: Self = Self { bits: 0 };

        const IS_RANGE_BIT: u32 = 1<<20;

        const RANGE_LEN_OFFSET: u32 = 13;
        const LEN_SR_MASK: u32 = 0b111_1111;
        const LEN_BIT_SIZE: u32 = 7;

        const MASK: u32 = 0x1F_FF_FF;
        const START_MASK: u32 = 0b1_1111_1111_1111;

        /// The amount of bits used to represent a LifetimeRnage.
        pub const BIT_SIZE:u32=21;

        /// The maximum value for the start of a range.
        pub const MAX_START:usize=Self::START_MASK as usize;
        /// The maximum length of a range.
        pub const MAX_LEN:usize=Self::LEN_SR_MASK as usize;

        /// Constructs a LifetimeRange from a single lifetime parameter.
        pub const fn Param(index: $repr) -> Self {
            Self::from_array([
                LifetimeIndex::Param(index),
                LifetimeIndex::NONE,
                LifetimeIndex::NONE,
                LifetimeIndex::NONE,
                LifetimeIndex::NONE,
            ])
        }

        /// Constructs a LifetimeRange from an array of 5 lifetime indices.
        #[inline]
        pub const fn from_array(lia:[LifetimeIndex;5])->Self{
            Self{
                bits:LifetimeIndexArray::from_array(lia).to_u20()
            }
        }

        /// Constructs a LifetimeRange from a range.
        #[inline]
        pub const fn from_range(range:std::ops::Range<usize>)->Self{
            let len=range.end-range.start;
            Self{
                bits:
                    Self::IS_RANGE_BIT
                    |((range.start as u32)&Self::START_MASK)
                    |((len as u32 & Self::LEN_SR_MASK) << Self::RANGE_LEN_OFFSET)
            }
        }

        #[inline]
        const fn range_len(self)->usize{
            ((self.bits >> Self::RANGE_LEN_OFFSET) & Self::LEN_SR_MASK) as usize * 2
        }

        /// The amount of lifetime indices this spans.
        #[inline]
        pub const fn len(self) -> usize {
            if self.is_range() {
                self.range_len()
            }else{
                LifetimeIndexArray::from_u20(self.bits).len()
            }
        }

        /// Whether this span of lifetimes is empty.
        #[inline]
        pub const fn is_empty(self) -> bool {
            self.len() == 0
        }

        /// Whether this is a range into a slice of `LifetimePair`s.
        #[inline]
        pub const fn is_range(self)->bool{
            (self.bits&Self::IS_RANGE_BIT)==Self::IS_RANGE_BIT
        }

        /// Converts this `LifetimeRange` into its representation
        #[inline]
        pub const fn to_u21(self) -> u32 {
            self.bits & Self::MASK
        }

        /// Constructs this `LifetimeRange` from its representation
        #[inline]
        pub const fn from_u21(bits: u32) -> Self {
            Self {
                bits: bits & Self::MASK,
            }
        }
    }


    ////////////////////////////////////////////////////////////////////////////////

    /// A pair of `LifetimeIndex`.
    #[repr(transparent)]
    #[derive(Copy, Clone, PartialEq, Eq)]
    $(#[ $extra_attrs ])*
    pub struct LifetimeIndexPair{
        bits:$repr,
    }

    /// The representation of `LifetimeIndexPair`.
    pub type LifetimeIndexPairRepr=u8;

    impl LifetimeIndexPair{
        /// A pair of `LifetimeIndex::NONE`
        pub const NONE:LifetimeIndexPair=
            LifetimeIndexPair::new( LifetimeIndex::NONE, LifetimeIndex::NONE );

        /// Constructs a `LifetimeIndexPair` from a pair of `LifetimeIndex`
        #[inline]
        pub const fn new(first:LifetimeIndex,second:LifetimeIndex)->Self{
            Self{
                bits:(first.to_u4() | (second.to_u4()<<4)) as _,
            }
        }

        /// Gets the first `LifetimeIndex` of this pair.
        #[inline]
        pub const fn first(self)->LifetimeIndex{
            LifetimeIndex::from_u4(self.bits as _)
        }

        /// Gets the second `LifetimeIndex` of this pair.
        #[inline]
        pub const fn second(self)->LifetimeIndex{
            LifetimeIndex::from_u4((self.bits>>4) as _)
        }

        /// Gets both `LifetimeIndex` in this `LifetimeIndexPair`.
        #[inline]
        pub const fn both(self)->(LifetimeIndex,LifetimeIndex){
            (self.first(),self.second())
        }

        /// Converts this `LifetimeIndexPair` into its representation.
        pub const fn to_u8(self)->u8{
            self.bits as _
        }
    }


    mod lifetime_index_pair_impls{
        use super::*;
        use std::fmt::{self,Debug};

        impl Debug for LifetimeIndexPair{
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
                f.debug_list()
                 .entry(&self.first())
                 .entry(&self.second())
                 .finish()
            }
        }
    }

)}
