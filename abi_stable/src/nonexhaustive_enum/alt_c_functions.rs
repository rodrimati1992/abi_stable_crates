use std::ptr;

use crate::{
    marker_type::ErasedObject,
    nonexhaustive_enum::{GetEnumInfo, NonExhaustive, NonExhaustiveVtable_Ref, SerializeEnum},
    sabi_types::{RMut, RRef},
    std_types::{RBoxError, RCmpOrdering, ROption, RResult, RSome},
    traits::IntoReprC,
};

pub(crate) unsafe extern "C" fn drop_impl<E>(this: RMut<'_, ErasedObject>) {
    extern_fn_panic_handling! {
        let this = this.transmute_into_mut::<E>();
        ptr::drop_in_place(this);
    }
}

pub(crate) unsafe extern "C" fn clone_impl<E, F, I>(
    this: RRef<'_, ErasedObject>,
    vtable: NonExhaustiveVtable_Ref<E, F, I>,
) -> NonExhaustive<E, F, I>
where
    E: GetEnumInfo,
    E: Clone,
{
    extern_fn_panic_handling! {
        let this = this.transmute_into_ref::<E>();
        let clone=this.clone();
        NonExhaustive::with_vtable(clone,vtable)
    }
}

pub(crate) unsafe extern "C" fn partial_eq_impl<E, F, I>(
    this: RRef<'_, ErasedObject>,
    other: RRef<'_, ErasedObject>,
) -> bool
where
    E: GetEnumInfo + PartialEq,
{
    extern_fn_panic_handling! {
        let this = this.transmute_into_ref::<E>();
        let other = other.transmute_into_ref::<NonExhaustive<E,F,I>>();
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
    extern_fn_panic_handling! {
        let this = this.transmute_into_ref::<E>();
        let other = other.transmute_into_ref::<NonExhaustive<E,F,I>>();

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
    extern_fn_panic_handling! {
        let this = this.transmute_into_ref::<E>();
        let other = other.transmute_into_ref::<NonExhaustive<E,F,I>>();

        match other.as_enum() {
            Ok(other)=>this.partial_cmp(other).map(IntoReprC::into_c).into_c(),
            Err(_)=>RSome(RCmpOrdering::Less),
        }
    }
}

pub(crate) unsafe extern "C" fn serialize_impl<NE, I>(
    this: RRef<'_, ErasedObject>,
) -> RResult<<I as SerializeEnum<NE>>::Proxy, RBoxError>
where
    I: SerializeEnum<NE>,
{
    extern_fn_panic_handling! {
        let this = this.transmute_into_ref::<NE>();
        I::serialize_enum(this).into()
    }
}
