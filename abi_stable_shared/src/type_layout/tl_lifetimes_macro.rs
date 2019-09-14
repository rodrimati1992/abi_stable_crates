#[doc(hidden)]
#[macro_export]
macro_rules! declare_tl_lifetime_types {( 
    attrs=[ $($extra_attrs:meta),* $(,)* ]
) => (

    /// Which lifetime is being referenced by a field.
    /// Allows lifetimes to be renamed,so long as the "same" lifetime is being referenced.
    #[repr(transparent)]
    #[derive(Copy, Clone, PartialEq, Eq)]
    $(#[ $extra_attrs ])*
    pub struct LifetimeIndex{
        bits:u8
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
        pub const STATIC: Self = LifetimeIndex{bits:2};

        const START_OF_LIFETIMES:u8=3;

        /// Constructs a LifetimeIndex to the nth lifetime parameter of a type.
        pub const fn Param(index: u8) -> LifetimeIndex {
            LifetimeIndex{
                bits:index + Self::START_OF_LIFETIMES,
            }
        }

        pub fn to_param(self)->Option<u8>{
            if self.bits < Self::START_OF_LIFETIMES {
                None
            }else{
                Some(self.bits-Self::START_OF_LIFETIMES)
            }
        }

        pub const fn from_u4(bits: u8) -> Self {
            LifetimeIndex{
                bits:bits & 0b1111
            }
        }

        pub const fn to_u4(self) -> u8 {
            self.bits & 0b1111
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

    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    $(#[ $extra_attrs ])*
    pub struct LifetimeIndexArray {
        /// (4 bits per LifetimeIndex)*5
        bits: u32,
    }

    impl LifetimeIndexArray {
        pub const EMPTY: Self = Self { bits: 0 };

        pub const fn from_array(array: [LifetimeIndex;5]) -> Self {
            let bits= array[0].bits as u32 | ((array[1].bits as u32) << 4)
                | ((array[2].bits as u32) << 8) | ((array[3].bits as u32) << 12)
                | ((array[4].bits as u32) << 16);
            Self {bits}
        }
        pub const fn to_array(self) -> [LifetimeIndexPair; 3] {
            [
                LifetimeIndexPair::new(
                    LifetimeIndex{ bits: (self.bits & 0b1111)as u8 },
                    LifetimeIndex{ bits: ((self.bits >> 4) & 0b1111)as u8 },
                ),
                LifetimeIndexPair::new(
                    LifetimeIndex{ bits: ((self.bits >> 8) & 0b1111)as u8 },
                    LifetimeIndex{ bits: ((self.bits >> 12) & 0b1111)as u8 },
                ),
                LifetimeIndexPair::new(
                    LifetimeIndex{ bits: ((self.bits >> 16) & 0b1111)as u8 },
                    LifetimeIndex::NONE,
                )
            ]
        }
        pub const fn to_u20(self) -> u32 {
            self.bits&0xF_FF_FF 
        }
        pub const fn from_u20(bits: u32) -> Self {
            Self {
                bits:bits&0xF_FF_FF 
            }
        }
        pub const fn len(mut self)->usize{
            (8-(self.bits.leading_zeros()>>2))as usize
        }
    }

    /////////////////////////////////////////////////////


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
        pub const EMPTY: Self = Self { bits: 0 };
        
        pub const IS_RANGE_BIT: u32 = 1<<20;

        pub const RANGE_LEN_OFFSET: u32 = 13;
        pub const LEN_SR_MASK: u32 = 0b111_1111;
        pub const LEN_BIT_SIZE: u32 = 7;
        
        pub const MASK: u32 = 0x1F_FF_FF;
        pub const START_MASK: u32 = 0b1_1111_1111_1111;
        pub const BIT_SIZE:u32=21;

        pub const fn Param(index: u8) -> Self {
            Self::from_array([
                LifetimeIndex::Param(index),
                LifetimeIndex::NONE,
                LifetimeIndex::NONE,
                LifetimeIndex::NONE,
                LifetimeIndex::NONE,
            ])
        }

        #[inline]
        pub const fn from_array(lia:[LifetimeIndex;5])->Self{
            Self{
                bits:LifetimeIndexArray::from_array(lia).to_u20()
            }
        }

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
            ((self.bits >> Self::RANGE_LEN_OFFSET) & Self::LEN_SR_MASK) as usize
        }

        /// The ammount of lifetime indices this spans.
        #[inline]
        pub fn len(self) -> usize {
            if self.is_range() {
                self.range_len()
            }else{
                LifetimeIndexArray::from_u20(self.bits).len()
            }
        }
        pub const fn is_range(self)->bool{
            (self.bits&Self::IS_RANGE_BIT)==Self::IS_RANGE_BIT
        }
        pub const fn to_u21(self) -> u32 {
            self.bits & Self::MASK
        }
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
        pub bits:u8,
    }

    pub type LifetimeIndexPairRepr=u8;

    impl LifetimeIndexPair{
        pub const STATICS:LifetimeIndexPair=
            LifetimeIndexPair::new( LifetimeIndex::STATIC, LifetimeIndex::STATIC );

        pub const NONE:LifetimeIndexPair=
            LifetimeIndexPair::new( LifetimeIndex::NONE, LifetimeIndex::NONE );

        pub const fn new(first:LifetimeIndex,second:LifetimeIndex)->Self{
            Self{
                bits:first.to_u4() | (second.to_u4()<<4),
            }
        }

        #[inline]
        pub const fn first(self)->LifetimeIndex{
            LifetimeIndex::from_u4(self.bits)
        }

        #[inline]
        pub const fn second(self)->LifetimeIndex{
            LifetimeIndex::from_u4(self.bits>>4)
        }

        #[inline]
        pub const fn both(self)->(LifetimeIndex,LifetimeIndex){
            (self.first(),self.second())
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