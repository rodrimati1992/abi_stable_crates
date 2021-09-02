macro_rules! deref_coerced_impl_cmp_traits {
    (
        $Self:ty;
        coerce_to = $coerce_to:ty,
        [$($Rhs:ty),* $(,)?]
    ) => {
        const _: () = {
            use std::cmp::{PartialEq, PartialOrd, Ordering};

            $(

                impl PartialEq<$Rhs> for $Self {
                    fn eq(&self, other: &$Rhs) -> bool {
                        <$coerce_to as PartialEq>::eq(self, other)
                    }
                }

                impl PartialOrd<$Rhs> for $Self {
                    fn partial_cmp(&self, other: &$Rhs) -> Option<Ordering> {
                        <$coerce_to as PartialOrd>::partial_cmp(self, other)
                    }
                }

                impl PartialEq<$Self> for $Rhs {
                    fn eq(&self, other: &$Self) -> bool {
                        <$coerce_to as PartialEq>::eq(self, other)
                    }
                }

                impl PartialOrd<$Self> for $Rhs {
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
        $Self:ty,
        where $where:tt;
        $($Rhs:ty),* $(,)?
    ) => {
        $(
            slice_like_impl_cmp_traits!{
                @inner
                $Self,
                where $where;
                $Rhs
            }
        )*
    };
    (@inner
        $Self:ty,
        where[$($where:tt)*];
        $Rhs:ty
    ) => {
        const _: () = {
            use std::cmp::{PartialEq, PartialOrd, Ordering};

            impl<T: PartialEq<U>, U> PartialEq<$Rhs> for $Self 
            where $($where)* 
            {
                fn eq(&self, other: &$Rhs) -> bool {
                    <[T] as PartialEq<[U]>>::eq(self, other)
                }
            }

            impl<T, U> PartialOrd<$Rhs> for $Self 
            where
                T: PartialOrd<U>,
                [T]: PartialOrd<[U]>,
                $($where)*
            {
                fn partial_cmp(&self, other: &$Rhs) -> Option<Ordering> {
                    <[T] as PartialOrd<[U]>>::partial_cmp(self, other)
                }
            }

            impl<U: PartialEq<T>, T> PartialEq<$Self> for $Rhs 
            where $($where)* 
            {
                fn eq(&self, other: &$Self) -> bool {
                    <[U] as PartialEq<[T]>>::eq(self, other)
                }
            }

            impl<U, T> PartialOrd<$Self> for $Rhs 
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