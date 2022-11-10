#[doc(hidden)]
#[macro_export]
macro_rules! declare_type_layout_index {(
    attrs=[ $($extra_attrs:meta),* $(,)* ]
) => (
    /// An index for a slice of type layouts.
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone)]
    $(#[$extra_attrs])*
    pub struct TypeLayoutIndex{
        bits:u16
    }

    impl TypeLayoutIndex{
        pub(crate) const MASK:u16=0b11_1111_1111;
        /// The amount of bits required to represent a `TypeLayoutIndex`.
        pub const BIT_SIZE:u32=10;

        /// The maximum value of a `TypeLayoutIndex`.
        pub const MAX_VAL_U16:u16=Self::MASK;

        /// The maximum value of a `TypeLayoutIndex`.
        pub const MAX_VAL:usize=Self::MAX_VAL_U16 as usize;

        /// Constructs this `TypeLayoutIndex` from its representation.
        #[inline]
        pub const fn from_u10(n:u16)->Self{
            Self{bits: n & Self::MASK }
        }

        /// Converts this `TypeLayoutIndex` into its representation.
        #[inline]
        pub const fn to_u10(self)->u16{
            self.bits & Self::MASK
        }

        #[inline]
        pub(crate) const fn mask_off(n:u16)->u16{
            n & Self::MASK
        }


    }

    mod type_layout_index_impls{
        use super::*;
        use std::fmt::{self,Display};

        impl Display for TypeLayoutIndex{
            #[inline]
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
                Display::fmt(&self.bits,f)
            }
        }
    }


)}
