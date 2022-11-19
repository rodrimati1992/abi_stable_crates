use std::ptr;

use crate::{
    marker_type::ErasedObject,
    nonexhaustive_enum::{
        vtable::NonExhaustiveVtable_Ref, GetEnumInfo, NonExhaustive, SerializeEnum,
    },
    sabi_types::{RMut, RRef},
    std_types::{RBoxError, RCmpOrdering, ROption, RResult, RSome},
    traits::IntoReprC,
};

pub(crate) unsafe extern "C" fn drop_impl<E>(this: RMut<'_, ErasedObject>) {
    extern_fn_panic_handling! {no_early_return; unsafe {
        let this = this.transmute_into_mut::<E>();
        ptr::drop_in_place(this);
    }}
}

pub(crate) unsafe extern "C" fn clone_impl<E, F, I>(
    this: RRef<'_, ErasedObject>,
    vtable: NonExhaustiveVtable_Ref<E, F, I>,
) -> NonExhaustive<E, F, I>
where
    E: GetEnumInfo,
    E: Clone,
{
    extern_fn_panic_handling! {no_early_return; unsafe {
        let this = this.transmute_into_ref::<E>();
        NonExhaustive::with_vtable(this.clone(), vtable)
    }}
}

pub(crate) unsafe extern "C" fn partial_eq_impl<E, F, I>(
    this: RRef<'_, ErasedObject>,
    other: RRef<'_, ErasedObject>,
) -> bool
where
    E: GetEnumInfo + PartialEq,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_ref::<E>() };
        let other = unsafe { other.transmute_into_ref::<NonExhaustive<E,F,I>>() };
        match other.as_enum() {
            Ok(other)=>this==other,
            Err(_)=>false,
        }
    }
}

pub(crate) unsafe extern "C" fn cmp_ord<E, F, I>(
    this: RRef<'_, ErasedObject>,
    other: RRef<'_, ErasedObject>,
) -> RCmpOrdering
where
    E: GetEnumInfo + Ord,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_ref::<E>() };
        let other = unsafe { other.transmute_into_ref::<NonExhaustive<E,F,I>>() };

        match other.as_enum() {
            Ok(other)=>this.cmp(other).into_c(),
            Err(_)=>RCmpOrdering::Less,
        }
    }
}

pub(crate) unsafe extern "C" fn partial_cmp_ord<E, F, I>(
    this: RRef<'_, ErasedObject>,
    other: RRef<'_, ErasedObject>,
) -> ROption<RCmpOrdering>
where
    E: GetEnumInfo + PartialOrd,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_ref::<E>() };
        let other = unsafe { other.transmute_into_ref::<NonExhaustive<E,F,I>>() };

        match other.as_enum() {
            Ok(other)=>this.partial_cmp(other).map(IntoReprC::into_c).into_c(),
            Err(_)=>RSome(RCmpOrdering::Less),
        }
    }
}

pub(crate) unsafe extern "C" fn serialize_impl<E, I>(
    this: RRef<'_, ErasedObject>,
) -> RResult<<I as SerializeEnum<E>>::Proxy, RBoxError>
where
    I: SerializeEnum<E>,
{
    extern_fn_panic_handling! {no_early_return;
        let this = unsafe { this.transmute_into_ref::<E>() };
        I::serialize_enum(this).into()
    }
}
