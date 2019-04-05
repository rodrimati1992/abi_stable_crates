use std::{
    ffi::c_void,
    fmt::{self, Debug},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
};

use crate::{
    pointer_trait::{StableDeref,StableDerefMut},
    reexports::*
};

/// Used to pass raw pointers and references to extern functions,
/// while allowing erasure of the concrete types the functions take.
///
/// # Safety motivation
///
/// The reason this is necessary is because the c standard disallows conversions
/// casting function pointers from `extern fn(*const T)` to `extern fn(*const c_void)`
#[repr(transparent)]
pub struct CAbi<T> {
    pointer: *const c_void,
    type_: PhantomData<T>,
}

impl<T: Copy> Copy for CAbi<T> {}
impl<T: Copy> Clone for CAbi<T> {
    fn clone(&self) -> Self {
        *self
    }
}

unsafe impl<T> StableAbi for CAbi<T>
where
    T: StableAbi,
{
    type IsNonZeroType = False;
    const LAYOUT: &'static TypeLayout = &TypeLayout::from_std_lib::<Self>(
        "CAbi",
        TLData::ReprTransparent(
            T::ABI_INFO.get()
        ),
        tl_genparams!(;T;),
    );
}

macro_rules! declare_conversion {
    ($type_:ty;impl[ $($lt:lifetime),* $(,)? ]) => {
        impl<$($lt,)* T> Sealed for $type_{}

        unsafe impl<$($lt,)* T> CAbiWrapped for $type_ {
            type Referent=T;

            fn as_ptr(&self)->*const T{
                *self as *const T
            }

            unsafe fn from_ptr(n:*const T)->Self{
                unsafe{ mem::transmute(n) }
            }
        }
    }
}

impl<T> From<T> for CAbi<T>
where
    T: CAbiWrapped,
{
    fn from(pointer: T) -> Self {
        CAbi {
            pointer: pointer.as_ptr() as _,
            type_: PhantomData,
        }
    }
}

impl<T> CAbi<*const T>{
    pub const fn from_raw(pointer:*const T)->Self{
        CAbi {
            pointer: pointer as *const _,
            type_: PhantomData,
        }
    }
}

impl<T> CAbi<*mut T>{
    pub const fn from_raw_mut(pointer:*mut T)->Self{
        CAbi {
            pointer: pointer as *const T as *const _,
            type_: PhantomData,
        }
    }
}

impl<T> CAbi<T>
where
    T: CAbiWrapped,
{
    pub fn into_inner(self) -> T {
        unsafe { T::from_ptr(self.pointer as *const T::Referent) }
    }
}

impl<'a, T> CAbi<&'a mut T> {
    pub fn freeze(&self) -> CAbi<&'_ T> {
        (&**self).into()
    }

    pub fn reborrow<'b>(&'b mut self) -> CAbi<&'b mut T> {
        unsafe { (&mut *(self.pointer as *mut T)).into() }
    }
}

declare_conversion! { *const T ; impl[] }
declare_conversion! { *mut T ; impl[] }
declare_conversion! { &'a T ; impl['a] }
declare_conversion! { &'a mut T ; impl['a] }

impl<T> Deref for CAbi<T>
where
    T: StableDeref,
    T::Target: Sized,
{
    type Target = T::Target;
    fn deref(&self) -> &T::Target {
        unsafe { 
            &*(self.pointer as *const T::Target)
        }
    }
}

impl<T> DerefMut for CAbi<T>
where
    T: StableDerefMut,
    T::Target: Sized,
{
    fn deref_mut(&mut self) -> &mut T::Target {
        unsafe { 
            unsafe { 
                &mut *(self.pointer as *const T::Target as *mut T::Target)
            }
        }
    }
}

impl<T> Debug for CAbi<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.pointer, f)
    }
}

//////////////////////////////////////////////

mod sealed {
    pub trait Sealed {}
}
use self::sealed::Sealed;

/// Types that implement this trait must be reference or pointer sized/aligned.
pub unsafe trait CAbiWrapped: Sealed {
    type Referent;
    fn as_ptr(&self) -> *const Self::Referent;

    unsafe fn from_ptr(n: *const Self::Referent) -> Self;
}
