#[doc(hidden)]
#[macro_export]
macro_rules! declare_multi_tl_types {( 
    attrs=[ $($extra_attrs:meta),* $(,)* ]
) => (

    /// Encodes up to 4 u13 indices + u6 length inline,
    /// using the last inline index as the start of a slice if the length is greater than 4.
    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
    $(#[ $extra_attrs ])*
    pub struct TypeLayoutRange{
        bits0:u32,
        bits1:u32,
    }

    impl TypeLayoutRange{
        pub const EMPTY:Self=Self{
            bits0:0,
            bits1:0,
        };

        pub const LEN_MASK:u32=0b11_1111;
        pub const INDEX_MASK:u32=0x1FFF;
        pub const INDEX_BIT_SIZE:u32=13;

        pub const LEN_BIT_SIZE:u32=6;

        pub const INDEX_0_OFFSET:u32=Self::LEN_BIT_SIZE;
        pub const INDEX_1_OFFSET:u32=Self::INDEX_0_OFFSET+Self::INDEX_BIT_SIZE;

        pub const INDEX_2_OFFSET:u32=0;
        pub const INDEX_3_OFFSET:u32=Self::INDEX_BIT_SIZE;
        
        fn size_assertions(){
            let _:[(); 32-(Self::LEN_BIT_SIZE+Self::INDEX_BIT_SIZE*2)as usize ];
        }

        #[inline]
        pub const fn with_1(index0:u16)->Self{
            Self{
                bits0:1|((index0 as u32)<<Self::INDEX_0_OFFSET),
                bits1:0
            }
        }

        #[inline]
        pub const fn with_2(index0:u16,index1:u16)->Self{
            Self{
                bits0:2
                    |((index0 as u32 & Self::INDEX_MASK)<<Self::INDEX_0_OFFSET)
                    |((index1 as u32 & Self::INDEX_MASK)<<Self::INDEX_1_OFFSET),
                bits1:0
            }
        }
        
        #[inline]
        pub const fn with_3(index0:u16,index1:u16,index2:u16)->Self{
            Self{
                bits0:3
                    |((index0 as u32 & Self::INDEX_MASK)<<Self::INDEX_0_OFFSET)
                    |((index1 as u32 & Self::INDEX_MASK)<<Self::INDEX_1_OFFSET),
                bits1:
                    (index2 as u32 & Self::INDEX_MASK),
            }
        }
        
        #[inline]
        pub const fn with_4(index0:u16,index1:u16,index2:u16,index3:u16)->Self{
            Self::with_more_than_4(4,index0,index1,index2,index3)
        }
        
        #[inline]
        pub const fn with_up_to_4(mut len:usize,i0:u16,i1:u16,i2:u16,i3:u16)->Self{
            let len=len & 0usize.wrapping_sub((len <= 4) as usize);
            Self::with_more_than_4(len,i0,i1,i2,i3)
        }

        #[inline]
        pub const fn with_more_than_4(len:usize,i0:u16,i1:u16,i2:u16,i3_plus:u16)->Self{
            Self{
                bits0:len as u32
                    |((i0 as u32 & Self::INDEX_MASK)<<Self::INDEX_0_OFFSET)
                    |((i1 as u32 & Self::INDEX_MASK)<<Self::INDEX_1_OFFSET),
                bits1:
                     ((i2 as u32 & Self::INDEX_MASK) << Self::INDEX_2_OFFSET)
                    |((i3_plus as u32 & Self::INDEX_MASK) << Self::INDEX_3_OFFSET),
            }
        }

        #[inline]
        pub const fn from_u64(bits:u64)->Self{
            Self{
                bits0:bits as u32,
                bits1:(bits>>32)as u32,
            }
        }

        #[inline]
        pub const fn to_u64(&self)->u64{
             (self.bits0 as u64)
            |((self.bits1 as u64) << 32)
        }

        #[inline]
        pub const fn len(&self)->usize{
            (self.bits0&Self::LEN_MASK) as usize
        }
    }


    ///////////////////////////////////////////////////////////////////////////////

)}

