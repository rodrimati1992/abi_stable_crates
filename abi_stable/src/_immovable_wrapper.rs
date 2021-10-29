//! Wrapper type to prevent moving the referent of a pointer out.

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use crate::pointer_trait::Pointer;

/// A Copy of ::std::pin::Pin without the `set` method,
/// so that `!Unpin` types are treated as though they are immovable.
#[repr(transparent)]
pub struct Immov<P> {
    pointer: P,
}

impl<P: Deref> Immov<P>
where
    P::Target: Unpin,
{
    #[inline(always)]
    pub fn new(pointer: P) -> Immov<P> {
        unsafe { Immov::new_unchecked(pointer) }
    }

    #[inline(always)]
    pub fn as_mut(self: &mut Immov<P>) -> Immov<&mut P::Target>
    where
        P: DerefMut,
    {
        unsafe { Immov::new_unchecked(&mut *self.pointer) }
    }
}

unsafe impl<P> Pointer for Immov<P>
where
    P: Pointer,
{
    type Referent = P::Referent;
}

impl<P: Deref> Immov<P> {
    #[inline(always)]
    pub unsafe fn new_unchecked(pointer: P) -> Immov<P> {
        Immov { pointer }
    }

    #[inline(always)]
    pub fn as_ref(self: &Immov<P>) -> Immov<&P::Target> {
        unsafe { Immov::new_unchecked(&*self.pointer) }
    }
}

impl<'a, T: ?Sized> Immov<&'a T> {
    pub unsafe fn map_unchecked<U, F>(self: Immov<&'a T>, func: F) -> Immov<&'a U>
    where
        F: FnOnce(&T) -> &U,
    {
        let pointer = &*self.pointer;
        let new_pointer = func(pointer);
        Immov::new_unchecked(new_pointer)
    }

    #[inline(always)]
    pub fn get_ref(self: Immov<&'a T>) -> &'a T {
        self.pointer
    }
}

impl<'a, T: ?Sized> Immov<&'a mut T> {
    #[inline(always)]
    pub fn into_ref(self: Immov<&'a mut T>) -> Immov<&'a T> {
        Immov {
            pointer: self.pointer,
        }
    }

    #[inline(always)]
    pub fn get_mut(self: Immov<&'a mut T>) -> &'a mut T
    where
        T: Unpin,
    {
        self.pointer
    }

    #[inline(always)]
    pub unsafe fn get_unchecked_mut(self: Immov<&'a mut T>) -> &'a mut T {
        self.pointer
    }

    pub unsafe fn map_unchecked_mut<U, F>(self: Immov<&'a mut T>, func: F) -> Immov<&'a mut U>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        let pointer = Immov::get_unchecked_mut(self);
        let new_pointer = func(pointer);
        Immov::new_unchecked(new_pointer)
    }
}

impl<P: Deref> Deref for Immov<P> {
    type Target = P::Target;
    fn deref(&self) -> &P::Target {
        Immov::get_ref(Immov::as_ref(self))
    }
}

impl<P: DerefMut> DerefMut for Immov<P>
where
    P::Target: Unpin,
{
    fn deref_mut(&mut self) -> &mut P::Target {
        Immov::get_mut(Immov::as_mut(self))
    }
}

impl<P: fmt::Debug> fmt::Debug for Immov<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.pointer, f)
    }
}

impl<P: fmt::Display> fmt::Display for Immov<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.pointer, f)
    }
}

impl<P: fmt::Pointer> fmt::Pointer for Immov<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.pointer, f)
    }
}
