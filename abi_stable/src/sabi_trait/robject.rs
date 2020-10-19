use super::*;
    
use std::{
    fmt,
    ops::{Deref,DerefMut},
};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use crate::{
    abi_stability::PrefixStableAbi,
    erased_types::{
        c_functions::adapt_std_fmt,
        InterfaceBound,
    },
    sabi_types::{MaybeCmp,RRef,RMut},
    std_types::UTypeId,
    pointer_trait::{
        CanTransmuteElement,TransmuteElement,
        GetPointerKind,PK_SmartPointer,PK_Reference,PointerKind,
    },
    type_level::{
        impl_enum::{Implemented,Unimplemented},
        trait_marker,
    },
    sabi_trait::vtable::{BaseVtable_Ref, BaseVtable_Prefix},
    utils::{transmute_reference,transmute_mut_reference},
    StableAbi,
};


/**
`RObject` implements ffi-safe trait objects,for a minimal selection of traits.

The main use of `RObject<_>` is as the default backend for `#[sabi_trait]` 
generated trait objects.

# Construction

`RObject<_>` is how `#[sabi_trait]`-based ffi-safe trait objects are implemented,
and there's no way to construct it separate from those.

# Trait object

`RObject<'borrow,Pointer<()>,Interface,VTable>` 
can be used as a trait object for any combination of 
the traits listed below.

These are the traits:

- `Send`

- `Sync`

- `Debug`

- `Display`

- `Error`

- `Clone`

# Deconstruction

`RObject<_>` can be unwrapped into a concrete type,
within the same dynamic library/executable that constructed it,
using these (fallible) conversion methods:

- `into_unerased`: Unwraps into a pointer to `T`.Requires `T: 'static`.

- `as_unerased`: Unwraps into a `&T`.Requires `T: 'static`.

- `as_unerased_mut`: Unwraps into a `&mut T`.Requires `T: 'static`.

`RObject` can only be converted back if the trait object was constructed to allow it.


*/
#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    not_stableabi(V),
    bound="V:PrefixStableAbi",
    bound="I:InterfaceBound",
    extra_checks="<I as InterfaceBound>::EXTRA_CHECKS",
)]
pub struct RObject<'lt,P,I,V>
where
    P:GetPointerKind
{
    vtable:PrefixRef<V>,
    ptr: ManuallyDrop<P>,
    _marker:PhantomData<(&'lt (),I)>,
}

mod clone_impl{
    pub trait CloneImpl<PtrKind>{
        fn clone_impl(&self) -> Self;
    }
}
use self::clone_impl::CloneImpl;


/// This impl is for smart pointers.
impl<'lt,P, I,V> CloneImpl<PK_SmartPointer> for RObject<'lt,P,I,V>
where
    P: Deref+GetPointerKind,
    I: InterfaceType<Clone = Implemented<trait_marker::Clone>>,
{
    fn clone_impl(&self) -> Self {
        let ptr=unsafe{
            self.sabi_robject_vtable()._sabi_clone().unwrap()(&self.ptr)
        };
        Self{
            vtable:self.vtable,
            ptr:ManuallyDrop::new(ptr),
            _marker:PhantomData,
        }
    }
}

/// This impl is for references.
impl<'lt,P, I,V> CloneImpl<PK_Reference> for RObject<'lt,P,I,V>
where
    P: Deref+Copy+GetPointerKind,
    I: InterfaceType,
{
    fn clone_impl(&self) -> Self {
        Self{
            vtable:self.vtable,
            ptr:ManuallyDrop::new(*self.ptr),
            _marker:PhantomData,
        }
    }
}

/**
Clone is implemented for references and smart pointers,
using `GetPointerKind` to decide whether `P` is a smart pointer or a reference.

RObject does not implement Clone if P==`&mut ()` :


```compile_fail
use abi_stable::{
    sabi_trait::{
        doc_examples::ConstExample_TO,
        TU_Opaque,
    },
    std_types::*,
};

let mut object=ConstExample_TO::from_value(10usize,TU_Opaque);
let borrow=object.sabi_reborrow_mut();
let _=borrow.clone();
```

Here is the same example with `sabi_reborrow`

```
use abi_stable::{
    sabi_trait::{
        doc_examples::ConstExample_TO,
        TU_Opaque,
    },
    std_types::*,
};

let mut object=ConstExample_TO::from_value(10usize,TU_Opaque);
let borrow=object.sabi_reborrow();
let _=borrow.clone();
```


*/
impl<'lt,P, I,V> Clone for RObject<'lt,P,I,V>
where
    P: Deref+GetPointerKind,
    I: InterfaceType,
    Self:CloneImpl<<P as GetPointerKind>::Kind>,
{
    fn clone(&self) -> Self {
        self.clone_impl()
    }
}


impl<'lt,P,I,V> Debug for RObject<'lt,P,I,V> 
where
    P: Deref<Target=()>+GetPointerKind,
    I: InterfaceType<Debug = Implemented<trait_marker::Debug>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe{
            adapt_std_fmt::<ErasedObject>(
                self.sabi_erased_ref(), 
                self.sabi_robject_vtable()._sabi_debug().unwrap(), 
                f
            )
        }
    }
}


impl<'lt,P,I,V> Display for RObject<'lt,P,I,V> 
where
    P: Deref<Target=()>+GetPointerKind,
    I: InterfaceType<Display = Implemented<trait_marker::Display>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe{
            adapt_std_fmt::<ErasedObject>(
                self.sabi_erased_ref(), 
                self.sabi_robject_vtable()._sabi_display().unwrap(), 
                f
            )
        }
    }
}

impl<'lt,P, I,V> std::error::Error for RObject<'lt,P,I,V>
where
    P: Deref<Target=()>+GetPointerKind,
    I: InterfaceBound<
        Display=Implemented<trait_marker::Display>,
        Debug=Implemented<trait_marker::Debug>,
        Error=Implemented<trait_marker::Error>,
    >,
{}



unsafe impl<'lt,P,I,V> Send for RObject<'lt,P,I,V> 
where 
    P:GetPointerKind,
    I:InterfaceType<Send = Implemented<trait_marker::Send>>,
{}

unsafe impl<'lt,P,I,V> Sync for RObject<'lt,P,I,V> 
where
    P:GetPointerKind,
    I:InterfaceType<Sync = Implemented<trait_marker::Sync>>,
{}


impl<'lt,P,I,V> RObject<'lt,P,I,V>
where
    P:GetPointerKind<Target=()>,
{
/**

Constructs an RObject from a pointer and an extra vtable.

This is mostly intended to be called by `#[sabi_trait]` generated trait objects.

# Safety

These are the requirements for the caller:

- `P` must be a pointer to the type that the vtable functions 
    take as the first parameter.

- The vtable must not come from a reborrowed `RObject`
    (created using `RObject::reborrow` or `RObject::reborrow_mut`).

- The vtable must be the `SomeVTableName` of a struct declared with 
    `#[derive(StableAbi)] #[sabi(kind(Prefix(prefix_ref="SomeVTableName")))]`.

- The vtable must have `RObjectVtable_Ref` as its first declared field

*/
    pub unsafe fn with_vtable<OrigPtr>(
        ptr:OrigPtr,
        vtable:PrefixRef<V>,
    )-> RObject<'lt,P,I,V>
    where 
        OrigPtr:CanTransmuteElement<(),TransmutedPtr=P>,
        OrigPtr::Target:Sized+'lt,
        P:Deref<Target=()>,
    {
        RObject{
            vtable,
            ptr:ManuallyDrop::new( ptr.transmute_element::<()>() ),
            _marker:PhantomData,
        }
    }
}


impl<'borr,'a,I,V> RObject<'borr,RRef<'a,()>,I,V>{

/**
This function allows constructing an RObject in a constant/static.

This is mostly intended for `#[sabi_trait] generated trait objects`

# Safety

This has the same safety requirements as `RObject::with_vtable`

# Example

Because this is intended for `#[sabi_trait]` generated trait objects,
this demonstrates how to construct one in a constant.

```
use abi_stable::sabi_trait::{
    doc_examples::{ConstExample_CTO,ConstExample_MV},
    prelude::TU_Opaque,
};

const EXAMPLE0:ConstExample_CTO<'static,'static>=
    ConstExample_CTO::from_const(
        &0usize,
        TU_Opaque,
        ConstExample_MV::VTABLE,
    );


```


*/
    pub const unsafe fn with_vtable_const<T,Unerasability>(
        ptr: &'a T,
        vtable:VTableTO_RO<T,RRef<'a,T>,Unerasability,V>,
    )-> Self
    where
        T:'borr,
    {
        RObject{
            vtable: vtable.robject_vtable(),
            ptr:{
                let x=RRef::new(ptr).transmute_ref::<()>();
                ManuallyDrop::new(x)
            },
            _marker:PhantomData,
        }
    }
}


impl<'lt,P,I,V> RObject<'lt,P,I,V>
where
    P:GetPointerKind,
{
    /// The uid in the vtable has to be the same as the one for T,
    /// otherwise it was not created from that T in the library that 
    /// declared the trait object.
    fn sabi_check_same_utypeid<T>(&self) -> Result<(), UneraseError<()>>
    where
        T:'static,
    {
        let expected_typeid=self.sabi_robject_vtable()._sabi_type_id().get();
        let actual_typeid=UTypeId::new::<T>();
        if expected_typeid == MaybeCmp::Just(actual_typeid) {
            Ok(())
        } else {
            Err(UneraseError {
                robject:(),
                expected_typeid,
                actual_typeid,
            })
        }
    }

    /// Attempts to unerase this trait object into the pointer it was constructed with.
    ///
    /// # Errors
    ///
    /// This will return an error in any of these conditions:
    ///
    /// - It is called in a dynamic library/binary outside
    /// the one from which this RObject was constructed.
    ///
    /// - The trait object wrapping this `RObject` was constructed with a 
    /// `TU_Unerasable` argument.
    ///
    /// - `T` is not the concrete type this `RObject<_>` was constructed with.
    ///
    pub fn into_unerased<T>(self) -> Result<P::TransmutedPtr, UneraseError<Self>>
    where
        T:'static,
        P: Deref<Target=()>+CanTransmuteElement<T>,
    {
        check_unerased!(self,self.sabi_check_same_utypeid::<T>());
        unsafe {
            let this=ManuallyDrop::new(self);
            Ok(ptr::read(&*this.ptr).transmute_element::<T>()) 
        }
    }

    /// Attempts to unerase this trait object into a reference of 
    /// the value was constructed with.
    ///
    /// # Errors
    ///
    /// This will return an error in any of these conditions:
    ///
    /// - It is called in a dynamic library/binary outside
    /// the one from which this RObject was constructed.
    ///
    /// - The trait object wrapping this `RObject` was constructed with a 
    /// `TU_Unerasable` argument.
    ///
    /// - `T` is not the concrete type this `RObject<_>` was constructed with.
    ///
    pub fn as_unerased<T>(&self) -> Result<&T, UneraseError<&Self>>
    where
        T:'static,
        P:Deref<Target=()>+CanTransmuteElement<T>,
    {
        check_unerased!(self,self.sabi_check_same_utypeid::<T>());
        unsafe { 
            Ok(transmute_reference::<(),T>(&**self.ptr))
        }
    }

    /// Attempts to unerase this trait object into a mutable reference of 
    /// the value was constructed with.
    ///
    /// # Errors
    ///
    /// This will return an error in any of these conditions:
    ///
    /// - It is called in a dynamic library/binary outside
    /// the one from which this RObject was constructed.
    ///
    /// - The trait object wrapping this `RObject` was constructed with a 
    /// `TU_Unerasable` argument.
    ///
    /// - `T` is not the concrete type this `RObject<_>` was constructed with.
    ///
    pub fn as_unerased_mut<T>(&mut self) -> Result<&mut T, UneraseError<&mut Self>>
    where
        T:'static,
        P:DerefMut<Target=()>+CanTransmuteElement<T>,
    {
        check_unerased!(self,self.sabi_check_same_utypeid::<T>());
        unsafe { 
            Ok(transmute_mut_reference::<(),T>(&mut **self.ptr))
        }
    }

    /// Unwraps the `RObject<_>` into a pointer to T,
    /// without checking whether `T` is the type that the RObject was constructed with.
    ///
    /// # Safety
    ///
    /// You must check that `T` is the type that RObject was constructed
    /// with through other means.
    #[inline]
    pub unsafe fn unchecked_into_unerased<T>(self) -> P::TransmutedPtr
    where
        P: Deref<Target=()> + CanTransmuteElement<T>,
    {
        let this=ManuallyDrop::new(self);
        ptr::read(&*this.ptr).transmute_element::<T>()
    }

    /// Unwraps the `RObject<_>` into a reference to T,
    /// without checking whether `T` is the type that the RObject was constructed with.
    ///
    /// # Safety
    ///
    /// You must check that `T` is the type that RObject was constructed
    /// with through other means.
    #[inline]
    pub unsafe fn unchecked_as_unerased<T>(&self) -> &T
    where
        P:Deref<Target=()>,
    {
        transmute_reference::<(),T>(&**self.ptr)
    }

    /// Unwraps the `RObject<_>` into a mutable reference to T,
    /// without checking whether `T` is the type that the RObject was constructed with.
    ///
    /// # Safety
    ///
    /// You must check that `T` is the type that RObject was constructed
    /// with through other means.
    #[inline]
    pub unsafe fn unchecked_as_unerased_mut<T>(&mut self) -> &mut T
    where
        P:DerefMut<Target=()>,
    {
        transmute_mut_reference::<(),T>(&mut **self.ptr)
    }

}


mod private_struct {
    pub struct PrivStruct;
}
use self::private_struct::PrivStruct;


/// This is used to make sure that reborrowing does not change 
/// the Send-ness or Sync-ness of the pointer.
pub trait ReborrowBounds<SendNess,SyncNess>{}

// If it's reborrowing,it must have either both Sync+Send or neither.
impl ReborrowBounds<Unimplemented<trait_marker::Send>,Unimplemented<trait_marker::Sync>> 
for PrivStruct 
{}

impl ReborrowBounds<Implemented<trait_marker::Send>,Implemented<trait_marker::Sync>> 
for PrivStruct 
{}


impl<'lt,P,I,V> RObject<'lt,P,I,V>
where
    P:GetPointerKind,
    I:InterfaceType,
{
    /// Creates a shared reborrow of this RObject.
    ///
    /// This is only callable if `RObject` is either `Send + Sync` or `!Send + !Sync`.
    ///
    pub fn reborrow<'re>(&'re self)->RObject<'lt,&'re (),I,V> 
    where
        P:Deref<Target=()>,
        PrivStruct:ReborrowBounds<I::Send,I::Sync>,
    {
        // Reborrowing will break if I add extra functions that operate on `P`.
        RObject{
            vtable:self.vtable,
            ptr:ManuallyDrop::new(&**self.ptr),
            _marker:PhantomData,
        }
    }

    /// Creates a mutable reborrow of this RObject.
    ///
    /// The reborrowed RObject cannot use these methods:
    ///
    /// - RObject::clone
    ///
    /// This is only callable if `RObject` is either `Send + Sync` or `!Send + !Sync`.
    /// 
    pub fn reborrow_mut<'re>(&'re mut self)->RObject<'lt,&'re mut (),I,V> 
    where
        P:DerefMut<Target=()>,
        PrivStruct:ReborrowBounds<I::Send,I::Sync>,
    {
        // Reborrowing will break if I add extra functions that operate on `P`.
        RObject {
            vtable: self.vtable,
            ptr: ManuallyDrop::new(&mut **self.ptr),
            _marker:PhantomData,
        }
    }
}




impl<'lt,P,I,V> RObject<'lt,P,I,V>
where
    P:GetPointerKind,
{
    /// Gets the vtable.
    #[inline]
    pub fn sabi_et_vtable(&self)->PrefixRef<V>{
        self.vtable
    }

    /// The vtable common to all `#[sabi_trait]` generated trait objects.
    #[inline]
    pub fn sabi_robject_vtable(&self)->RObjectVtable_Ref<(),P,I>{
        unsafe{ 
            BaseVtable_Ref(self.vtable.cast::<BaseVtable_Prefix<(),P,I>>())
                ._sabi_vtable()
        }
    }

    #[inline]
    fn sabi_into_erased_ptr(self)->ManuallyDrop<P>{
        let mut __this= ManuallyDrop::new(self);
        unsafe{ ptr::read(&mut __this.ptr) }
    }

    /// Gets an `RRef` pointing to the erased object.
    #[inline]
    pub fn sabi_erased_ref(&self)->&ErasedObject<()>
    where
        P: __DerefTrait<Target=()>
    {
        unsafe{&*((&**self.ptr) as *const () as *const ErasedObject<()>)}
    }
    
    /// Gets an `RMut` pointing to the erased object.
    #[inline]
    pub fn sabi_erased_mut(&mut self)->&mut ErasedObject<()>
    where
        P: __DerefMutTrait<Target=()>
    {
        unsafe{&mut *((&mut **self.ptr) as *mut () as *mut ErasedObject<()>)}
    }

    /// Gets an `RRef` pointing to the erased object.
    pub fn sabi_as_rref(&self) -> RRef<'_, ()>
    where
        P: __DerefTrait<Target=()>
    {
        unsafe {
            std::mem::transmute(&**self.ptr as *const _ as *const ())
        }
    }

    /// Gets an `RMut` pointing to the erased object.
    pub fn sabi_as_rmut(&mut self) -> RMut<'_, ()>
    where
        P: __DerefMutTrait<Target=()>
    {
        unsafe {
            std::mem::transmute(&mut **self.ptr as *mut _ as *mut ())
        }
    }

    /// Calls the `f` callback with an `MovePtr` pointing to the erased object.
    #[inline]
    pub fn sabi_with_value<F,R>(self,f:F)->R
    where 
        P: OwnedPointer<Target=()>,
        F:FnOnce(MovePtr<'_,()>)->R,
    {
        OwnedPointer::with_move_ptr(self.sabi_into_erased_ptr(),f)
    }
}

impl<P,I,V> Drop for RObject<'_,P,I,V>
where
    P:GetPointerKind,
{
    fn drop(&mut self){
        // This condition is necessary because if the RObject was reborrowed,
        // the destructor function would take a different pointer type.
        if <P as GetPointerKind>::KIND==PointerKind::SmartPointer {
            let destructor=self.sabi_robject_vtable()._sabi_drop();
            unsafe{
                destructor(&mut self.ptr);
            }
        }
    }
}





//////////////////////////////////////////////////////////////////

/// Error for `RObject<_>` being unerased into the wrong type
/// with one of the `*unerased*` methods.
#[derive(Copy, Clone)]
pub struct UneraseError<T> {
    robject:T,
    expected_typeid:MaybeCmp<UTypeId>,
    actual_typeid:UTypeId,
}


impl<T> UneraseError<T>{
    fn map<F,U>(self,f:F)->UneraseError<U>
    where F:FnOnce(T)->U
    {
        UneraseError{
            robject        :f(self.robject),
            expected_typeid:self.expected_typeid,
            actual_typeid  :self.actual_typeid,
        }
    }

    /// Extracts the RObject,to handle the failure to unerase it.
    #[must_use]
    pub fn into_inner(self)->T{
        self.robject
    }
}


impl<D> fmt::Debug for UneraseError<D>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UneraseError")
            .field("dyn_trait",&"<not shown>")
            .field("expected_typeid",&self.expected_typeid)
            .field("actual_typeid",&self.actual_typeid)
            .finish()
    }
}

impl<D> fmt::Display for UneraseError<D>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl<D> ::std::error::Error for UneraseError<D> {}

//////////////////////////////////////////////////////////////////
