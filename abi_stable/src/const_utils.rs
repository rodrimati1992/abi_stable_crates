use std::{
    marker::PhantomData,
};


use core_extensions::prelude::*;


//////////////////////////////////////


/// Creates an empty slice.
pub const fn empty_slice<'a, T>() -> &'a [T]
where
    T: 'a,
{
    GetEmptySlice::<'a, T>::EMPTY
}

struct GetEmptySlice<'a, T>(&'a T);

impl<'a, T> GetEmptySlice<'a, T>
where
    T: 'a,
{
    const EMPTY: &'a [T] = &[];
}


//////////////////////////////////////


pub const fn min_u64(l:u64,r:u64)->u64{
    [r,l][ (l < r)as usize ]
}

pub const fn min_usize(l:usize,r:usize)->usize{
    [r,l][ (l < r)as usize ]
}

pub const fn max_u64(l:u64,r:u64)->u64{
    [l,r][ (l < r)as usize ]
}

pub const fn max_usize(l:usize,r:usize)->usize{
    [l,r][ (l < r)as usize ]
}



//////////////////////////////////////


/// Struct used to assert that its type parameters are the same type.
pub struct AssertEq<L,R>
where L:TypeIdentity<Type=R>
{
    _marker:PhantomData<(L,R)>
}

/// Allows transmuting between `From_` and `To`
pub union Transmuter<From_:Copy,To:Copy>{
    pub from:From_,
    pub to:To,
}

/// Allows converting between generic types that are the same concrete type 
/// (using AssertEq to prove that they are).
///
/// # Safety
///
/// This is safe to do,
/// since both types are required to be the same concrete type inside the macro.
#[macro_export]
macro_rules! type_identity {
    ($from:ty=>$to:ty; $expr:expr ) => {unsafe{
        let _:$crate::const_utils::AssertEq<$from,$to>;

        $crate::const_utils::Transmuter::<$from,$to>{ from:$expr }
            .to
    }}
}
