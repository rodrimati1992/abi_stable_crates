//! Contains the ffi-safe equivalent of `std::boxed::Box`.

use std::{
    borrow::{Borrow, BorrowMut},
    error::Error as StdError,
    future::Future,
    hash::Hasher,
    io::{self, BufRead, IoSlice, IoSliceMut, Read, Seek, Write},
    iter::FusedIterator,
    marker::{PhantomData, Unpin},
    mem::ManuallyDrop,
    ops::DerefMut,
    pin::Pin,
    ptr::{self, NonNull},
    task::{Context, Poll},
};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use crate::{
    marker_type::NonOwningPhantom,
    pointer_trait::{
        AsMutPtr, AsPtr, CallReferentDrop, CanTransmuteElement, Deallocate, GetPointerKind,
        OwnedPointer, PK_SmartPointer,
    },
    prefix_type::WithMetadata,
    sabi_types::MovePtr,
    std_types::utypeid::{new_utypeid, UTypeId},
    traits::IntoReprRust,
};

// #[cfg(test)]
#[cfg(all(test, not(feature = "only_new_tests")))]
mod test;

mod private {
    use super::*;

    /// Ffi-safe equivalent of `std::box::Box`.
    ///
    /// # Example
    ///
    /// Declaring a recursive datatype.
    ///
    /// ```
    /// use abi_stable::{
    ///     std_types::{RBox, RString},
    ///     StableAbi,
    /// };
    ///
    /// #[repr(u8)]
    /// #[derive(StableAbi)]
    /// enum Command {
    ///     SendProduct {
    ///         id: u64,
    ///     },
    ///     GoProtest {
    ///         cause: RString,
    ///         place: RString,
    ///     },
    ///     SendComplaint {
    ///         cause: RString,
    ///         website: RString,
    ///     },
    ///     WithMetadata {
    ///         command: RBox<Command>,
    ///         metadata: RString,
    ///     },
    /// }
    ///
    /// ```
    ///
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct RBox<T> {
        data: NonNull<T>,
        vtable: BoxVtable_Ref<T>,
        _marker: PhantomData<T>,
    }

    impl<T> RBox<T> {
        /// Constucts an `RBox<T>` from a value.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::std_types::RBox;
        ///
        /// let baux = RBox::new(100);
        /// assert_eq!(*baux, 100);
        ///
        /// ```
        pub fn new(value: T) -> Self {
            Box::new(value).piped(RBox::from_box)
        }

        /// Constructs a `Pin<RBox<T>>`.
        ///
        pub fn pin(value: T) -> Pin<RBox<T>> {
            RBox::new(value).into_pin()
        }

        /// Converts a `Box<T>` to an `RBox<T>`, reusing its heap allocation.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::std_types::RBox;
        ///
        /// let baux = Box::new(200);
        /// let baux = RBox::from_box(baux);
        /// assert_eq!(*baux, 200);
        ///
        /// ```
        pub fn from_box(p: Box<T>) -> RBox<T> {
            RBox {
                data: unsafe { NonNull::new_unchecked(Box::into_raw(p)) },
                vtable: VTableGetter::<T>::LIB_VTABLE,
                _marker: PhantomData,
            }
        }

        /// Constructs a `Box<T>` from a `MovePtr<'_, T>`.
        ///
        /// # Example
        ///
        /// ```
        /// use std::mem::ManuallyDrop;
        ///
        /// use abi_stable::{
        ///     pointer_trait::OwnedPointer,
        ///     sabi_types::RSmallBox,
        ///     std_types::RBox,
        /// };
        ///
        /// let b = RSmallBox::<_, [u8; 1]>::new(77u8);
        /// let rbox: RBox<_> = b.in_move_ptr(|x| RBox::from_move_ptr(x));
        ///
        /// assert_eq!(*rbox, 77);
        ///
        /// ```
        pub fn from_move_ptr(p: MovePtr<'_, T>) -> RBox<T> {
            MovePtr::into_rbox(p)
        }

        #[inline(always)]
        pub(super) const fn data(&self) -> *mut T {
            self.data.as_ptr()
        }
        #[inline(always)]
        pub(super) fn data_mut(&mut self) -> *mut T {
            self.data.as_ptr()
        }

        #[inline(always)]
        pub(super) const fn vtable(&self) -> BoxVtable_Ref<T> {
            self.vtable
        }

        #[allow(dead_code)]
        #[cfg(test)]
        pub(super) fn set_vtable_for_testing(&mut self) {
            self.vtable = VTableGetter::<T>::LIB_VTABLE_FOR_TESTING;
        }
    }

    unsafe impl<T> AsPtr for RBox<T> {
        #[inline(always)]
        fn as_ptr(&self) -> *const T {
            self.data.as_ptr()
        }
    }
    unsafe impl<T> AsMutPtr for RBox<T> {
        #[inline(always)]
        fn as_mut_ptr(&mut self) -> *mut T {
            self.data.as_ptr()
        }
    }
}

pub use self::private::RBox;

unsafe impl<T> GetPointerKind for RBox<T> {
    type Kind = PK_SmartPointer;

    type PtrTarget = T;
}

unsafe impl<T, O> CanTransmuteElement<O> for RBox<T> {
    type TransmutedPtr = RBox<O>;

    unsafe fn transmute_element_(self) -> Self::TransmutedPtr {
        unsafe { core_extensions::utils::transmute_ignore_size(self) }
    }
}

impl<T> RBox<T> {
    /// Converts this `RBox<T>` into a `Box<T>`
    ///
    /// # Allocation
    ///
    /// If this is invoked outside of the dynamic library/binary that created the `RBox<T>`,
    /// it will allocate a new `Box<T>` and move the data into it.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RBox;
    ///
    /// let baux: RBox<u32> = RBox::new(200);
    /// let baux: Box<u32> = RBox::into_box(baux);
    /// assert_eq!(*baux, 200);
    ///
    /// ```
    pub fn into_box(this: Self) -> Box<T> {
        let this = ManuallyDrop::new(this);

        unsafe {
            let this_vtable = this.vtable();
            let other_vtable = VTableGetter::LIB_VTABLE;
            if ::std::ptr::eq(this_vtable.0.to_raw_ptr(), other_vtable.0.to_raw_ptr())
                || this_vtable.type_id()() == other_vtable.type_id()()
            {
                Box::from_raw(this.data())
            } else {
                let ret = Box::new(this.data().read());
                // Just deallocating the Box<_>. without dropping the inner value
                (this.vtable().destructor())(
                    this.data() as *mut (),
                    CallReferentDrop::No,
                    Deallocate::Yes,
                );
                ret
            }
        }
    }

    /// Unwraps this `Box<T>` into the value it owns on the heap.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RBox;
    ///
    /// let baux: RBox<u32> = RBox::new(200);
    /// let baux: u32 = RBox::into_inner(baux);
    /// assert_eq!(baux, 200);
    ///
    /// ```
    pub fn into_inner(this: Self) -> T {
        unsafe {
            let value = this.data().read();
            Self::drop_allocation(&mut ManuallyDrop::new(this));
            value
        }
    }

    /// Wraps this `RBox` in a `Pin`
    ///
    pub fn into_pin(self) -> Pin<RBox<T>> {
        // safety: this is the same as what Box does.
        unsafe { Pin::new_unchecked(self) }
    }
}

impl<T> DerefMut for RBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data() }
    }
}

/////////////////////////////////////////////////////////////////

unsafe impl<T> OwnedPointer for RBox<T> {
    #[inline]
    unsafe fn get_move_ptr(this: &mut ManuallyDrop<Self>) -> MovePtr<'_, T> {
        unsafe { MovePtr::from_raw(this.data_mut()) }
    }

    #[inline]
    unsafe fn drop_allocation(this: &mut ManuallyDrop<Self>) {
        let data: *mut T = this.data();
        unsafe {
            (this.vtable().destructor())(data as *mut (), CallReferentDrop::No, Deallocate::Yes);
        }
    }
}

/////////////////////////////////////////////////////////////////

impl<T> Borrow<T> for RBox<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T> BorrowMut<T> for RBox<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<T> AsRef<T> for RBox<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> AsMut<T> for RBox<T> {
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

/////////////////////////////////////////////////////////////////

impl_from_rust_repr! {
    impl[T] From<Box<T>> for RBox<T> {
        fn(this){
            RBox::from_box(this)
        }
    }
}

impl<T> From<RBox<T>> for Pin<RBox<T>> {
    fn from(boxed: RBox<T>) -> Pin<RBox<T>> {
        boxed.into_pin()
    }
}

/////////////////////////////////////////////////////////////////

impl<T> IntoReprRust for RBox<T> {
    type ReprRust = Box<T>;

    fn into_rust(self) -> Self::ReprRust {
        Self::into_box(self)
    }
}

/////////////////////////////////////////////////////////////////

impl<T> Default for RBox<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> Clone for RBox<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        (**self).clone().piped(Box::new).into()
    }
}

shared_impls! {pointer
    mod = box_impls
    new_type = RBox[][T],
    original_type = Box,
}

unsafe impl<T: Send> Send for RBox<T> {}
unsafe impl<T: Sync> Sync for RBox<T> {}
impl<T> Unpin for RBox<T> {}

///////////////////////////////////////////////////////////////

impl<I> Iterator for RBox<I>
where
    I: Iterator,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        (**self).next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
    fn nth(&mut self, n: usize) -> Option<I::Item> {
        (**self).nth(n)
    }
    fn last(self) -> Option<I::Item> {
        RBox::into_inner(self).last()
    }
}

impl<I> DoubleEndedIterator for RBox<I>
where
    I: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<I::Item> {
        (**self).next_back()
    }
    fn nth_back(&mut self, n: usize) -> Option<I::Item> {
        (**self).nth_back(n)
    }
}

impl<I> ExactSizeIterator for RBox<I>
where
    I: ExactSizeIterator,
{
    fn len(&self) -> usize {
        (**self).len()
    }
}

impl<I> FusedIterator for RBox<I> where I: FusedIterator {}

///////////////////////////////////////////////////////////////

impl<F> Future for RBox<F>
where
    F: Future + Unpin,
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        F::poll(Pin::new(&mut *self), cx)
    }
}

///////////////////////////////////////////////////////////////

impl<T> StdError for RBox<T>
where
    T: StdError,
{
    #[allow(deprecated, deprecated_in_future)]
    fn description(&self) -> &str {
        StdError::description(&**self)
    }

    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn StdError> {
        StdError::cause(&**self)
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        StdError::source(&**self)
    }
}

///////////////////////////////////////////////////////////////

impl<T> Read for RBox<T>
where
    T: Read,
{
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (**self).read_vectored(bufs)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        (**self).read_exact(buf)
    }
}

impl<T> Write for RBox<T>
where
    T: Write,
{
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).write(buf)
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        (**self).write_vectored(bufs)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        (**self).flush()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (**self).write_all(buf)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()> {
        (**self).write_fmt(fmt)
    }
}

impl<T> Seek for RBox<T>
where
    T: Seek,
{
    #[inline]
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }
}

impl<T> BufRead for RBox<T>
where
    T: BufRead,
{
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        (**self).fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        (**self).consume(amt)
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_until(byte, buf)
    }

    #[inline]
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_line(buf)
    }
}

///////////////////////////////////////////////////////////////

impl<T> Hasher for RBox<T>
where
    T: Hasher,
{
    fn finish(&self) -> u64 {
        (**self).finish()
    }
    fn write(&mut self, bytes: &[u8]) {
        (**self).write(bytes)
    }
    fn write_u8(&mut self, i: u8) {
        (**self).write_u8(i)
    }
    fn write_u16(&mut self, i: u16) {
        (**self).write_u16(i)
    }
    fn write_u32(&mut self, i: u32) {
        (**self).write_u32(i)
    }
    fn write_u64(&mut self, i: u64) {
        (**self).write_u64(i)
    }
    fn write_u128(&mut self, i: u128) {
        (**self).write_u128(i)
    }
    fn write_usize(&mut self, i: usize) {
        (**self).write_usize(i)
    }
    fn write_i8(&mut self, i: i8) {
        (**self).write_i8(i)
    }
    fn write_i16(&mut self, i: i16) {
        (**self).write_i16(i)
    }
    fn write_i32(&mut self, i: i32) {
        (**self).write_i32(i)
    }
    fn write_i64(&mut self, i: i64) {
        (**self).write_i64(i)
    }
    fn write_i128(&mut self, i: i128) {
        (**self).write_i128(i)
    }
    fn write_isize(&mut self, i: isize) {
        (**self).write_isize(i)
    }
}

///////////////////////////////////////////////////////////////

impl<T> Drop for RBox<T> {
    fn drop(&mut self) {
        unsafe {
            let data = self.data();
            let dstr = RBox::vtable(self).destructor();
            dstr(data as *mut (), CallReferentDrop::Yes, Deallocate::Yes);
        }
    }
}

///////////////////////////////////////////////////////////////

#[derive(StableAbi)]
#[repr(C)]
#[sabi(kind(Prefix))]
#[sabi(missing_field(panic))]
pub(crate) struct BoxVtable<T> {
    type_id: extern "C" fn() -> UTypeId,
    #[sabi(last_prefix_field)]
    destructor: unsafe extern "C" fn(*mut (), CallReferentDrop, Deallocate),
    _marker: NonOwningPhantom<T>,
}

struct VTableGetter<'a, T>(&'a T);

impl<'a, T: 'a> VTableGetter<'a, T> {
    const DEFAULT_VTABLE: BoxVtable<T> = BoxVtable {
        type_id: new_utypeid::<RBox<()>>,
        destructor: destroy_box::<T>,
        _marker: NonOwningPhantom::NEW,
    };

    staticref! {
        const WM_DEFAULT: WithMetadata<BoxVtable<T>> = WithMetadata::new(Self::DEFAULT_VTABLE);
    }

    // The VTABLE for this type in this executable/library
    const LIB_VTABLE: BoxVtable_Ref<T> = BoxVtable_Ref(Self::WM_DEFAULT.as_prefix());

    #[cfg(test)]
    staticref! {
        const WM_FOR_TESTING: WithMetadata<BoxVtable<T>> =
            WithMetadata::new(
                BoxVtable {
                    type_id: new_utypeid::<RBox<i32>>,
                    ..Self::DEFAULT_VTABLE
                },
            )
    }

    #[allow(dead_code)]
    #[cfg(test)]
    const LIB_VTABLE_FOR_TESTING: BoxVtable_Ref<T> =
        BoxVtable_Ref(Self::WM_FOR_TESTING.as_prefix());
}

unsafe extern "C" fn destroy_box<T>(
    ptr: *mut (),
    call_drop: CallReferentDrop,
    dealloc: Deallocate,
) {
    extern_fn_panic_handling! {no_early_return;
        let ptr = ptr as *mut T;
        if let CallReferentDrop::Yes = call_drop {
            unsafe { ptr::drop_in_place(ptr); }
        }
        if let Deallocate::Yes = dealloc {
            unsafe { drop(Box::from_raw(ptr as *mut ManuallyDrop<T>)); }
        }
    }
}

/////////////////////////////////////////////////////////////////
