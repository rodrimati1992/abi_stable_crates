macro_rules! deref_coerced_impl_cmp_traits {
    (
        $Self:ty;
        coerce_to = $coerce_to:ty,
        [$($Rhs:ty),* $(,)?]
    ) => {
        const _: () = {
            use std::cmp::{PartialEq, PartialOrd, Ordering};

            $(
                #[allow(unused_lifetimes)]
                impl<'a, 'b> PartialEq<$Rhs> for $Self {
                    fn eq(&self, other: &$Rhs) -> bool {
                        <$coerce_to as PartialEq>::eq(self, other)
                    }
                }

                #[allow(unused_lifetimes)]
                impl<'a, 'b> PartialOrd<$Rhs> for $Self {
                    fn partial_cmp(&self, other: &$Rhs) -> Option<Ordering> {
                        <$coerce_to as PartialOrd>::partial_cmp(self, other)
                    }
                }

                #[allow(unused_lifetimes)]
                impl<'a, 'b> PartialEq<$Self> for $Rhs {
                    fn eq(&self, other: &$Self) -> bool {
                        <$coerce_to as PartialEq>::eq(self, other)
                    }
                }

                #[allow(unused_lifetimes)]
                impl<'a, 'b> PartialOrd<$Self> for $Rhs {
                    fn partial_cmp(&self, other: &$Self) -> Option<Ordering> {
                        <$coerce_to as PartialOrd>::partial_cmp(self, other)
                    }
                }
            )*
        };
    };
}

macro_rules! slice_like_impl_cmp_traits {
    (
        impl $impl_params:tt $Self:ty,
        where $where:tt;
        $($Rhs:ty),* $(,)?
    ) => {
        $(
            slice_like_impl_cmp_traits!{
                @inner
                impl $impl_params $Self,
                where $where;
                $Rhs
            }
        )*
    };
    (@inner
        impl[$($impl_params:tt)*] $Self:ty,
        where[$($where:tt)*];
        $Rhs:ty
    ) => {
        const _: () = {
            use std::cmp::{PartialEq, PartialOrd, Ordering};

            #[allow(unused_lifetimes)]
            impl<'a, 'b, T: PartialEq<U>, U, $($impl_params)*> PartialEq<$Rhs> for $Self
            where $($where)*
            {
                fn eq(&self, other: &$Rhs) -> bool {
                    <[T] as PartialEq<[U]>>::eq(self, other)
                }
            }

            #[allow(unused_lifetimes)]
            impl<T, U, $($impl_params)*> PartialOrd<$Rhs> for $Self
            where
                T: PartialOrd<U>,
                [T]: PartialOrd<[U]>,
                $($where)*
            {
                fn partial_cmp(&self, other: &$Rhs) -> Option<Ordering> {
                    <[T] as PartialOrd<[U]>>::partial_cmp(self, other)
                }
            }

            #[allow(unused_lifetimes)]
            impl<U: PartialEq<T>, T, $($impl_params)*> PartialEq<$Self> for $Rhs
            where $($where)*
            {
                fn eq(&self, other: &$Self) -> bool {
                    <[U] as PartialEq<[T]>>::eq(self, other)
                }
            }

            #[allow(unused_lifetimes)]
            impl<U, T, $($impl_params)*> PartialOrd<$Self> for $Rhs
            where
                U: PartialOrd<T>,
                [U]: PartialOrd<[T]>,
                $($where)*
            {
                fn partial_cmp(&self, other: &$Self) -> Option<Ordering> {
                    <[U] as PartialOrd<[T]>>::partial_cmp(self, other)
                }
            }
        };
    };
}

macro_rules! zst_assert {
    ($Self:ty) => {{
        ["Expected this to be Zero-sized"][(std::mem::size_of::<$Self>() != 0) as usize];
        ["Expected this to be 1 aligned"][(std::mem::align_of::<$Self>() != 1) as usize];

        ["Expected Tuple1<Self> to be Zero-sized"]
            [(std::mem::size_of::<crate::std_types::Tuple1<$Self>>() != 0) as usize];

        ["Expected Tuple1<Self> to be 1 aligned"]
            [(std::mem::align_of::<crate::std_types::Tuple1<$Self>>() != 1) as usize];

        ["Expected Tuple1<Self, Self> to be Zero-sized"]
            [(std::mem::size_of::<crate::std_types::Tuple2<$Self, $Self>>() != 0) as usize];

        ["Expected Tuple1<Self, Self> to be 1 aligned"]
            [(std::mem::align_of::<crate::std_types::Tuple2<$Self, $Self>>() != 1) as usize];
    }};
}

///////////////////////////////////////////////////////////////////////////////7
