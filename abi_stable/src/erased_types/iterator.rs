use crate::{
    marker_type::{ErasedObject, NonOwningPhantom},
    sabi_types::{RMut, RRef},
    std_types::{RNone, ROption, RSome, RVec, Tuple2},
    traits::IntoReprC,
    utils::Transmuter,
};

///////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
pub struct IteratorFns<Item> {
    pub(super) next: unsafe extern "C" fn(RMut<'_, ErasedObject>) -> ROption<Item>,
    pub(super) extending_rvec:
        unsafe extern "C" fn(RMut<'_, ErasedObject>, &mut RVec<Item>, ROption<usize>),
    pub(super) size_hint:
        unsafe extern "C" fn(RRef<'_, ErasedObject>) -> Tuple2<usize, ROption<usize>>,
    pub(super) count: unsafe extern "C" fn(RMut<'_, ErasedObject>) -> usize,
    pub(super) last: unsafe extern "C" fn(RMut<'_, ErasedObject>) -> ROption<Item>,
    pub(super) nth: unsafe extern "C" fn(RMut<'_, ErasedObject>, usize) -> ROption<Item>,
    pub(super) skip_eager: unsafe extern "C" fn(RMut<'_, ErasedObject>, usize),
}

impl<Item> Copy for IteratorFns<Item> {}
impl<Item> Clone for IteratorFns<Item> {
    fn clone(&self) -> Self {
        *self
    }
}

///////////////////////////////////////////////////////////////////////////////////

pub struct MakeIteratorFns<I>(NonOwningPhantom<I>);

impl<I> MakeIteratorFns<I>
where
    I: Iterator,
{
    const ITER: IteratorFns<I::Item> = IteratorFns {
        next: next::<I>,
        extending_rvec: extending_rvec::<I>,
        size_hint: size_hint::<I>,
        count: count::<I>,
        last: last::<I>,
        nth: nth::<I>,
        skip_eager: skip_eager::<I>,
    };

    pub(super) const NEW: IteratorFns<()> = unsafe { Transmuter { from: Self::ITER }.to };
}

///////////////////////////////////////////////////////////////////////////////////

pub(super) unsafe extern "C" fn next<I>(this: RMut<'_, ErasedObject>) -> ROption<I::Item>
where
    I: Iterator,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<I>() };
        this.next().into_c()
    }
}

pub(super) unsafe extern "C" fn extending_rvec<I>(
    this: RMut<'_, ErasedObject>,
    vec: &mut RVec<I::Item>,
    taking: ROption<usize>,
) where
    I: Iterator,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<I>() };

        vec.extend(
            this.take(taking.unwrap_or(!0))
        );
    }
}

pub(super) unsafe extern "C" fn size_hint<I>(
    this: RRef<'_, ErasedObject>,
) -> Tuple2<usize, ROption<usize>>
where
    I: Iterator,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_ref::<I>() };
        let (l,r)=this.size_hint();

        Tuple2(l,r.into_c())
    }
}

pub(super) unsafe extern "C" fn count<I>(this: RMut<'_, ErasedObject>) -> usize
where
    I: Iterator,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<I>() };
        this.count()
    }
}

pub(super) unsafe extern "C" fn last<I>(this: RMut<'_, ErasedObject>) -> ROption<I::Item>
where
    I: Iterator,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<I>() };
        this.last().into_c()
    }
}

pub(super) unsafe extern "C" fn nth<I>(this: RMut<'_, ErasedObject>, at: usize) -> ROption<I::Item>
where
    I: Iterator,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<I>() };
        this.nth(at).into_c()
    }
}

pub(super) unsafe extern "C" fn skip_eager<I>(this: RMut<'_, ErasedObject>, skipping: usize)
where
    I: Iterator,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<I>() };

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
pub struct DoubleEndedIteratorFns<Item> {
    pub(super) next_back: unsafe extern "C" fn(RMut<'_, ErasedObject>) -> ROption<Item>,
    pub(super) extending_rvec_back:
        unsafe extern "C" fn(RMut<'_, ErasedObject>, &mut RVec<Item>, ROption<usize>),
    pub(super) nth_back: unsafe extern "C" fn(RMut<'_, ErasedObject>, usize) -> ROption<Item>,
}

impl<Item> Copy for DoubleEndedIteratorFns<Item> {}
impl<Item> Clone for DoubleEndedIteratorFns<Item> {
    fn clone(&self) -> Self {
        *self
    }
}

///////////////////////////////////////////////////////////////////////////////////

pub struct MakeDoubleEndedIteratorFns<I>(NonOwningPhantom<I>);

impl<I> MakeDoubleEndedIteratorFns<I>
where
    I: DoubleEndedIterator,
{
    pub(super) const ITER: DoubleEndedIteratorFns<I::Item> = DoubleEndedIteratorFns {
        next_back: next_back::<I>,
        extending_rvec_back: extending_rvec_back::<I>,
        nth_back: nth_back::<I>,
    };

    pub(super) const NEW: DoubleEndedIteratorFns<()> =
        unsafe { Transmuter { from: Self::ITER }.to };
}

///////////////////////////////////////////////////////////////////////////////////

pub(super) unsafe extern "C" fn next_back<I>(this: RMut<'_, ErasedObject>) -> ROption<I::Item>
where
    I: DoubleEndedIterator,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<I>() };
        this.next_back().into_c()
    }
}

pub(super) unsafe extern "C" fn extending_rvec_back<I>(
    this: RMut<'_, ErasedObject>,
    vec: &mut RVec<I::Item>,
    taking: ROption<usize>,
) where
    I: DoubleEndedIterator,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_mut::<I>() };

        vec.extend(
            this.rev().take(taking.unwrap_or(!0))
        );
    }
}

pub(super) unsafe extern "C" fn nth_back<I>(
    this: RMut<'_, ErasedObject>,
    mut at: usize,
) -> ROption<I::Item>
where
    I: DoubleEndedIterator,
{
    extern_fn_panic_handling! { // returns early
        let this = unsafe { this.transmute_into_mut::<I>() };
        for x in this.rev() {
            if at == 0 {
                return RSome(x)
            }
            at -= 1;
        }
        RNone
    }
}
