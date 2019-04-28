/*!
Contains the `DynTrait` type,and related traits/type aliases.
*/

use std::{
    ops::DerefMut,
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr,
};

use serde::{de, ser, Deserialize, Deserializer};

#[allow(unused_imports)]
use core_extensions::{prelude::*, ResultLike};

use crate::{
    pointer_trait::{StableDeref, TransmuteElement},
    ErasedObject, 
    std_types::{RBox, RCow, RStr},
};

use super::*;
use super::{
    c_functions::adapt_std_fmt,
    trait_objects::*,
    vtable::{GetVtable, VTable},
    traits::InterfaceFor,
};


#[cfg(test)]
mod tests;

mod priv_ {
    use super::*;


    /**

DynTrait implements ffi-safe trait objects,for a selection of traits.

# Passing opaque values around with `DynTrait<_>`

One can pass non-StableAbi types around by using type erasure,using this type.

It generally looks like `DynTrait<Pointer<()>,Interface>`,where:

- Pointer is some `pointer_trait::StableDeref` pointer type.

- Interface is an `InterfaceType`,which describes what traits are 
    required when constructing the `DynTrait<_>` and which ones it implements.

`trait InterfaceType` allows describing which traits are required 
when constructing a `DynTrait<_>`,and which ones it implements.

### Construction

To construct a `DynTrait<_>` one can use these associated functions:
    
- from_value:
    Can be constructed from the value directly.
    Requires a value that implements ImplType.
    
- from_ptr:
    Can be constructed from a pointer of a value.
    Requires a value that implements ImplType.
    
- from_any_value:
    Can be constructed from the value directly.Requires a `'static` value.
    
- from_any_ptr
    Can be constructed from a pointer of a value.Requires a `'static` value.

### Trait object

`DynTrait<Pointer<()>,Interface>` 
can be used as a trait object for any combination of 
the traits listed bellow.

These are the traits:

- Clone 

- Display 

- Debug 

- Default: Can be called as an inherent method.

- Eq 

- PartialEq 

- Ord 

- PartialOrd 

- Hash 

- serde::Deserialize:
    first deserializes from a string,and then calls the objects' Deserialize impl.

- serde::Serialize:
    first calls the objects' Deserialize impl,then serializes that as a string.

### Deconstruction

`DynTrait<_>` can then be unwrapped into a concrete type,
within the same dynamic library/executable that constructed it,
using these (fallible) conversion methods:

- into_unerased:
    Unwraps into a pointer to `T`.
    Where `DynTrait<P<()>,Interface>`'s 
        Interface must equal `<T as ImplType>::Interface`

- as_unerased:
    Unwraps into a `&T`.
    Where `DynTrait<P<()>,Interface>`'s 
        Interface must equal `<T as ImplType>::Interface`

- as_unerased_mut:
    Unwraps into a `&mut T`.
    Where `DynTrait<P<()>,Interface>`'s 
        Interface must equal `<T as ImplType>::Interface`

- into_any_unerased:Unwraps into a pointer to `T`.Requires `T:'static`.

- as_any_unerased:Unwraps into a `&T`.Requires `T:'static`.

- as_any_unerased_mut:Unwraps into a `&mut T`.Requires `T:'static`.

# Example 

The primary example using `DynTrait<_>` is in the readme.


    
    */
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    #[sabi(bound="I:TagFromInterface")]
    #[sabi(tag="<I as TagFromInterface>::TAG")]
    pub struct DynTrait<P,I> {
        pub(super) object: ManuallyDrop<P>,
        vtable: *const VTable<P>,
        _marker:PhantomData<extern fn()->I>
    }

    impl DynTrait<(),()> {
        /// Constructors the `DynTrait<_>` from an ImplType implementor.
        ///
        /// Use this whenever possible instead of `from_any_value`,
        /// because it produces better error messages when unerasing the `DynTrait<_>`
        pub fn from_value<T>(object: T) -> DynTrait<RBox<()>,T::Interface>
        where
            T: ImplType,
            T: GetVtable<T,RBox<()>,RBox<T>>,
        {
            let object = RBox::new(object);
            DynTrait::from_ptr(object)
        }

        /// Constructors the `DynTrait<_>` from a pointer to an ImplType implementor.
        ///
        /// Use this whenever possible instead of `from_any_ptr`,
        /// because it produces better error messages when unerasing the `DynTrait<_>`
        pub fn from_ptr<P, T>(object: P) -> DynTrait<P::TransmutedPtr,T::Interface>
        where
            T: ImplType,
            T: GetVtable<T,P::TransmutedPtr,P>,
            P: StableDeref<Target = T>+TransmuteElement<()>,
        {
            DynTrait {
                object: unsafe{
                    // The lifetime here is 'static,so it's fine to erase the type.
                    ManuallyDrop::new(object.transmute_element(<()>::T))
                },
                vtable: T::get_vtable(),
                _marker:PhantomData,
            }
        }

        /// Constructors the `DynTrait<_>` from a type that doesn't implement `ImplType`.
        pub fn from_any_value<T,I>(object: T,interface:I) -> DynTrait<RBox<()>,I>
        where
            T:'static,
            I:InterfaceType,
            InterfaceFor<T,I> : GetVtable<T,RBox<()>,RBox<T>>,
        {
            let object = RBox::new(object);
            DynTrait::from_any_ptr(object,interface)
        }

        /// Constructors the `DynTrait<_>` from a pointer to a 
        /// type that doesn't implement `ImplType`.
        pub fn from_any_ptr<P, T,I>(object: P,_interface:I) -> DynTrait<P::TransmutedPtr,I>
        where
            I:InterfaceType,
            T:'static,
            InterfaceFor<T,I>: GetVtable<T,P::TransmutedPtr,P>,
            P: StableDeref<Target = T>+TransmuteElement<()>,
        {
            DynTrait {
                object: unsafe{
                    // The lifetime here is 'static,so it's fine to erase the type.
                    ManuallyDrop::new(object.transmute_element(<()>::T))
                },
                vtable: <InterfaceFor<T,I>>::get_vtable(),
                _marker:PhantomData,
            }
        }
    }

    impl<P,I> DynTrait<P,I> {
        pub(super) fn vtable<'a>(&self) -> &'a VTable<P>{
            unsafe {
                &*self.vtable
            }
        }

        /// Allows checking whether 2 `DynTrait<_>`s have a value of the same type.
        ///
        /// Note that types from different dynamic libraries/executables are 
        /// never considered equal.
        pub fn is_same_type<Other,I2>(&self,other:&DynTrait<Other,I2>)->bool{
            self.vtable_address()==other.vtable_address()||
            self.vtable().type_info().is_compatible(other.vtable().type_info())
        }

        pub(super)fn vtable_address(&self) -> usize {
            self.vtable as usize
        }

        pub(super) fn as_abi(&self) -> &ErasedObject
        where
            P: Deref,
        {
            self.object()
        }

        #[allow(dead_code)]
        pub(super) fn as_abi_mut(&mut self) -> &mut ErasedObject
        where
            P: DerefMut,
        {
            self.object_mut()
        }

        /// Returns the address of the wrapped object.
        ///
        /// This will not change between calls for the same `DynTrait<_>`.
        pub fn object_address(&self) -> usize
        where
            P: Deref,
        {
            self.object() as *const ErasedObject as usize
        }

        pub(super) fn object(&self) -> &ErasedObject
        where
            P: Deref,
        {
            unsafe { self.object_as() }
        }
        pub(super) fn object_mut(&mut self) -> &mut ErasedObject
        where
            P: DerefMut,
        {
            unsafe { self.object_as_mut() }
        }

        unsafe fn object_as<T>(&self) -> &T
        where
            P: Deref,
        {
            &*((&**self.object) as *const P::Target as *const T)
        }
        unsafe fn object_as_mut<T>(&mut self) -> &mut T
        where
            P: DerefMut,
        {
            &mut *((&mut **self.object) as *mut P::Target as *mut T)
        }
    }

    impl<P,I> DynTrait<P,I> {
        /// The uid in the vtable has to be the same as the one for T,
        /// otherwise it was not created from that T in the library that declared the opaque type.
        pub(super) fn check_same_destructor_opaque<A,T>(&self) -> Result<(), UneraseError>
        where
            P: TransmuteElement<T>,
            A: GetVtable<T,P,P::TransmutedPtr>,
        {
            let t_vtable:&VTable<P> = A::get_vtable();
            if self.vtable_address() == t_vtable as *const _ as usize
                || self.vtable().type_info().is_compatible(t_vtable.type_info())
            {
                Ok(())
            } else {
                Err(UneraseError {
                    expected_vtable_address: t_vtable as *const _ as usize,
                    expected_type_info:t_vtable.type_info(),
                    found_vtable_address: self.vtable as usize,
                    found_type_info:self.vtable().type_info(),
                })
            }
        }

        /// Unwraps the `DynTrait<_>` into a pointer of 
        /// the concrete type that it was constructed with.
        ///
        /// T is required to implement ImplType.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        pub fn into_unerased<T>(self) -> Result<P::TransmutedPtr, UneraseError>
        where
            P: TransmuteElement<T>,
            P::Target:Sized,
            T: ImplType + GetVtable<T,P,P::TransmutedPtr>,
        {
            self.check_same_destructor_opaque::<T,T>()?;
            unsafe { 
                let this=ManuallyDrop::new(self);
                Ok(ptr::read(&*this.object).transmute_element(T::T)) 
            }
        }

        /// Unwraps the `DynTrait<_>` into a reference of 
        /// the concrete type that it was constructed with.
        ///
        /// T is required to implement ImplType.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        pub fn as_unerased<T>(&self) -> Result<&T, UneraseError>
        where
            P: Deref + TransmuteElement<T>,
            T: ImplType + GetVtable<T,P,P::TransmutedPtr>,
        {
            self.check_same_destructor_opaque::<T,T>()?;
            unsafe { Ok(self.object_as()) }
        }

        /// Unwraps the `DynTrait<_>` into a mutable reference of 
        /// the concrete type that it was constructed with.
        ///
        /// T is required to implement ImplType.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        pub fn as_unerased_mut<T>(&mut self) -> Result<&mut T, UneraseError>
        where
            P: DerefMut + TransmuteElement<T>,
            T: ImplType + GetVtable<T,P,P::TransmutedPtr>,
        {
            self.check_same_destructor_opaque::<T,T>()?;
            unsafe { Ok(self.object_as_mut()) }
        }


        /// Unwraps the `DynTrait<_>` into a pointer of 
        /// the concrete type that it was constructed with.
        ///
        /// T is required to not borrows anything.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        pub fn into_any_unerased<T>(self) -> Result<P::TransmutedPtr, UneraseError>
        where
            P: TransmuteElement<T>,
            P::Target:Sized,
            Self:DynTraitBound,
            InterfaceFor<T,GetVWInterface<Self>>: GetVtable<T,P,P::TransmutedPtr>,
        {
            self.check_same_destructor_opaque::<InterfaceFor<T,GetVWInterface<Self>>,T>()?;
            unsafe {
                unsafe { 
                    let this=ManuallyDrop::new(self);
                    Ok(ptr::read(&*this.object).transmute_element(T::T)) 
                }
            }
        }

        /// Unwraps the `DynTrait<_>` into a reference of 
        /// the concrete type that it was constructed with.
        ///
        /// T is required to not borrows anything.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        pub fn as_any_unerased<T>(&self) -> Result<&T, UneraseError>
        where
            P: Deref + TransmuteElement<T>,
            Self:DynTraitBound,
            InterfaceFor<T,GetVWInterface<Self>>: GetVtable<T,P,P::TransmutedPtr>,
        {
            self.check_same_destructor_opaque::<InterfaceFor<T,GetVWInterface<Self>>,T>()?;
            unsafe { Ok(self.object_as()) }
        }

        /// Unwraps the `DynTrait<_>` into a mutable reference of 
        /// the concrete type that it was constructed with.
        ///
        /// T is required to not borrows anything.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        pub fn as_any_unerased_mut<T>(&mut self) -> Result<&mut T, UneraseError>
        where
            P: DerefMut + TransmuteElement<T>,
            Self:DynTraitBound,
            InterfaceFor<T,GetVWInterface<Self>>: GetVtable<T,P,P::TransmutedPtr>,
        {
            self.check_same_destructor_opaque::<InterfaceFor<T,GetVWInterface<Self>>,T>()?;
            unsafe { Ok(self.object_as_mut()) }
        }

    }

    impl<P,I> DynTrait<P,I> {
        /// Constructs a DynTrait<P,I> wrapping a `P`,using the same vtable.
        /// `P` must come from a function in the vtable,
        /// to ensure that it is compatible with the functions in it.
        pub(super) fn from_new_ptr(&self, object: P) -> Self {
            Self {
                object:ManuallyDrop::new(object),
                vtable: self.vtable,
                _marker:PhantomData,
            }
        }

        /// Constructs a `DynTrait<P,I>` with the default value for `P`.
        pub fn default(&self) -> Self
        where
            P: Deref,
            I: InterfaceType<Default = True>,
        {
            let new = self.vtable().default_ptr::<I>()();
            self.from_new_ptr(new)
        }

        /// It serializes a `DynTrait<_>` into a string by using 
        /// `<ConcreteType as SerializeImplType>::serialize_impl`.
        pub fn serialized<'a>(&'a self) -> Result<RCow<'a, RStr<'a>>, RBoxError>
        where
            P: Deref,
            I: InterfaceType<Serialize = True>,
        {
            self.vtable().serialize::<I>()(self.as_abi()).into_result()
        }

        /// Deserializes a string into a `DynTrait<_>`,by using 
        /// `<I as DeserializeInterfaceType>::deserialize_impl`.
        pub fn deserialize_from_str<'a>(s: &'a str) -> Result<Self, RBoxError>
        where
            P: Deref,
            I: DeserializeInterfaceType<Deserialize = True, Deserialized = Self>,
        {
            s.piped(RStr::from).piped(I::deserialize_impl)
        }
    }

    impl<P,I> Drop for DynTrait<P,I>{
        fn drop(&mut self){
            let vtable=self.vtable();
            unsafe{
                vtable.drop_ptr()(&mut *self.object);
            }
        }
    }

}

pub use self::priv_::DynTrait;

impl<P, I> Clone for DynTrait<P,I>
where
    P: Deref,
    I: InterfaceType<Clone = True>,
{
    fn clone(&self) -> Self {
        let vtable = self.vtable();
        let new = vtable.clone_ptr::<I>()(&*self.object);
        self.from_new_ptr(new)
    }
}

impl<P, I> Display for DynTrait<P,I>
where
    P: Deref,
    I: InterfaceType<Display = True>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        adapt_std_fmt::<ErasedObject>(self.object(), self.vtable().display::<I>(), f)
    }
}

impl<P, I> Debug for DynTrait<P,I>
where
    P: Deref,
    I: InterfaceType<Debug = True>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        adapt_std_fmt::<ErasedObject>(self.object(), self.vtable().debug::<I>(), f)
    }
}

/**
First it serializes a `DynTrait<_>` into a string by using 
<ConcreteType as SerializeImplType>::serialize_impl,
then it serializes the string.

*/
/// ,then it .
impl<P, I> Serialize for DynTrait<P,I>
where
    P: Deref,
    I: InterfaceType<Serialize = True>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.vtable().serialize::<I>()(self.as_abi())
            .into_result()
            .map_err(ser::Error::custom)?
            .serialize(serializer)
    }
}

/// First it Deserializes a string,then it deserializes into a 
/// `DynTrait<_>`,by using `<I as DeserializeInterfaceType>::deserialize_impl`.
impl<'a, P, I> Deserialize<'a> for DynTrait<P,I>
where
    P: Deref,
    I: DeserializeInterfaceType<Deserialize = True, Deserialized = Self>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let s = String::deserialize(deserializer)?;
        I::deserialize_impl(RStr::from(&*s)).map_err(de::Error::custom)
    }
}

impl<P, I> Eq for DynTrait<P,I>
where
    Self: PartialEq,
    P: Deref,
    I: InterfaceType<Eq = True>,
{
}

impl<P, I> PartialEq for DynTrait<P,I>
where
    P: Deref,
    I: InterfaceType<PartialEq = True>,
{
    fn eq(&self, other: &Self) -> bool {
        // unsafe: must check that the vtable is the same,otherwise return a sensible value.
        if !self.is_same_type(other) {
            return false;
        }

        self.vtable().partial_eq::<I>()(self.as_abi(), other.as_abi())
    }
}

impl<P, I> Ord for DynTrait<P,I>
where
    P: Deref,
    I: InterfaceType<Ord = True>,
    Self: PartialOrd + Eq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        // unsafe: must check that the vtable is the same,otherwise return a sensible value.
        if !self.is_same_type(other) {
            return self.vtable_address().cmp(&other.vtable_address());
        }

        self.vtable().cmp::<I>()(self.as_abi(), other.as_abi()).into()
    }
}

impl<P, I> PartialOrd for DynTrait<P,I>
where
    P: Deref,
    I: InterfaceType<PartialOrd = True>,
    Self: PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // unsafe: must check that the vtable is the same,otherwise return a sensible value.
        if !self.is_same_type(other) {
            return Some(self.vtable_address().cmp(&other.vtable_address()));
        }

        self.vtable().partial_cmp::<I>()(self.as_abi(), other.as_abi())
            .map(IntoReprRust::into_rust)
            .into()
    }
}

impl<P, I> Hash for DynTrait<P,I>
where
    P: Deref,
    I: InterfaceType<Hash = True>,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.vtable().hash::<I>()(self.as_abi(), HasherObject::new(state))
    }
}


unsafe impl<P,I> Send for DynTrait<P,I>
where
    P: Send,
    P: Deref,
    I: InterfaceType<Send = True>,
{}


unsafe impl<P,I> Sync for DynTrait<P,I>
where
    P: Sync,
    P: Deref,
    I: InterfaceType<Sync = True>,
{}


//////////////////////////////////////////////////////////////////

mod sealed {
    use super::*;
    pub trait Sealed {}
    impl<P,I> Sealed for DynTrait<P,I> {}
}
use self::sealed::Sealed;

/// For accessing the Interface of a `DynTrait<Pointer<ZeroSized< Interface >>>`.
pub trait DynTraitBound: Sealed {
    type Interface: InterfaceType;
}

impl<P, I> DynTraitBound for DynTrait<P,I>
where
    P: Deref,
    I: InterfaceType,
{
    type Interface = I;
}


/// For accessing the `Interface` in a `DynTrait<Pointer<ZeroSized< Interface >>>`.
pub type GetVWInterface<This>=
    <This as DynTraitBound>::Interface;


//////////////////////////////////////////////////////////////////

/// Error for `DynTrait<_>` being unerased into the wrong type
/// with one of the `*unerased*` methods.
#[derive(Debug,Copy, Clone)]
pub struct UneraseError {
    expected_vtable_address: usize,
    expected_type_info:&'static TypeInfo,
    found_vtable_address: usize,
    found_type_info:&'static TypeInfo,
}


impl fmt::Display for UneraseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl ::std::error::Error for UneraseError {}
