use std::ops::DerefMut;

use serde::{de, ser, Deserialize, Deserializer};

#[allow(unused_imports)]
use core_extensions::{prelude::*, ResultLike};

use crate::{
    pointer_trait::{ErasedStableDeref, StableDeref, TransmuteElement},
    traits::{DeserializeImplType, ImplType},
    ErasedObject, RBox, RCow, RStr,
};

use super::*;
use super::{
    c_functions::adapt_std_fmt,
    trait_objects::*,
    vtable::{GetVtable, VTable},
};

/**

VirtualWrapper implements trait objects,for a selection of traits,
that is safe to use across the ffi boundary.

``

*/

#[cfg(test)]
mod tests;

mod priv_ {
    use super::*;

    /// Emulates trait objects for a selected number of traits,
    /// look at `InterfaceType` for a list of them.
    ///
    /// To construct this with an unwrapped value use `VirtualWrapper::from_value`.
    ///
    /// To construct this with a pointer of a value use `VirtualWrapper::from_ptr`.
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(inside_abi_stable_crate)]
    pub struct VirtualWrapper<P> {
        pub(super) object: P,
        vtable: &'static VTable<ErasedObject, ErasedObject>,
    }

    impl VirtualWrapper<()> {
        pub fn from_value<T>(object: T) -> VirtualWrapper<RBox<OpaqueType<T::Interface>>>
        where
            T: GetVtable<RBox<T>>,
        {
            let object = RBox::new(object);
            VirtualWrapper::from_ptr(object)
        }

        pub fn from_ptr<P, T>(object: P) -> VirtualWrapper<P::TransmutedPtr>
        where
            P: StableDeref<Target = T>,
            T: GetVtable<P> + ImplType,
            P: ErasedStableDeref<T::Interface>,
        {
            VirtualWrapper {
                object: object.erased(T::Interface::T),
                vtable: T::erased_vtable::<()>(),
            }
        }
    }

    impl<P> VirtualWrapper<P> {
        pub fn into_inner(self) -> P {
            self.object
        }

        // Allows us to call function pointers that take `P``as a parameter
        pub(super) fn vtable<'a, E>(&self) -> &'a VTable<ErasedObject, P>
        where
            P: Deref<Target = OpaqueType<E>>,
            E: GetImplFlags,
        {
            unsafe {
                mem::transmute::<&'a VTable<ErasedObject, ErasedObject>, &'a VTable<ErasedObject, P>>(
                    self.vtable,
                )
            }
        }

        pub fn vtable_address(&self) -> usize {
            self.vtable as *const _ as usize
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
            &*((&*self.object) as *const P::Target as *const T)
        }
        unsafe fn object_as_mut<T>(&mut self) -> &mut T
        where
            P: DerefMut,
        {
            &mut *((&mut *self.object) as *mut P::Target as *mut T)
        }
    }

    impl<P> VirtualWrapper<P> {
        /// The typeinfo in the vtable has to be the same as the one for T,
        /// otherwise it was not created from that T in the library that declared the opaque type.
        pub(super) fn check_same_destructor_opaque<T>(&self) -> Result<(), UneraseError>
        where
            P: TransmuteElement<T>,
            T: GetVtable<P::TransmutedPtr>,
        {
            let t_vtable = T::erased_vtable::<()>();
            if self.vtable_address() == t_vtable as *const _ as usize
                || self.vtable.type_info.is_compatible(t_vtable.type_info)
            {
                Ok(())
            } else {
                Err(UneraseError {
                    original_vtable: t_vtable,
                    opaque_vtable: self.vtable,
                })
            }
        }

        pub fn into_unerased<T>(self) -> Result<P::TransmutedPtr, UneraseError>
        where
            P: TransmuteElement<T>,
            T: GetVtable<P::TransmutedPtr>,
        {
            self.check_same_destructor_opaque::<T>()?;
            unsafe { Ok(self.object.transmute_element(T::T)) }
        }

        pub fn as_unerased<T>(&self) -> Result<&T, UneraseError>
        where
            P: Deref + TransmuteElement<T>,
            T: GetVtable<P::TransmutedPtr>,
        {
            self.check_same_destructor_opaque::<T>()?;
            unsafe { Ok(self.object_as()) }
        }

        pub fn as_unerased_mut<T>(&mut self) -> Result<&mut T, UneraseError>
        where
            P: DerefMut + TransmuteElement<T>,
            T: GetVtable<P::TransmutedPtr>,
        {
            self.check_same_destructor_opaque::<T>()?;
            unsafe { Ok(self.object_as_mut()) }
        }
    }

    impl<P> VirtualWrapper<P> {
        pub(super) fn from_new_ptr(&self, object: P) -> Self {
            Self {
                object,
                vtable: self.vtable,
            }
        }

        /// Constructs the default value for the pointer type this wraps.
        pub fn default<E>(&self) -> Self
        where
            P: Deref<Target = OpaqueType<E>>,
            E: InterfaceType<Default = True>,
        {
            let new = self.vtable().default_ptr::<E>()();
            self.from_new_ptr(new)
        }

        pub fn serialized<'a, E>(&'a self) -> Result<RCow<'a, str>, RBoxError>
        where
            P: Deref<Target = OpaqueType<E>>,
            E: InterfaceType<Serialize = True>,
        {
            self.vtable().serialize::<E>()(self.as_abi()).into_result()
        }

        pub fn deserialize_from_str<'a, E>(s: &'a str) -> Result<Self, RBoxError>
        where
            P: Deref<Target = OpaqueType<E>>,
            E: DeserializeImplType<Deserialize = True, Deserialized = Self>,
        {
            s.piped(RStr::from).piped(E::deserialize_impl)
        }
    }

}

pub use self::priv_::VirtualWrapper;

impl<P, E> Clone for VirtualWrapper<P>
where
    P: Deref<Target = OpaqueType<E>>,
    E: InterfaceType<Clone = True>,
{
    fn clone(&self) -> Self {
        let vtable = self.vtable();
        let new = vtable.clone_ptr::<E>()(&self.object);
        self.from_new_ptr(new)
    }
}

impl<P, E> Display for VirtualWrapper<P>
where
    P: Deref<Target = OpaqueType<E>>,
    E: InterfaceType<Display = True>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        adapt_std_fmt::<ErasedObject>(self.object(), self.vtable().display::<E>(), f)
    }
}

impl<P, E> Debug for VirtualWrapper<P>
where
    P: Deref<Target = OpaqueType<E>>,
    E: InterfaceType<Debug = True>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        adapt_std_fmt::<ErasedObject>(self.object(), self.vtable().debug::<E>(), f)
    }
}

impl<P, E> Serialize for VirtualWrapper<P>
where
    P: Deref<Target = OpaqueType<E>>,
    E: InterfaceType<Serialize = True>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.vtable().serialize::<E>()(self.as_abi())
            .into_result()
            .map_err(ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'a, P, E> Deserialize<'a> for VirtualWrapper<P>
where
    P: Deref<Target = OpaqueType<E>>,
    E: DeserializeImplType<Deserialize = True, Deserialized = Self>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let s = String::deserialize(deserializer)?;
        E::deserialize_impl(RStr::from(&*s)).map_err(de::Error::custom)
    }
}

impl<P, E> Eq for VirtualWrapper<P>
where
    Self: PartialEq,
    P: Deref<Target = OpaqueType<E>>,
    E: InterfaceType<Eq = True>,
{
}

impl<P, E> PartialEq for VirtualWrapper<P>
where
    P: Deref<Target = OpaqueType<E>>,
    E: InterfaceType<PartialEq = True>,
{
    fn eq(&self, other: &Self) -> bool {
        // unsafe: must check that the vtable is the same,otherwise return a sensible value.
        if self.vtable_address() != other.vtable_address() {
            return false;
        }

        self.vtable().partial_eq::<E>()(self.as_abi(), other.as_abi())
    }
}

impl<P, E> Ord for VirtualWrapper<P>
where
    P: Deref<Target = OpaqueType<E>>,
    E: InterfaceType<Ord = True>,
    Self: PartialOrd + Eq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        // unsafe: must check that the vtable is the same,otherwise return a sensible value.
        if self.vtable_address() != other.vtable_address() {
            return self.vtable_address().cmp(&other.vtable_address());
        }

        self.vtable().cmp::<E>()(self.as_abi(), other.as_abi()).into()
    }
}

impl<P, E> PartialOrd for VirtualWrapper<P>
where
    P: Deref<Target = OpaqueType<E>>,
    E: InterfaceType<PartialOrd = True>,
    Self: PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // unsafe: must check that the vtable is the same,otherwise return a sensible value.
        if self.vtable_address() != other.vtable_address() {
            return Some(self.vtable_address().cmp(&other.vtable_address()));
        }

        self.vtable().partial_cmp::<E>()(self.as_abi(), other.as_abi())
            .map(IntoReprRust::into_rust)
            .into()
    }
}

impl<P, E> Hash for VirtualWrapper<P>
where
    P: Deref<Target = OpaqueType<E>>,
    E: InterfaceType<Hash = True>,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.vtable().hash::<E>()(self.as_abi(), HasherTraitObject::new(state))
    }
}

//////////////////////////////////////////////////////////////////

mod sealed {
    use super::*;
    pub trait Sealed {}
    impl<P> Sealed for VirtualWrapper<P> {}
}
use self::sealed::Sealed;

pub trait VirtualWrapperTrait: Sealed {
    type Interface: InterfaceType;
}

impl<P, E> VirtualWrapperTrait for VirtualWrapper<P>
where
    P: Deref<Target = OpaqueType<E>>,
    E: InterfaceType,
{
    type Interface = E;
}

//////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
pub struct UneraseError {
    pub original_vtable: &'static VTable<ErasedObject, ErasedObject>,
    pub opaque_vtable: &'static VTable<ErasedObject, ErasedObject>,
}

impl Debug for UneraseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UneraseError")
            .field(
                "original_vtable_address",
                &(self.original_vtable as *const _ as usize),
            )
            .field("original_vtable", self.original_vtable)
            .field(
                "opaque_vtable_address",
                &(self.opaque_vtable as *const _ as usize),
            )
            .field("opaque_vtable", self.opaque_vtable)
            .finish()
    }
}

impl fmt::Display for UneraseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl ::std::error::Error for UneraseError {}
