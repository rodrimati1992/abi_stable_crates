use super::*;

use crate::{
    erased_types::{FormattingMode,InterfaceBound,VTableDT},
    marker_type::NonOwningPhantom,
    prefix_type::PrefixRef,
    type_level::{
        downcasting::{GetUTID},
        impl_enum::{Implemented,Unimplemented},
        trait_marker,
    },
    sabi_types::Constructor,
    std_types::{UTypeId,RResult,RString,Tuple3},
};

//////////////////////////////////////////////////////////////////////////////

/// Gets an `RObjectVtable_Ref<_Self,ErasedPtr,TO>`(the vtable for RObject itself),
/// which is stored as the first field of all generated trait object vtables.
pub unsafe trait GetRObjectVTable<IA,_Self,ErasedPtr,OrigPtr>:Sized+InterfaceType{
    const ROBJECT_VTABLE:RObjectVtable_Ref<_Self,ErasedPtr,Self>;
}


unsafe impl<IA,_Self,ErasedPtr,OrigPtr,I> 
    GetRObjectVTable<IA,_Self,ErasedPtr,OrigPtr> for I
where
    I:AreTraitsImpld<IA,_Self,ErasedPtr,OrigPtr>+InterfaceType,
{
    const ROBJECT_VTABLE:RObjectVtable_Ref<_Self,ErasedPtr,Self>={
        GetRObjectVTableHelper::<IA,_Self,ErasedPtr,OrigPtr,I>::TMP_VTABLE
    };
}


//////////////////////////////////////////////////////////////////////////////

/// The `VTableTO` passed to `#[sabi_trait]`
/// generated trait objects that have `RObject` as their backend.
#[allow(non_camel_case_types)]
pub type VTableTO_RO<T,OrigPtr,Downcasting,V>=VTableTO<T,OrigPtr,Downcasting,V,()>;

/// The `VTableTO` passed to `#[sabi_trait]`
/// generated trait objects that have `DynTrait` as their backend.
#[allow(non_camel_case_types)]
pub type VTableTO_DT<'borr,_Self,ErasedPtr,OrigPtr,I,Downcasting,V>=
    VTableTO<
        _Self,
        OrigPtr,
        Downcasting,
        V,
        VTableDT<'borr,_Self,ErasedPtr,OrigPtr,I,Downcasting>
    >;



/// This is used to safely pass the vtable to `#[sabi_trait]` generated trait objects,
/// using `<Trait>_CTO::from_const( &value, <Trait>_MV::VTABLE )`.
///
/// `<Trait>` is whatever the name of the trait that one is constructing the trait object for.
pub struct VTableTO<_Self,OrigPtr,Downcasting,V,DT>{
    vtable:PrefixRef<V>,
    for_dyn_trait:DT,
    _for:PhantomData<Constructor<Tuple3<_Self,OrigPtr,Downcasting>>>,
}

impl<_Self,OrigPtr,Downcasting,V,DT> Copy for VTableTO<_Self,OrigPtr,Downcasting,V,DT>
where DT:Copy
{}

impl<_Self,OrigPtr,Downcasting,V,DT> Clone for VTableTO<_Self,OrigPtr,Downcasting,V,DT>
where DT:Copy
{
    fn clone(&self)->Self{
        *self
    }
}


impl<_Self,OrigPtr,Downcasting,V> VTableTO<_Self,OrigPtr,Downcasting,V,()>{

/**
Wraps an erased vtable.

# Safety

These are the requirements for the caller:

- `OrigPtr` must be a pointer to the type that the vtable functions 
    take as the first parameter.

- The vtable must not come from a reborrowed RObject
    (created using RObject::reborrow or RObject::reborrow_mut).

- The vtable must be the `<SomeVTableName>` of a struct declared with 
    `#[derive(StableAbi)]``#[sabi(kind(Prefix(prefix_ref="<SomeVTableName>")))]`.

- The vtable must have `PrefixRef<RObjectVtable<..>>` 
    as its first declared field
*/
    pub const unsafe fn for_robject(vtable:PrefixRef<V>)->Self{
        Self{
            vtable,
            for_dyn_trait:(),
            _for:PhantomData
        }
    }


}


impl<_Self,OrigPtr,Downcasting,V,DT> VTableTO<_Self,OrigPtr,Downcasting,V,DT>{
    /// Gets the vtable that RObject is constructed with.
    pub const fn robject_vtable(&self)->PrefixRef<V>{
        self.vtable
    }
}

impl<'borr,_Self,ErasedPtr,OrigPtr,I,Downcasting,V> 
    VTableTO_DT<'borr,_Self,ErasedPtr,OrigPtr,I,Downcasting,V>
{
    /// Gets the vtable for DynTrait.
    pub const fn dyntrait_vtable(
        &self
    )->VTableDT<'borr,_Self,ErasedPtr,OrigPtr,I,Downcasting>{
        self.for_dyn_trait
    }
}

impl<'borr,_Self,ErasedPtr,OrigPtr,I,Downcasting,V> 
    VTableTO_DT<'borr,_Self,ErasedPtr,OrigPtr,I,Downcasting,V>
{

/**
Wraps an erased vtable,alongside the vtable for DynTrait.

# Safety

This has the same safety requirements as the 'for_robject' constructor
*/
    pub const unsafe fn for_dyntrait(
        vtable:PrefixRef<V>,
        for_dyn_trait:VTableDT<'borr,_Self,ErasedPtr,OrigPtr,I,Downcasting>,
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
    const VTABLE_VAL:RObjectVtable<_Self,ErasedPtr,Self>;
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
    const VTABLE_VAL:RObjectVtable<_Self,ErasedPtr,I>=
        RObjectVtable{
            _sabi_tys:NonOwningPhantom::NEW,
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
    staticref!{
        const TMP_WM: WithMetadata<RObjectVtable<_Self,ErasedPtr,I>> =
            WithMetadata::new(PrefixTypeTrait::METADATA, I::VTABLE_VAL);
    }

    const TMP_VTABLE: RObjectVtable_Ref<_Self,ErasedPtr,I> = unsafe{
        RObjectVtable_Ref(Self::TMP_WM.as_prefix())
    };


}


/// The vtable for RObject,which all  `#[trait_object]` derived trait objects contain.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
#[sabi(missing_field(default))]
pub struct RObjectVtable<_Self,ErasedPtr,I>{
    pub _sabi_tys:NonOwningPhantom<(_Self,ErasedPtr,I)>,
    
    pub _sabi_type_id:Constructor<MaybeCmp<UTypeId>>,

    pub _sabi_drop :unsafe extern "C" fn(this:RMut<'_, ErasedPtr>),
    pub _sabi_clone:Option<unsafe extern "C" fn(this:RRef<'_, ErasedPtr>)->ErasedPtr>,
    pub _sabi_debug:Option<
        unsafe extern "C" fn(RRef<'_, ErasedObject>,FormattingMode,&mut RString)->RResult<(),()>
    >,
    #[sabi(last_prefix_field)]
    pub _sabi_display:Option<
        unsafe extern "C" fn(RRef<'_, ErasedObject>,FormattingMode,&mut RString)->RResult<(),()>
    >,
}



/// The common prefix of all `#[trait_object]` derived vtables,
/// with `RObjectVtable_Ref` as its first field.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    bound="I:InterfaceBound",
    extra_checks="<I as InterfaceBound>::EXTRA_CHECKS",
    kind(Prefix),
)]
pub(super)struct BaseVtable<_Self,ErasedPtr,I>{
    pub _sabi_tys:NonOwningPhantom<(_Self,ErasedPtr,I)>,

    #[sabi(last_prefix_field)]
    pub _sabi_vtable: RObjectVtable_Ref<_Self,ErasedPtr,I>,
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
        type=unsafe extern "C" fn(this:RRef<'_, ErasedPtr>)->ErasedPtr,
        value=c_functions::clone_pointer_impl::<OrigPtr,ErasedPtr>,
    }
    declare_field_initalizer!{
        type Debug;
        trait InitDebugField[_Self,ErasedPtr,OrigPtr]
        where [ _Self:Debug ]
        type=unsafe extern "C" fn(RRef<'_, ErasedObject>,FormattingMode,&mut RString)->RResult<(),()>,
        value=c_functions::debug_impl::<_Self>,
    }
    declare_field_initalizer!{
        type Display;
        trait InitDisplayField[_Self,ErasedPtr,OrigPtr]
        where [ _Self:Display ]
        type=unsafe extern "C" fn(RRef<'_, ErasedObject>,FormattingMode,&mut RString)->RResult<(),()>,
        value=c_functions::display_impl::<_Self>,
    }




}

