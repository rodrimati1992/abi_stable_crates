use std::{
    marker::PhantomData,
};



use crate::{
    const_utils::Transmuter,
    std_types::{RVec,ROption,RSome,RNone,Tuple2},
    marker_type::ErasedObject,
    utils::{transmute_reference,transmute_mut_reference},
    traits::IntoReprC,
};


///////////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(StableAbi)]
pub struct IteratorFns<Item>{
    pub(super) next       :unsafe extern fn(&mut ErasedObject)->ROption<Item>,
    pub(super) extending_rvec:
        unsafe extern fn(
            &mut ErasedObject,
            &mut RVec<Item>,
            ROption<usize>
        ),
    pub(super) size_hint  :unsafe extern fn(&    ErasedObject)-> Tuple2<usize, ROption<usize>>,    
    pub(super) count      :unsafe extern fn(&mut ErasedObject)->usize,
    pub(super) last       :unsafe extern fn(&mut ErasedObject)->ROption<Item>,
    pub(super) nth        :unsafe extern fn(&mut ErasedObject,usize)->ROption<Item>,
    pub(super) skip_eager :unsafe extern fn(&mut ErasedObject,usize),
}


impl<Item> Copy for IteratorFns<Item>{}
impl<Item> Clone for IteratorFns<Item>{
    fn clone(&self)->Self{
        *self
    }
}


///////////////////////////////////////////////////////////////////////////////////


pub struct MakeIteratorFns<I>(PhantomData<unsafe extern fn()->I>);

impl<I> MakeIteratorFns<I>
where I:Iterator
{
    const ITER:IteratorFns<I::Item>=IteratorFns{
        next:next::<I>,
        extending_rvec:extending_rvec::<I>,
        size_hint:size_hint::<I>,
        count:count::<I>,
        last:last::<I>,
        nth:nth::<I>,
        skip_eager:skip_eager::<I>,
    };

    pub(super) const NEW:IteratorFns<()>=unsafe{
        Transmuter{
            from:Self::ITER
        }.to
    };
}


///////////////////////////////////////////////////////////////////////////////////


pub(super) unsafe extern fn next<I>(this:&mut ErasedObject)->ROption<I::Item>
where I:Iterator
{
    extern_fn_panic_handling! {
        let this=transmute_mut_reference::<ErasedObject,I>(this);
        this.next().into_c()
    }
}

pub(super) unsafe extern fn extending_rvec<I>(
    this:&mut ErasedObject,
    vec:&mut RVec<I::Item>,
    taking:ROption<usize>,
)where 
    I:Iterator
{
    extern_fn_panic_handling! {
        let this=transmute_mut_reference::<ErasedObject,I>(this);

        vec.extend(
            this.take(taking.unwrap_or(!0))
        );
    }
}

pub(super) unsafe extern fn size_hint<I>(this:&ErasedObject)-> Tuple2<usize, ROption<usize>>
where I:Iterator
{
    extern_fn_panic_handling! {
        let this=transmute_reference::<ErasedObject,I>(this);
        let (l,r)=this.size_hint();

        Tuple2(l,r.into_c())
    }
}

pub(super) unsafe extern fn count<I>(this:&mut ErasedObject)->usize
where I:Iterator
{
    extern_fn_panic_handling! {
        let this=transmute_mut_reference::<ErasedObject,I>(this);
        this.count()
    }
}

pub(super) unsafe extern fn last<I>(this:&mut ErasedObject)->ROption<I::Item>
where I:Iterator
{
    extern_fn_panic_handling! {
        let this=transmute_mut_reference::<ErasedObject,I>(this);
        this.last().into_c()
    }
}

pub(super) unsafe extern fn nth<I>(this:&mut ErasedObject,at:usize)->ROption<I::Item>
where I:Iterator
{
    extern_fn_panic_handling! {
        let this=transmute_mut_reference::<ErasedObject,I>(this);
        this.nth(at).into_c()
    }
}

pub(super) unsafe extern fn skip_eager<I>(this:&mut ErasedObject,skipping:usize)
where I:Iterator
{
    extern_fn_panic_handling! {
        let this=transmute_mut_reference::<ErasedObject,I>(this);

        if skipping!=0 {
            let _=this.nth(skipping-1);
        }
    }
}



////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////



#[repr(C)]
#[derive(StableAbi)]
pub struct DoubleEndedIteratorFns<Item>{
    pub(super) next_back       :unsafe extern fn(&mut ErasedObject)->ROption<Item>,
    pub(super) extending_rvec_back:
        unsafe extern fn(
            &mut ErasedObject,
            &mut RVec<Item>,
            ROption<usize>
        ),
    pub(super) nth_back:unsafe extern fn(&mut ErasedObject,usize)->ROption<Item>,
}


impl<Item> Copy for DoubleEndedIteratorFns<Item>{}
impl<Item> Clone for DoubleEndedIteratorFns<Item>{
    fn clone(&self)->Self{
        *self
    }
}


///////////////////////////////////////////////////////////////////////////////////


pub struct MakeDoubleEndedIteratorFns<I>(PhantomData<unsafe extern fn()->I>);

impl<I> MakeDoubleEndedIteratorFns<I>
where I:DoubleEndedIterator
{
    pub(super) const ITER:DoubleEndedIteratorFns<I::Item>=DoubleEndedIteratorFns{
        next_back:next_back::<I>,
        extending_rvec_back:extending_rvec_back::<I>,
        nth_back:nth_back::<I>,
    };

    pub(super) const NEW:DoubleEndedIteratorFns<()>=unsafe{
        Transmuter{
            from:Self::ITER
        }.to
    };
}


///////////////////////////////////////////////////////////////////////////////////


pub(super) unsafe extern fn next_back<I>(this:&mut ErasedObject)->ROption<I::Item>
where 
    I:DoubleEndedIterator
{
    extern_fn_panic_handling! {
        let this=transmute_mut_reference::<ErasedObject,I>(this);
        this.next_back().into_c()
    }
}

pub(super) unsafe extern fn extending_rvec_back<I>(
    this:&mut ErasedObject,
    vec:&mut RVec<I::Item>,
    taking:ROption<usize>
)where 
    I:DoubleEndedIterator
{
    extern_fn_panic_handling! {
        let this=transmute_mut_reference::<ErasedObject,I>(this);

        vec.extend(
            this.rev().take(taking.unwrap_or(!0))
        );
    }
}

pub(super) unsafe extern fn nth_back<I>(this:&mut ErasedObject,mut at:usize)->ROption<I::Item>
where 
    I:DoubleEndedIterator
{
    extern_fn_panic_handling! {
        let this=transmute_mut_reference::<ErasedObject,I>(this);
        for x in this.rev() {
            if at == 0 { 
                return RSome(x) 
            }
            at -= 1;
        }
        RNone
    }
}
