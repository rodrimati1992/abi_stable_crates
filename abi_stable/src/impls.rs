macro_rules! shared_impls {
    (pointer
        mod=$mod_:ident
        new_type=$tconst:ident[$($lt:lifetime),*][$($ty:ident),*]
            $(extra[$($ex_ty:ident),* $(,)* ])?
            $(where [ $($where_:tt)* ])? ,
        original_type=$original:ident,
    ) => {

        mod $mod_{
            use super::*;

            use std::{
                fmt::{self,Display, Pointer as PointerFmt},
                ops::Deref,
            };

            use serde::{Deserialize,Serialize,Deserializer,Serializer};

            impl<$($lt,)* $($ty,)* $($($ex_ty,)*)?> Deref
                for $tconst<$($lt,)* $($ty,)* $($($ex_ty,)*)?>
            {
                type Target=T;

                fn deref(&self)->&Self::Target{
                    unsafe{
                        &*self.data()
                    }
                }
            }

            impl<$($lt,)* $($ty,)* $($($ex_ty,)*)?> Display
                for $tconst<$($lt,)* $($ty,)* $($($ex_ty,)*)?>
            where
                $($ty:Display,)*
            {
                fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
                    Display::fmt(&**self,f)
                }
            }

            impl<$($lt,)* $($ty,)* $($($ex_ty,)*)?> PointerFmt
                for $tconst<$($lt,)* $($ty,)* $($($ex_ty,)*)?>
            {
                fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
                    let ptr: *const _ = &**self;
                    PointerFmt::fmt(&ptr, f)
                }
            }


            impl<'de,$($lt,)* $($ty,)* $($($ex_ty,)*)?> Deserialize<'de>
                for $tconst<$($lt,)* $($ty,)* $($($ex_ty,)*)?>
            where
                T:Deserialize<'de>,
            {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>
                {
                    T::deserialize(deserializer)
                        .map(Self::new)
                }
            }

            impl<$($lt,)* $($ty,)* $($($ex_ty,)*)?> Serialize
                for $tconst<$($lt,)* $($ty,)* $($($ex_ty,)*)?>
            where
                T:Serialize
            {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    (&**self).serialize(serializer)
                }
            }

            shared_impls!{
                mod=$mod_
                new_type=$tconst[$($lt),*][$($ty),*]
                    $(extra[$($ex_ty),*])?
                    $(where [ $($where_)* ])? ,
                original_type=$original,
            }
        }

    };
    (
        mod=$mod_:ident
        new_type=$tconst:ident[$($lt:lifetime),*][$($ty:ident),*]
            $(extra[$($ex_ty:ident),* $(,)*])?
            $(constrained[$($c_ty:ty),* $(,)*])?
            $(where [ $($where_:tt)* ])? ,
        original_type=$original:ident,
        $(deref_approach=$deref:tt,)?
    ) => {
        mod $mod_{
            use std::{
                cmp::{PartialEq,Eq,Ord,PartialOrd,Ordering},
                fmt::{self,Debug},
                hash::{Hash,Hasher},
            };

            use super::*;
            impl<$($lt,)* $($ty,)* $($($ex_ty,)*)?> Debug
                for $tconst<$($lt,)* $($ty,)* $($($ex_ty,)*)?>
            where
                $($ty:Debug,)*
                $($($c_ty:Debug,)*)?
                $($($where_)*)?
            {
                fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
                    Debug::fmt(si_deref!($($deref)? self),f)
                }
            }

            impl<$($lt,)* $($ty,)* $($($ex_ty,)*)?> Eq
                for $tconst<$($lt,)* $($ty,)* $($($ex_ty,)*)?>
            where
                $($ty:Eq,)*
                $($($c_ty:Eq,)*)?
                $($($where_)*)?
            {}


            impl<$($lt,)* $($ty,)* $($($ex_ty,)*)?> PartialEq
                for $tconst<$($lt,)* $($ty,)* $($($ex_ty,)*)?>
            where
                $($ty:PartialEq,)*
                $($($c_ty:PartialEq,)*)?
                $($($where_)*)?
            {
                fn eq(&self, other: &Self) -> bool{
                    ::std::ptr::eq(si_deref!($($deref)? self),si_deref!($($deref)? other))||
                    si_deref!($($deref)? self) == si_deref!($($deref)? other)
                }
            }


            impl<$($lt,)* $($ty,)* $($($ex_ty,)*)?> Ord
                for $tconst<$($lt,)* $($ty,)* $($($ex_ty,)*)?>
            where
                $($ty:Ord,)*
                $($($c_ty:Ord,)*)?
                $($($where_)*)?
            {
                fn cmp(&self, other: &Self) -> Ordering{
                    if ::std::ptr::eq(si_deref!($($deref)? self),si_deref!($($deref)? other)) {
                        return Ordering::Equal;
                    }
                    si_deref!($($deref)? self).cmp(si_deref!($($deref)? other))
                }
            }


            impl<$($lt,)* $($ty,)* $($($ex_ty,)*)?> PartialOrd
                for $tconst<$($lt,)* $($ty,)* $($($ex_ty,)*)?>
            where
                $($ty:PartialOrd,)*
                $($($c_ty:PartialOrd,)*)?
                $($($where_)*)?
            {
                fn partial_cmp(&self, other: &Self) -> Option<Ordering>{
                    if ::std::ptr::eq(si_deref!($($deref)? self),si_deref!($($deref)? other)) {
                        return Some(Ordering::Equal);
                    }
                    si_deref!($($deref)? self).partial_cmp(si_deref!($($deref)? other))
                }
            }


            impl<$($lt,)* $($ty,)* $($($ex_ty,)*)?> Hash
                for $tconst<$($lt,)* $($ty,)* $($($ex_ty,)*)?>
            where
                $($ty:Hash,)*
                $($($c_ty:Hash,)*)?
                $($($where_)*)?
            {
                fn hash<H>(&self, state: &mut H)
                where
                    H: Hasher
                {
                    si_deref!($($deref)? self).hash(state)
                }
            }
        }
    };
}

macro_rules! si_deref {
    ($self:ident) => {
        &**$self
    };
    (double_deref $self:ident) => {
        &**$self
    };
    ((method = $method:ident) $self:ident) => {
        $self.$method()
    };
}
