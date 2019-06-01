use super::*;
    
use std::{
    fmt,
    ops::{Deref,DerefMut},
};

use core_extensions::SelfOps;

use crate::{
    sabi_types::MaybeCmp,
    std_types::{RBox,UTypeId},
    pointer_trait::StableDeref,
    sabi_trait::markers::*,
    utils::{transmute_reference,transmute_mut_reference},
};

#[repr(C)]
#[derive(StableAbi)]
pub struct RObject<'lt,P,I,V>{
    vtable:StaticRef<V>,
    ptr: ManuallyDrop<P>,
    _marker:PhantomData<Tuple2<&'lt (),I>>,
}


macro_rules! impl_from_ptr_method {
    ( 
        $( #[$attr:meta] )*
        method_name=$method_name:ident,
        requires_any=$requires_any:ty 
    ) => (
        $( #[$attr] )*
        pub fn $method_name<'lt,OrigPtr,Params>(
            ptr:OrigPtr,
        )-> RObject<'lt,P,I,V>
        where 
            OrigPtr:TransmuteElement<(),TransmutedPtr=P>+'lt,
            P:StableDeref<Target=()>,
            OrigPtr::Target:Sized+'lt,
            I:GetVTable<
                $requires_any,
                OrigPtr::Target,
                OrigPtr::TransmutedPtr,
                OrigPtr,
                Params,
                VTable=V
            >,
            I:GetRObjectVTable<$requires_any,OrigPtr::Target,OrigPtr::TransmutedPtr,OrigPtr>
        {
            RObject{
                vtable:I::get_vtable(),
                ptr:unsafe{
                    let ptr=TransmuteElement::<()>::transmute_element(ptr,PhantomData);
                    ManuallyDrop::new(ptr)
                },
                _marker:PhantomData,
            }
        }

    )
}

impl<P,I,V> RObject<'_,P,I,V>{
    impl_from_ptr_method!{
        /**
Creates a trait object from a pointer to a type that must implement the 
trait that I requires.

The constructed trait object cannot be converted back to the original type.

        */
        method_name=from_ptr,
        requires_any=NoImplAny 
    }
    impl_from_ptr_method!{
        /**
Creates a trait object from a pointer to a type that must implement the 
trait that I requires.

The constructed trait object can be converted back to the original type with 
the `sabi_*_unerased` methods (RObject reserves `sabi` as a prefix for its own methods).

        */
        method_name=from_ptr_unerasable,
        requires_any=YesImplAny 
    }
}

impl<I,V> RObject<'_,RBox<()>,I,V>{
/**
Creates a trait object from a type that must implement the trait that I requires.

The constructed trait object cannot be converted back to the original type.
*/
    pub fn from_value<'lt,T,Params>(
        value:T,
    )-> RObject<'lt,RBox<()>,I,V>
    where 
        T:'lt,
        I:GetVTable<NoImplAny,T,RBox<()>,RBox<T>,Params,VTable=V>,
        I:GetRObjectVTable<NoImplAny,T,RBox<()>,RBox<T>>
    {
        Self::from_ptr::<_,Params>(RBox::new(value))
    }

/**
Creates a trait object from a type that must implement the trait that I requires.

The constructed trait object can be converted back to the original type with 
the `sabi_*_unerased` methods (RObject reserves `sabi` as a prefix for its own methods).
*/
    pub fn from_value_unerasable<'lt,T,Params>(
        value:T,
    )-> RObject<'lt,RBox<()>,I,V>
    where 
        T:'lt,
        I:GetVTable<YesImplAny,T,RBox<()>,RBox<T>,Params,VTable=V>,
        I:GetRObjectVTable<YesImplAny,T,RBox<()>,RBox<T>>
    {
        Self::from_ptr_unerasable::<_,Params>(RBox::new(value))
    }
}


impl<'lt,P,I,V> Clone for RObject<'lt,P,I,V> 
where 
    I:InterfaceType<Clone=True>,
{
    fn clone(&self)->Self{
        let ptr=self.sabi_robject_vtable()._sabi_clone().unwrap()(&self.ptr);
        Self{
            vtable:self.vtable,
            ptr:ManuallyDrop::new(ptr),
            _marker:PhantomData,
        }
    }
}


unsafe impl<'lt,P,I,V> Send for RObject<'lt,P,I,V> 
where 
    I:InterfaceType<Send=True>,
{}

unsafe impl<'lt,P,I,V> Sync for RObject<'lt,P,I,V> 
where 
    I:InterfaceType<Sync=True>,
{}


impl<'lt,P,I,V> RObject<'lt,P,I,V>{
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
    /// - The RObject was constructed using the `from_(value|ptr)` method
    ///
    /// - `T` is not the concrete type this `RObject<_>` was constructed with.
    ///
    pub fn sabi_into_unerased<T>(self) -> Result<P::TransmutedPtr, UneraseError<Self>>
    where
        T:'static,
        P: Deref<Target=()>+TransmuteElement<T>,
    {
        check_unerased!(self,self.sabi_check_same_utypeid::<T>());
        unsafe {
            let this=ManuallyDrop::new(self);
            Ok(ptr::read(&*this.ptr).transmute_element(T::T)) 
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
    /// - The RObject was constructed using the `from_(value|ptr)` method
    ///
    /// - `T` is not the concrete type this `RObject<_>` was constructed with.
    ///
    pub fn sabi_as_unerased<T>(&self) -> Result<&T, UneraseError<&Self>>
    where
        T:'static,
        P:Deref<Target=()>+TransmuteElement<T>,
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
    /// - The RObject was constructed using the `frfrom_(value|ptr)_*` method
    ///
    /// - `T` is not the concrete type this `RObject<_>` was constructed with.
    ///
    pub fn sabi_as_unerased_mut<T>(&mut self) -> Result<&mut T, UneraseError<&mut Self>>
    where
        T:'static,
        P:DerefMut<Target=()>+TransmuteElement<T>,
    {
        check_unerased!(self,self.sabi_check_same_utypeid::<T>());
        unsafe { 
            Ok(transmute_mut_reference::<(),T>(&mut **self.ptr))
        }
    }

}


impl<'lt,P,I,V> RObject<'lt,P,I,V>{

    #[inline]
    pub fn sabi_vtable<'a>(&self)->&'a V{
        self.vtable.get()
    }

    /// The vtable common to all `#[sabi_trait]` generated trait objects.
    #[inline]
    pub fn sabi_robject_vtable<'a>(&self)->&'a RObjectVtable<P,I>{
        unsafe{ 
            let vtable=&*(self.vtable.get() as *const V as *const BaseVtable<P,I>);
            vtable._sabi_vtable().get()
        }
    }

    #[inline]
    fn sabi_into_erased_ptr(self)->ManuallyDrop<P>{
        let mut __this= ManuallyDrop::new(self);
        unsafe{ ptr::read(&mut __this.ptr) }
    }

    #[inline]
    pub fn sabi_erased_ref(&self)->&ErasedObject<()>
    where
        P: __DerefTrait<Target=()>
    {
        unsafe{&*((&**self.ptr) as *const () as *const ErasedObject<()>)}
    }
    
    #[inline]
    pub fn sabi_erased_mut(&mut self)->&mut ErasedObject<()>
    where
        P: __DerefMutTrait<Target=()>
    {
        unsafe{&mut *((&mut **self.ptr) as *mut () as *mut ErasedObject<()>)}
    }

    #[inline]
    pub fn sabi_with_value<F,R>(self,f:F)->R
    where 
        P: OwnedPointer<Target=()>,
        F:FnOnce(MovePtr<'_,()>)->R,
    {
        OwnedPointer::with_moved_ptr(self.sabi_into_erased_ptr(),f)
    }
}

impl<P,I,V> Drop for RObject<'_,P,I,V>{
    fn drop(&mut self){
        let destructor=self.sabi_robject_vtable()._sabi_drop();
        unsafe{
            destructor(&mut self.ptr);
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
