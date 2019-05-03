use std::{
    marker::PhantomData,
};

use super::*;

use crate::{
    std_types::{RVec,ROption,Tuple2},
    marker_type::ErasedObject,
    utils::{transmute_reference,transmute_mut_reference},
    traits::IntoReprC,
};


///////////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct IteratorFns<Item>{
    pub(super) next       :extern fn(&mut ErasedObject)->ROption<Item>,
    pub(super) extend_buffer:
        extern fn(
            &mut ErasedObject,
            &mut RVec<Item>,
            ROption<usize>
        ),
    pub(super) size_hint  :extern fn(&    ErasedObject)-> Tuple2<usize, ROption<usize>>,    
    pub(super) count      :extern fn(&mut ErasedObject)->usize,
    pub(super) last       :extern fn(&mut ErasedObject)->ROption<Item>,
    pub(super) nth        :extern fn(&mut ErasedObject,usize)->ROption<Item>,
    pub(super) skip_eager :extern fn(&mut ErasedObject,usize),
}


impl<Item> Copy for IteratorFns<Item>{}
impl<Item> Clone for IteratorFns<Item>{
    fn clone(&self)->Self{
        *self
    }
}


///////////////////////////////////////////////////////////////////////////////////


pub struct MakeIteratorFns<I>(PhantomData<extern fn()->I>);

impl<I> MakeIteratorFns<I>
where I:Iterator
{
    pub(super) const NEW:IteratorFns<I::Item>=IteratorFns{
        next:next::<I>,
        extend_buffer:extend_buffer::<I>,
        size_hint:size_hint::<I>,
        count:count::<I>,
        last:last::<I>,
        nth:nth::<I>,
        skip_eager:skip_eager::<I>,
    };
}


///////////////////////////////////////////////////////////////////////////////////


pub(super) extern fn next<I>(this:&mut ErasedObject)->ROption<I::Item>
where I:Iterator
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,I>(this) };
        this.next().into_c()
    }
}

pub(super) extern fn extend_buffer<I>(
    this:&mut ErasedObject,
    vec:&mut RVec<I::Item>,
    taking:ROption<usize>,
)where 
    I:Iterator
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,I>(this) };

        vec.extend(
            this.take(taking.unwrap_or(!0))
        );
    }
}

pub(super) extern fn size_hint<I>(this:&ErasedObject)-> Tuple2<usize, ROption<usize>>
where I:Iterator
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_reference::<ErasedObject,I>(this) };
        let (l,r)=this.size_hint();

        Tuple2(l,r.into_c())
    }
}

pub(super) extern fn count<I>(this:&mut ErasedObject)->usize
where I:Iterator
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,I>(this) };
        this.count()
    }
}

pub(super) extern fn last<I>(this:&mut ErasedObject)->ROption<I::Item>
where I:Iterator
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,I>(this) };
        this.last().into_c()
    }
}

pub(super) extern fn nth<I>(this:&mut ErasedObject,at:usize)->ROption<I::Item>
where I:Iterator
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,I>(this) };
        this.nth(at).into_c()
    }
}

pub(super) extern fn skip_eager<I>(this:&mut ErasedObject,skipping:usize)
where I:Iterator
{
    extern_fn_panic_handling! {
        let this=unsafe{ transmute_mut_reference::<ErasedObject,I>(this) };

        if skipping!=0 {
            let _=this.nth(skipping-1);
        }
    }
}

