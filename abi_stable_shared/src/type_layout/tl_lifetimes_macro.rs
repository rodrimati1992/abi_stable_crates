#[doc(hidden)]
#[macro_export]
macro_rules! declare_tl_lifetime_types {( 
    attrs=[ $($extra_attrs:meta),* $(,)* ]
) => (

    /// Which lifetime is being referenced by a field.
    /// Allows lifetimes to be renamed,so long as the "same" lifetime is being referenced.
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    $(#[ $extra_attrs ])*
    pub struct LifetimeIndex{
        bits:u8
    }

    impl LifetimeIndex {
        pub const STATIC: Self = LifetimeIndex{bits:0};
        /// A lifetime parameter in a function pointer that is only used once,
        /// and does not appear in the return type.
        pub const ANONYMOUS: Self = LifetimeIndex{bits:1};
        /// A sentinel value to represent the absence of a lifetime.
        pub const NONE: Self = LifetimeIndex{bits:2};

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
                Some(self.bits)
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

    /////////////////////////////////////////////////////

    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    $(#[ $extra_attrs ])*
    pub struct LifetimeIndexArray {
        /// (4 bits per LifetimeIndex)*3
        bits: u16,
    }

    impl LifetimeIndexArray {
        pub const EMPTY: Self = Self { bits: 0 };

        const MASK: u16 = 0b1111_1111_1111;

        pub const fn with_1(index: LifetimeIndex) -> Self {
            Self {
                bits: index.bits as u16,
            }
        }
        pub const fn with_2(i0: LifetimeIndex,i1:LifetimeIndex) -> Self {
            Self {
                bits: i0.bits as u16 | ((i1.bits as u16) << 4),
            }
        }
        pub const fn with_3(i0: LifetimeIndex,i1: LifetimeIndex,i2: LifetimeIndex) -> Self {
            Self {
                bits: i0.bits as u16 | ((i1.bits as u16) << 4) | ((i2.bits as u16) << 8),
            }
        }
        pub const fn to_array(self) -> [LifetimeIndexPair; 2] {
            [
                LifetimeIndexPair::new(
                    LifetimeIndex{ bits: (self.bits & 0b1111)as u8 },
                    LifetimeIndex{ bits: ((self.bits >> 4) & 0b1111)as u8 },
                ),
                LifetimeIndexPair::new(
                    LifetimeIndex{ bits: ((self.bits >> 8) & 0b1111)as u8 },
                    LifetimeIndex::NONE,
                )
            ]
        }
        pub const fn to_u12(self) -> u16 {
            self.bits & Self::MASK
        }
        pub const fn from_u12(bits: u16) -> Self {
            Self {
                bits: bits & Self::MASK,
            }
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
        
        pub const LEN_OFFSET: u32 = 13;
        pub const MASK: u32 = 0x1FFFFF;
        pub const START_MASK: u32 = 0b1_1111_1111_1111;
        pub const BIT_SIZE:u32=21;
        
        #[inline]
        pub const fn with_array_length(lia:LifetimeIndexArray,len:usize)->Self{
            Self{
                bits:(lia.bits as u32) | ((len as u32) << Self::LEN_OFFSET)
            }
        }

        pub const fn Param(index: u8) -> Self {
            Self::with_1(LifetimeIndex::Param(index))
        }
        pub const fn with_1(index: LifetimeIndex) -> Self {
            Self::with_array_length( LifetimeIndexArray::with_1(index), 1 )
        }
        pub const fn with_2(i0: LifetimeIndex,i1:LifetimeIndex) -> Self {
            Self::with_array_length( LifetimeIndexArray::with_2(i0,i1), 2 )
        }
        pub const fn with_3(i0: LifetimeIndex,i1: LifetimeIndex,i2: LifetimeIndex) -> Self {
            Self::with_array_length( LifetimeIndexArray::with_3(i0,i1,i2), 3 )
        }

        pub const fn with_more_than_3(range:std::ops::Range<usize>)->Self{
            Self{
                bits: 
                     (range.start as u32)&Self::START_MASK
                    |((range.end as u32) << Self::LEN_OFFSET)
            }
        }

        pub const fn len(self) -> usize {
            (self.bits >> Self::LEN_OFFSET) as usize
        }
        const fn array_bits(self)-> u16{
            (self.bits as u16) & LifetimeIndexArray::MASK
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
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
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


)}