//! Contains the `MovePtr<_>` type.

use std::{
    alloc::{self, Layout},
    fmt::{self, Display},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

use crate::{sabi_types::RMut, std_types::RBox, traits::IntoInner};

/// A move pointer, which allows moving the value from the reference,
/// consuming it in the process.
///
/// If `MovePtr::into_inner` isn't called, this drops the referenced value when it's dropped
///
/// # Safety
///
/// This is unsafe to construct since the user must ensure that the
/// original owner of the value never accesses it again.
///
/// # Motivation
///
/// `MovePtr` was created as a way to pass `self` by value to ffi-safe trait object methods,
/// since one can't simply pass `self` by value(because the type is erased).
///
/// # Examples
///
/// ### Using OwnedPointer::in_move_ptr
///
/// This is how one can use MovePtr without `unsafe`.
///
/// This simply moves the contents of an `RBox<T>` into a `Box<T>`.
///
/// ```
/// use abi_stable::{
///     pointer_trait::OwnedPointer, sabi_types::MovePtr, std_types::RBox,
/// };
///
/// fn move_rbox_to_box<T>(rbox: RBox<T>) -> Box<T> {
///     rbox.in_move_ptr(|move_ptr| MovePtr::into_box(move_ptr))
/// }
///
/// assert_eq!(move_rbox_to_box(RBox::new(99)), Box::new(99));
///
/// assert_eq!(move_rbox_to_box(RBox::new(())), Box::new(()));
///
/// assert_eq!(
///     move_rbox_to_box(RBox::new(String::from("SHIT"))),
///     Box::new(String::from("SHIT"))
/// );
///
///
/// ```
///
/// ### Using the (unsafe) `MovePtr::new`
///
/// This is (sort of) how `RBox<T>` implements moving the `T` it owns out of its allocation
///
/// This is basically what `OwnedPointer::{with_move_ptr,in_move_ptr}` do.
///
/// ```
/// use abi_stable::{
///     pointer_trait::{AsMutPtr, OwnedPointer},
///     sabi_types::MovePtr,
///     std_types::RBox,
/// };
///
/// use std::mem::ManuallyDrop;
///
/// let rbox = RBox::new(0x100);
///
/// let second_rbox;
///
/// unsafe {
///     let mut rbox = ManuallyDrop::new(rbox);
///
///     let move_ptr = unsafe { MovePtr::from_rmut(rbox.as_rmut()) };
///     second_rbox = RBox::from_move_ptr(move_ptr);
///
///     OwnedPointer::drop_allocation(&mut rbox);
/// }
///
/// assert_eq!(second_rbox, RBox::new(0x100));
///
///
/// ```
#[repr(transparent)]
#[derive(StableAbi)]
#[sabi(bound(T:'a))]
pub struct MovePtr<'a, T> {
    ptr: NonNull<T>,
    _marker: PhantomData<crate::utils::MutRef<'a, T>>,
}

impl<'a, T> MovePtr<'a, T> {
    /// Constructs this move pointer from a mutable reference,
    /// moving the value out of the reference.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the original owner of the value won't
    /// access the moved-out value anymore.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::MovePtr;
    ///
    /// use std::mem::ManuallyDrop;
    ///
    /// let mut manual = ManuallyDrop::new(String::from("hello"));
    ///
    /// let moveptr = unsafe { MovePtr::new(&mut *manual) };
    ///
    /// drop(moveptr); // moveptr drops the String here.
    /// ```
    #[inline]
    pub unsafe fn new(ptr: &'a mut T) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _marker: PhantomData,
        }
    }

    /// Constructs this move pointer from an `RMut`,
    /// moving the value out of the reference.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the original owner of the value won't
    /// access the moved-out value anymore.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::AsMutPtr, sabi_types::MovePtr, std_types::RString,
    ///     utils::manuallydrop_as_rmut,
    /// };
    ///
    /// use std::mem::ManuallyDrop;
    ///
    /// let mut mdrop = ManuallyDrop::new(RString::from("hello"));
    ///
    /// // safety: `mdrop` is never accessed again
    /// let moveptr = unsafe { MovePtr::from_rmut(manuallydrop_as_rmut(&mut mdrop)) };
    /// assert_eq!(*moveptr, "hello");
    ///
    /// let string: RString = MovePtr::into_inner(moveptr);
    /// assert_eq!(string, "hello");
    ///
    /// ```
    #[inline]
    pub const unsafe fn from_rmut(ptr: RMut<'a, T>) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr.into_raw()) },
            _marker: PhantomData,
        }
    }

    /// Constructs this move pointer from a raw pointer,
    /// moving the value out of it.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the original owner of the value won't
    /// access the moved-out value anymore.
    ///
    /// Because this takes a mutable pointer, the lifetime of this `MovePtr` is unbounded.
    /// You must ensure that it's not used for longer than the lifetime of the
    /// pointed-to value.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::AsMutPtr, rvec, sabi_types::MovePtr, std_types::RVec,
    ///     utils::manuallydrop_as_raw_mut,
    /// };
    ///
    /// use std::mem::ManuallyDrop;
    ///
    /// let mut mdrop = ManuallyDrop::new(rvec![3, 5, 8]);
    ///
    /// // safety: `mdrop` is never accessed again
    /// let moveptr = unsafe { MovePtr::from_raw(manuallydrop_as_raw_mut(&mut mdrop)) };
    /// assert_eq!(moveptr[..], [3, 5, 8]);
    ///
    /// let vector: RVec<u8> = MovePtr::into_inner(moveptr);
    /// assert_eq!(vector[..], [3, 5, 8]);
    ///
    /// ```
    pub const unsafe fn from_raw(ptr: *mut T) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _marker: PhantomData,
        }
    }

    /// Gets a raw pointer to the value being moved.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer, sabi_types::MovePtr, std_types::RBox,
    /// };
    ///
    /// let rbox = RBox::new(String::from("NOPE"));
    /// let address_rbox = &*rbox as *const String as usize;
    ///
    /// rbox.in_move_ptr(|move_ptr| {
    ///     assert_eq!(address_rbox, MovePtr::as_ptr(&move_ptr) as usize);
    /// });
    ///
    /// ```
    #[inline(always)]
    pub const fn as_ptr(this: &Self) -> *const T {
        this.ptr.as_ptr()
    }

    /// Gets a raw pointer to the value being moved.
    ///
    /// # Example
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer, sabi_types::MovePtr, std_types::RBox,
    /// };
    ///
    /// let rbox = RBox::new(String::from("NOPE"));
    /// let address_rbox = &*rbox as *const String as usize;
    ///
    /// rbox.in_move_ptr(|mut move_ptr| {
    ///     assert_eq!(address_rbox, MovePtr::as_mut_ptr(&mut move_ptr) as usize);
    /// });
    ///
    /// ```
    #[inline(always)]
    pub fn as_mut_ptr(this: &mut Self) -> *mut T {
        this.ptr.as_ptr()
    }

    /// Converts this MovePtr into a raw pointer,
    /// which must be moved from before the pointed to value is deallocated,
    /// otherwise the value will be leaked.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer, sabi_types::MovePtr, std_types::RBox,
    /// };
    ///
    /// let rbox = RBox::new(String::from("NOPE"));
    ///
    /// let string =
    ///     rbox.in_move_ptr(|move_ptr| unsafe { MovePtr::into_raw(move_ptr).read() });
    ///
    /// assert_eq!(string, String::from("NOPE"));
    ///
    /// ```
    #[inline]
    pub const fn into_raw(this: Self) -> *mut T {
        let ptr = this.ptr.as_ptr();
        std::mem::forget(this);
        ptr
    }

    /// Moves the value into a new `Box<T>`
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer, sabi_types::MovePtr, std_types::RBox,
    /// };
    ///
    /// let rbox = RBox::new(String::from("WHAT!!!"));
    ///
    /// let boxed = rbox.in_move_ptr(|move_ptr| unsafe { MovePtr::into_box(move_ptr) });
    ///
    /// assert_eq!(boxed, Box::new(String::from("WHAT!!!")));
    ///
    /// ```
    #[inline]
    pub fn into_box(this: Self) -> Box<T> {
        unsafe {
            let raw = Self::into_raw(this);

            if std::mem::size_of::<T>() == 0 {
                Box::from_raw(raw)
            } else {
                let allocated = alloc::alloc(Layout::new::<T>()) as *mut T;

                raw.copy_to_nonoverlapping(allocated, 1);

                Box::from_raw(allocated)
            }
        }
    }

    /// Moves the value into a new `RBox<T>`
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer, sabi_types::MovePtr, std_types::RBox,
    /// };
    ///
    /// let rbox = RBox::new(String::from("WHAT!!!"));
    ///
    /// let boxed = rbox.in_move_ptr(|move_ptr| unsafe { MovePtr::into_rbox(move_ptr) });
    ///
    /// assert_eq!(boxed, RBox::new(String::from("WHAT!!!")));
    ///
    /// ```
    #[inline]
    pub fn into_rbox(this: Self) -> RBox<T> {
        Self::into_box(this).into()
    }

    /// Moves the value out of the reference
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer, sabi_types::MovePtr, std_types::RBox,
    /// };
    ///
    /// let rbox = RBox::new(String::from("(The Wi)zard(of)oz"));
    ///
    /// let string = rbox.in_move_ptr(|ptr| MovePtr::into_inner(ptr));
    ///
    /// assert_eq!(string, String::from("(The Wi)zard(of)oz"));
    ///
    /// ```
    #[inline]
    pub fn into_inner(this: Self) -> T {
        let this = ManuallyDrop::new(this);
        unsafe { this.ptr.as_ptr().read() }
    }

    /// Transmute this `RMove<'a, T>` into a `RMove<'a, U>`.
    ///
    /// # Safety
    ///
    /// This has the safety requirements as
    /// [`std::mem::transmute`](https://doc.rust-lang.org/std/mem/fn.transmute.html),
    /// as well as requiring that this `MovePtr` is aligned for `U`.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::OwnedPointer,
    ///     sabi_types::MovePtr,
    ///     std_types::{RBox, RString, RVec},
    /// };
    ///
    /// let rbox = RBox::new(RString::from("hello"));
    ///
    /// let bytes = rbox.in_move_ptr(|ptr| unsafe {
    ///     MovePtr::into_inner(MovePtr::transmute::<RVec<u8>>(ptr))
    /// });
    ///
    /// assert_eq!(bytes.as_slice(), b"hello");
    ///
    /// ```
    #[inline]
    pub const unsafe fn transmute<U>(this: Self) -> MovePtr<'a, U>
    where
        U: 'a,
    {
        unsafe { std::mem::transmute::<MovePtr<'a, T>, MovePtr<'a, U>>(this) }
    }
}

shared_impls! {
    mod=move_ptr_impls
    new_type=MovePtr['a][T],
    original_type=AAAA,
}

impl<'a, T> Display for MovePtr<'a, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<'a, T> Deref for MovePtr<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*(self.ptr.as_ptr() as *const _) }
    }
}

impl<'a, T> DerefMut for MovePtr<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr.as_ptr() }
    }
}

impl<'a, T> IntoInner for MovePtr<'a, T> {
    type Element = T;

    fn into_inner_(self) -> T {
        Self::into_inner(self)
    }
}

impl<'a, T> Drop for MovePtr<'a, T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.ptr.as_ptr());
        }
    }
}

unsafe impl<'a, T: Send> Send for MovePtr<'a, T> {}

unsafe impl<'a, T: Sync> Sync for MovePtr<'a, T> {}

//#[cfg(test)]
#[cfg(all(test, not(feature = "only_new_tests")))]
mod test {
    use super::*;

    use std::sync::Arc;

    #[test]
    fn with_manuallydrop() {
        let arc = Arc::new(10);
        unsafe {
            let mut cloned_arc = ManuallyDrop::new(arc.clone());

            let move_ptr = MovePtr::new(&mut *cloned_arc);
            assert_eq!(Arc::strong_count(&*move_ptr), 2);

            let moved_arc = MovePtr::into_inner(move_ptr);
            assert_eq!(Arc::strong_count(&moved_arc), 2);
        }
        assert_eq!(Arc::strong_count(&arc), 1);
        unsafe {
            let mut cloned_arc = ManuallyDrop::new(arc.clone());

            let move_ptr = MovePtr::new(&mut *cloned_arc);
            assert_eq!(Arc::strong_count(&*move_ptr), 2);
        }
        assert_eq!(Arc::strong_count(&arc), 1);
    }

    #[test]
    fn take_mutable_reference() {
        unsafe {
            let mut val = 3u32;
            let mut mutref = ManuallyDrop::new(&mut val);

            let mut move_ptr = MovePtr::<&mut u32>::new(&mut *mutref);
            assert_eq!(**move_ptr, 3);

            **move_ptr += 2;
            assert_eq!(**move_ptr, 5);

            let moved_mut: &mut u32 = MovePtr::into_inner(move_ptr);
            assert_eq!(*moved_mut, 5);
        }
    }
}
