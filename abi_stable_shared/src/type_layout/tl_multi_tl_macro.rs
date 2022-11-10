#[doc(hidden)]
#[macro_export]
macro_rules! declare_multi_tl_types {(
    attrs=[ $($extra_attrs:meta),* $(,)* ]
) => (

    /// A range of indices into a slice of type layouts
    /// which can store up to five indices inline,
    /// requiring additional layouts to be stored contiguously after the
    /// fourth one in the slice.
    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
    $(#[ $extra_attrs ])*
    pub struct TypeLayoutRange{
        bits0:u32,
        bits1:u32,
    }

    impl TypeLayoutRange{
        /// An empty `TypeLayoutRange`.
        pub const EMPTY:Self=Self{
            bits0:0,
            bits1:0,
        };

        const LEN_MASK:u32=0b11_1111_1111;
        const INDEX_MASK:u32=0x3FF;

        const LEN_BIT_SIZE:u32=10;

        /// How many indices can be stored inline.
        pub const STORED_INLINE:usize=5;

        const INDEX_0_OFFSET:u32=Self::LEN_BIT_SIZE;
        const INDEX_1_OFFSET:u32=Self::LEN_BIT_SIZE+TypeLayoutIndex::BIT_SIZE;

        const INDEX_2_OFFSET:u32=0;
        const INDEX_3_OFFSET:u32=TypeLayoutIndex::BIT_SIZE;
        const INDEX_4_OFFSET:u32=TypeLayoutIndex::BIT_SIZE*2;

        fn size_assertions(){
            let _:[(); 32-(Self::LEN_BIT_SIZE+TypeLayoutIndex::BIT_SIZE*2)as usize ];
        }

        /// Constructs a `TypeLayoutRange` with up to five type layout indices.
        #[inline]
        pub const fn with_up_to_5(mut len:usize,indices:[u16;5]) -> Self {
            let len=len & 0usize.wrapping_sub((len <= Self::STORED_INLINE) as usize);
            Self::with_more_than_5(len,indices)
        }

        /// Constructs a `TypeLayoutRange` with more
        /// than five type layout indices,
        /// in which the indices from `indices[4]` onwards are
        /// stored contiguously in the slice.
        #[inline]
        pub const fn with_more_than_5(len:usize,indices:[u16;5]) -> Self {
            Self{
                bits0:len as u32
                    |((indices[0] as u32 & Self::INDEX_MASK) << Self::INDEX_0_OFFSET)
                    |((indices[1] as u32 & Self::INDEX_MASK) << Self::INDEX_1_OFFSET),
                bits1:
                     ((indices[2] as u32 & Self::INDEX_MASK) << Self::INDEX_2_OFFSET)
                    |((indices[3] as u32 & Self::INDEX_MASK) << Self::INDEX_3_OFFSET)
                    |((indices[4] as u32 & Self::INDEX_MASK) << Self::INDEX_4_OFFSET),
            }
        }

        /// Constructs this `TypeLayoutRange` from its representation.
        #[inline]
        pub const fn from_u64(bits:u64) -> Self {
            Self{
                bits0:bits as u32,
                bits1:(bits>>32)as u32,
            }
        }

        /// Converts this `TypeLayoutRange` into its representation.
        #[inline]
        pub const fn to_u64(&self) -> u64 {
             (self.bits0 as u64)
            |((self.bits1 as u64) << 32)
        }

        /// The amount of type layouts in this range.
        #[inline]
        pub const fn len(&self) -> usize {
            (self.bits0 & Self::LEN_MASK) as usize
        }

        /// Whether this range of type layouts is empty.
        #[inline]
        pub const fn is_empty(&self) -> bool {
            self.len() == 0
        }
    }


    ///////////////////////////////////////////////////////////////////////////////

)}
