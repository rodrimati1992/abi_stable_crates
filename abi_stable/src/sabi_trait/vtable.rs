use super::*;

use crate::{
    erased_types::{FormattingMode,InterfaceBound,VTableDT},
    type_level::{
        unerasability::{GetUTID},
        impl_enum::{Implemented,Unimplemented},
        trait_marker,
    },
    sabi_types::Constructor,
    std_types::{UTypeId,RResult,RString,Tuple3},
};

/// Gets the vtable of a trait object.
///
/// # Safety
///
/// The vtable of the type implementing this trait must have
/// `StaticRef<RObjectVtable<_Self,ErasedPtr,I>>` as its first declared field.
pub unsafe trait GetVTable<IA,_Self,ErasedPtr,OrigPtr,Params>{
    type VTable;
    fn get_vtable()->StaticRef<Self::VTable>;
}


//////////////////////////////////////////////////////////////////////////////

/// Gets an `RObjectVtable<_Self,ErasedPtr,TO>`,
/// which is stored as the first field of all generated trait object vtables.
pub unsafe trait GetRObjectVTable<IA,_Self,ErasedPtr,OrigPtr>:Sized+InterfaceType{
    const ROBJECT_VTABLE:StaticRef<RObjectVtable<_Self,ErasedPtr,Self>>;
}




unsafe impl<IA,_Self,ErasedPtr,OrigPtr,I> 
    GetRObjectVTable<IA,_Self,ErasedPtr,OrigPtr> for I
where
    I:AreTraitsImpld<IA,_Self,ErasedPtr,OrigPtr>+InterfaceType,
{
    const ROBJECT_VTABLE:StaticRef<RObjectVtable<_Self,ErasedPtr,Self>>={
        let prefix=GetRObjectVTableHelper::<IA,_Self,ErasedPtr,OrigPtr,I>::TMP_VTABLE;
        WithMetadata::staticref_as_prefix(prefix)
    };
}


//////////////////////////////////////////////////////////////////////////////

#[allow(non_camel_case_types)]
pub type VTableTO_RO<T,OrigPtr,Unerasability,V>=VTableTO<T,OrigPtr,Unerasability,V,()>;

#[allow(non_camel_case_types)]
pub type VTableTO_DT<'borr,_Self,ErasedPtr,OrigPtr,I,Unerasability,V>=
    VTableTO<
        _Self,
        OrigPtr,
        Unerasability,
        V,
        VTableDT<'borr,_Self,ErasedPtr,OrigPtr,I,Unerasability>
    >;



/// This is used to safely pass the vtable to `#[sabi_trait]` generated trait objects.
pub struct VTableTO<_Self,OrigPtr,Unerasability,V,DT>{
    vtable:StaticRef<V>,
    for_dyn_trait:DT,
    _for:PhantomData<Constructor<Tuple3<_Self,OrigPtr,Unerasability>>>,
}

impl<_Self,OrigPtr,Unerasability,V,DT> Copy for VTableTO<_Self,OrigPtr,Unerasability,V,DT>
where DT:Copy
{}

impl<_Self,OrigPtr,Unerasability,V,DT> Clone for VTableTO<_Self,OrigPtr,Unerasability,V,DT>
where DT:Copy
{
    fn clone(&self)->Self{
        *self
    }
}


impl<_Self,OrigPtr,Unerasability,V> VTableTO<_Self,OrigPtr,Unerasability,V,()>{

/**
Wraps an erased vtable.

# Safety

These are the requirements for the caller:

- `OrigPtr` must be a pointer to the type that the vtable functions 
    take as the first parameter.

- The vtable must not come from a reborrowed RObject
    (created using RObject::reborrow or RObject::reborrow_mut).

- The vtable must be the `<SomeVTableName>` of a struct declared with 
    `#[derive(StableAbi)]``#[sabi(kind(Prefix(prefix_struct="<SomeVTableName>")))]`.

- The vtable must have `StaticRef<RObjectVtable<..>>` 
    as its first declared field
*/
    pub const unsafe fn for_robject(vtable:StaticRef<V>)->Self{
        Self{
            vtable,
            for_dyn_trait:(),
            _for:PhantomData
        }
    }


}


impl<_Self,OrigPtr,Unerasability,V,DT> VTableTO<_Self,OrigPtr,Unerasability,V,DT>{
    pub const fn robject_vtable(&self)->StaticRef<V>{
        self.vtable
    }
    
    pub const fn dyntrait_vtable(&self)->&DT{
        &self.for_dyn_trait
    }
}

impl<'borr,_Self,ErasedPtr,OrigPtr,I,Unerasability,V> 
    VTableTO_DT<'borr,_Self,ErasedPtr,OrigPtr,I,Unerasability,V>
{

/**
Wraps an erased vtable,alongside the vtable for DynTrait.

# Safety

This has the same safety requirements as the 'for_robject' constructor
*/
    pub const unsafe fn for_dyntrait(
        vtable:StaticRef<V>,
        for_dyn_trait:VTableDT<'borr,_Self,ErasedPtr,OrigPtr,I,Unerasability>,
    )->Self{
        Self{
            vtable,
            for_dyn_trait,
            _for:PhantomData
        }
    }
    
}


//////////////////////////////////////////////////////////////////////////////



#[doc(hidden)]
pub trait AreTraitsImpld<IA,_Self,ErasedPtr,OrigPtr>:Sized {
    const VTABLE_VAL:RObjectVtableVal<_Self,ErasedPtr,Self>;
}

impl<IA,_Self,ErasedPtr,OrigPtr,I> AreTraitsImpld<IA,_Self,ErasedPtr,OrigPtr> for I
where 
    I:InterfaceType,
    I::Sync:RequiresSync<_Self,ErasedPtr,OrigPtr>,
    I::Send:RequiresSend<_Self,ErasedPtr,OrigPtr>,
    I::Clone:InitCloneField<_Self,ErasedPtr,OrigPtr>,
    I::Debug:InitDebugField<_Self,ErasedPtr,OrigPtr>,
    I::Display:InitDisplayField<_Self,ErasedPtr,OrigPtr>,
    IA:GetUTID<_Self>,
{
    const VTABLE_VAL:RObjectVtableVal<_Self,ErasedPtr,I>=
        RObjectVtableVal{
            _sabi_tys:PhantomData,
            _sabi_type_id:<IA as GetUTID<_Self>>::UID,
            _sabi_drop:c_functions::drop_pointer_impl::<OrigPtr,ErasedPtr>,
            _sabi_clone:<I::Clone as InitCloneField<_Self,ErasedPtr,OrigPtr>>::VALUE,
            _sabi_debug:<I::Debug as InitDebugField<_Self,ErasedPtr,OrigPtr>>::VALUE,
            _sabi_display:<I::Display as InitDisplayField<_Self,ErasedPtr,OrigPtr>>::VALUE,
        };
}

// A dummy type used to get around a compiler limitation WRT associated constants in traits.
#[doc(hidden)]
struct GetRObjectVTableHelper<IA,_Self,ErasedPtr,OrigPtr,I>(IA,_Self,ErasedPtr,OrigPtr,I);

impl<IA,_Self,ErasedPtr,OrigPtr,I> 
    GetRObjectVTableHelper<IA,_Self,ErasedPtr,OrigPtr,I>
where 
    I:AreTraitsImpld<IA,_Self,ErasedPtr,OrigPtr>,
{
    const __TMP_0:*const WithMetadata<RObjectVtableVal<_Self,ErasedPtr,I>>={
        &__sabi_re::WithMetadata::new(
            PrefixTypeTrait::METADATA,
            I::VTABLE_VAL,
        )
    };
    const TMP_VTABLE:StaticRef<WithMetadata<RObjectVtableVal<_Self,ErasedPtr,I>>>=unsafe{
        StaticRef::from_raw(Self::__TMP_0)
    };


}


/// The vtable for RObject,which all  `#[trait_object]` derived trait objects contain.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_struct="RObjectVtable")))]
#[sabi(missing_field(default))]
pub struct RObjectVtableVal<_Self,ErasedPtr,I>{
    pub _sabi_tys:PhantomData<extern "C" fn(_Self,ErasedPtr,I)>,
    
    pub _sabi_type_id:Constructor<MaybeCmp<UTypeId>>,

    #[sabi(last_prefix_field)]
    pub _sabi_drop :unsafe extern "C" fn(this:&mut ErasedPtr),
    pub _sabi_clone:Option<unsafe extern "C" fn(this:&ErasedPtr)->ErasedPtr>,
    pub _sabi_debug:Option<
        unsafe extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>
    >,
    pub _sabi_display:Option<
        unsafe extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>
    >,
}



/// The common prefix of all `#[trait_object]` derived vtables,
/// with `RObjectVtable` as its first field.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    bound="I:InterfaceBound",
    extra_checks="<I as InterfaceBound>::extra_checks",
    kind(Prefix(prefix_struct="BaseVtable")),
)]
pub(super)struct BaseVtableVal<_Self,ErasedPtr,I>{
    pub _sabi_tys:PhantomData<unsafe extern "C" fn(_Self,ErasedPtr,I)>,

    #[sabi(last_prefix_field)]
    pub _sabi_vtable:StaticRef<RObjectVtable<_Self,ErasedPtr,I>>,
}

use self::trait_bounds::*;
pub mod trait_bounds{
    use super::*;

    macro_rules! declare_conditional_marker {
        (
            type $selector:ident;
            trait $trait_name:ident[$self_:ident,$ErasedPtr:ident,$OrigPtr:ident]
            where [ $($where_preds:tt)* ]
        ) => (
            pub trait $trait_name<$self_,$ErasedPtr,$OrigPtr>{}

            impl<$self_,$ErasedPtr,$OrigPtr> $trait_name<$self_,$ErasedPtr,$OrigPtr> 
            for Unimplemented<trait_marker::$selector>
            {}
            
            impl<$self_,$ErasedPtr,$OrigPtr> $trait_name<$self_,ErasedPtr,$OrigPtr> 
            for Implemented<trait_marker::$selector>
            where
                $($where_preds)*
            {}
        )
    }

    macro_rules! declare_field_initalizer {
        (
            type $selector:ident;
            trait $trait_name:ident[$self_:ident,$ErasedPtr:ident,$OrigPtr:ident]
            where [ $($where_preds:tt)* ]
            type=$field_ty:ty,
            value=$field_value:expr,
        ) => (
            pub trait $trait_name<$self_,$ErasedPtr,$OrigPtr>{
                const VALUE:Option<$field_ty>;
            }

            impl<$self_,$ErasedPtr,$OrigPtr> $trait_name<$self_,$ErasedPtr,$OrigPtr> 
            for Unimplemented<trait_marker::$selector>
            {
                const VALUE:Option<$field_ty>=None;
            }

            impl<$self_,$ErasedPtr,$OrigPtr> $trait_name<$self_,ErasedPtr,$OrigPtr> 
            for Implemented<trait_marker::$selector>
            where
                $($where_preds)*
            {
                const VALUE:Option<$field_ty>=Some($field_value);
            }
        )
    }


    declare_conditional_marker!{
        type Send;
        trait RequiresSend[_Self,ErasedPtr,OrigPtr]
        where [ _Self:Send,OrigPtr:Send ]
    }

    declare_conditional_marker!{
        type Sync;
        trait RequiresSync[_Self,ErasedPtr,OrigPtr]
        where [ _Self:Sync,OrigPtr:Sync ]
    }

    declare_field_initalizer!{
        type Clone;
        trait InitCloneField[_Self,ErasedPtr,OrigPtr]
        where [ OrigPtr:Clone ]
        type=unsafe extern "C" fn(this:&ErasedPtr)->ErasedPtr,
        value=c_functions::clone_pointer_impl::<OrigPtr,ErasedPtr>,
    }
    declare_field_initalizer!{
        type Debug;
        trait InitDebugField[_Self,ErasedPtr,OrigPtr]
        where [ _Self:Debug ]
        type=unsafe extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>,
        value=c_functions::debug_impl::<_Self>,
    }
    declare_field_initalizer!{
        type Display;
        trait InitDisplayField[_Self,ErasedPtr,OrigPtr]
        where [ _Self:Display ]
        type=unsafe extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>,
        value=c_functions::display_impl::<_Self>,
    }




}

