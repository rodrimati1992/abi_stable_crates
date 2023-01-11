//! Contains the `RSmallBox<_>` type.

use crate::{
    pointer_trait::{
        AsMutPtr, AsPtr, CallReferentDrop, CanTransmuteElement, Deallocate, GetPointerKind,
        OwnedPointer, PK_SmartPointer,
    },
    sabi_types::MovePtr,
    std_types::RBox,
};

use std::{
    alloc::{self, Layout},
    fmt::{self, Display},
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr,
};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::inline_storage::ScratchSpace;
pub use crate::inline_storage::{alignment, InlineStorage};

pub use self::private::RSmallBox;

mod private {
    use super::*;

    ///
    /// A box type which stores small values inline as an optimization.
    ///
    /// # Inline storage
    ///
    /// The `Inline` type parameter
    /// is the storage space on the stack (as in inline with the `RSmallBox` struct)
    /// where small values get stored,instead of storing them on the heap.
    ///
    /// It has to have an alignment greater than or equal to the value being stored,
    /// otherwise storing the value on the heap.
    ///
    /// To ensure that the inline storage has enough alignemnt you can use one of the
    /// `AlignTo*` types from the (reexported) alignment submodule,
    /// or from `abi_stable::inline_storage::alignment`.
    ///
    /// # Examples
    ///
    /// ### In a nonexhaustive enum
    ///
    /// Using an RSmallBox to store a generic type in a nonexhaustive enum.
    ///
    /// ```
    /// use abi_stable::{reexports::SelfOps, sabi_types::RSmallBox, std_types::RString, StableAbi};
    ///
    /// #[repr(u8)]
    /// #[derive(StableAbi, Debug, Clone, PartialEq)]
    /// #[sabi(kind(WithNonExhaustive(
    ///     // Determines the maximum size of this enum in semver compatible versions.
    ///     // This is 7 usize large because:
    ///     //    - The enum discriminant occupies 1 usize(because the enum is usize aligned).
    ///     //    - RSmallBox<T,[usize;4]>: is 6 usize large
    ///     size = [usize;7],
    ///     // Determines the traits that are required when wrapping this enum in NonExhaustive,
    ///     // and are then available with it.
    ///     traits(Debug,Clone,PartialEq),
    /// )))]
    /// #[non_exhaustive]
    /// pub enum SomeEnum<T> {
    ///     Foo,
    ///     Bar,
    ///     // This variant was added in a newer (compatible) version of the library.
    ///     Baz(RSmallBox<T, [usize; 4]>),
    /// }
    ///
    /// impl<T> SomeEnum<T> {
    ///     pub fn is_inline(&self) -> bool {
    ///         match self {
    ///             SomeEnum::Foo => true,
    ///             SomeEnum::Bar => true,
    ///             SomeEnum::Baz(rsbox) => RSmallBox::is_inline(rsbox),
    ///             _ => true,
    ///         }
    ///     }
    ///
    ///     pub fn is_heap_allocated(&self) -> bool {
    ///         !self.is_inline()
    ///     }
    /// }
    ///
    /// #[repr(C)]
    /// #[derive(StableAbi, Debug, Clone, PartialEq)]
    /// pub struct FullName {
    ///     pub name: RString,
    ///     pub surname: RString,
    /// }
    ///
    /// # fn main(){
    ///
    /// let rstring = "Oh boy!"
    ///     .piped(RString::from)
    ///     .piped(RSmallBox::new)
    ///     .piped(SomeEnum::Baz);
    ///
    /// let full_name = FullName {
    ///     name: "R__e".into(),
    ///     surname: "L_____e".into(),
    /// }
    /// .piped(RSmallBox::new)
    /// .piped(SomeEnum::Baz);
    ///
    /// assert!(rstring.is_inline());
    /// assert!(full_name.is_heap_allocated());
    ///
    /// # }
    ///
    /// ```
    ///
    /// ### Trying out different `Inline` type parameters
    ///
    /// This example demonstrates how changing the `Inline` type parameter can
    /// change whether an RString is stored inline or on the heap.
    ///
    /// ```
    /// use abi_stable::{
    ///     inline_storage::alignment::AlignToUsize, sabi_types::RSmallBox, std_types::RString,
    ///     StableAbi,
    /// };
    ///
    /// use std::mem;
    ///
    /// type JustRightInlineBox<T> = RSmallBox<T, AlignToUsize<[u8; mem::size_of::<usize>() * 4]>>;
    ///
    /// let string = RString::from("What is that supposed to mean?");
    ///
    /// let small = RSmallBox::<_, [usize; 3]>::new(string.clone());
    /// let medium = RSmallBox::<_, [usize; 4]>::new(string.clone());
    /// let large = RSmallBox::<_, [usize; 16]>::new(string.clone());
    /// let not_enough_alignment = RSmallBox::<_, [u8; 64]>::new(string.clone());
    /// let just_right = JustRightInlineBox::new(string.clone());
    ///
    /// assert!(RSmallBox::is_heap_allocated(&small));
    /// assert!(RSmallBox::is_inline(&medium));
    /// assert!(RSmallBox::is_inline(&large));
    /// assert!(RSmallBox::is_heap_allocated(&not_enough_alignment));
    /// assert!(RSmallBox::is_inline(&just_right));
    ///
    /// ```
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(not_stableabi(Inline))]
    pub struct RSmallBox<T, Inline> {
        // This is an opaque field since we only care about its size and alignment
        #[sabi(unsafe_opaque_field)]
        inline: ScratchSpace<(), Inline>,
        ptr: *mut T,
        destroy: unsafe extern "C" fn(*mut T, CallReferentDrop, Deallocate),
        _marker: PhantomData<T>,
    }

    impl<T, Inline> RSmallBox<T, Inline> {
        /// Constructs this RSmallBox from a value.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::{sabi_types::RSmallBox, std_types::RString};
        ///
        /// let xbox = RSmallBox::<_, [usize; 4]>::new(RString::from("one"));
        ///
        /// ```
        #[inline]
        pub fn new(value: T) -> RSmallBox<T, Inline>
        where
            Inline: InlineStorage,
        {
            let mut value = ManuallyDrop::new(value);

            unsafe { RSmallBox::from_move_ptr(MovePtr::new(&mut *value)) }
        }

        /// Gets a raw pointer into the underlying data.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::{sabi_types::RSmallBox, std_types::RString};
        ///
        /// let mut play = RSmallBox::<_, [usize; 4]>::new(RString::from("station"));
        ///
        /// let play_addr = &mut play as *mut RSmallBox<_, _> as usize;
        /// let heap_addr = RSmallBox::as_mut_ptr(&mut play) as usize;
        ///
        /// assert_eq!(play_addr, heap_addr);
        ///
        /// ```
        #[inline]
        pub fn as_mut_ptr(this: &mut Self) -> *mut T {
            if this.ptr.is_null() {
                &mut this.inline as *mut ScratchSpace<(), Inline> as *mut T
            } else {
                this.ptr
            }
        }

        /// Gets a raw pointer into the underlying data.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::{reexports::SelfOps, sabi_types::RSmallBox, std_types::RVec};
        ///
        /// let mut generations = vec![1, 2, 3, 4, 5, 6, 7, 8]
        ///     .piped(RVec::from)
        ///     .piped(RSmallBox::<_, [usize; 2]>::new);
        ///
        /// let generations_addr = &generations as *const RSmallBox<_, _> as usize;
        /// let heap_addr = RSmallBox::as_ptr(&generations) as usize;
        ///
        /// assert_ne!(generations_addr, heap_addr);
        ///
        /// ```
        #[inline]
        pub fn as_ptr(this: &Self) -> *const T {
            if this.ptr.is_null() {
                &this.inline as *const ScratchSpace<(), Inline> as *const T
            } else {
                this.ptr
            }
        }

        /// Constructs this `RSmallBox` from a `MovePtr`.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::{pointer_trait::OwnedPointer, sabi_types::RSmallBox, std_types::RBox};
        ///
        /// let rbox = RBox::new(1000_u64);
        /// let rsbox: RSmallBox<u64, [u64; 1]> =
        ///     rbox.in_move_ptr(|x| RSmallBox::<u64, [u64; 1]>::from_move_ptr(x));
        ///
        /// assert_eq!(*rsbox, 1000_u64);
        ///
        /// ```
        pub fn from_move_ptr(from_ptr: MovePtr<'_, T>) -> Self
        where
            Inline: InlineStorage,
        {
            let destroy = destroy::<T>;
            let inline_size = mem::size_of::<Inline>();
            let value_size = mem::size_of::<T>();

            let inline_align = mem::align_of::<Inline>();
            let value_align = mem::align_of::<T>();

            unsafe {
                let mut inline: ScratchSpace<(), Inline> = ScratchSpace::uninit();
                let (storage_ptr, ptr) = if inline_size < value_size || inline_align < value_align {
                    let x = alloc::alloc(Layout::new::<T>());
                    (x, x as *mut T)
                } else {
                    (
                        (&mut inline as *mut ScratchSpace<(), Inline> as *mut u8),
                        ptr::null_mut(),
                    )
                };

                (MovePtr::into_raw(from_ptr) as *const T as *const u8)
                    .copy_to_nonoverlapping(storage_ptr, value_size);

                Self {
                    inline,
                    ptr,
                    destroy,
                    _marker: PhantomData,
                }
            }
        }

        /// Converts this `RSmallBox` into another one with a differnet inline size.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::sabi_types::RSmallBox;
        ///
        /// let old = RSmallBox::<u64, [u8; 4]>::new(599_u64);
        /// assert!(!RSmallBox::is_inline(&old));
        ///
        /// let new = RSmallBox::move_::<[u64; 1]>(old);
        /// assert!(RSmallBox::is_inline(&new));
        /// assert_eq!(*new, 599_u64);
        ///
        /// ```
        #[inline]
        pub fn move_<Inline2>(this: Self) -> RSmallBox<T, Inline2>
        where
            Inline2: InlineStorage,
        {
            Self::with_move_ptr(ManuallyDrop::new(this), RSmallBox::from_move_ptr)
        }

        /// Queries whether the value is stored inline.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::{sabi_types::RSmallBox, std_types::RString};
        ///
        /// let heap = RSmallBox::<u64, [u8; 4]>::new(599_u64);
        /// assert!(!RSmallBox::is_inline(&heap));
        ///
        /// let inline = RSmallBox::<RString, [usize; 4]>::new("hello".into());
        /// assert!(RSmallBox::is_inline(&inline));
        ///
        /// ```
        pub fn is_inline(this: &Self) -> bool {
            this.ptr.is_null()
        }

        /// Queries whether the value is stored on the heap.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::{sabi_types::RSmallBox, std_types::RHashMap};
        ///
        /// let heap = RSmallBox::<_, [u8; 4]>::new(String::new());
        /// assert!(RSmallBox::is_heap_allocated(&heap));
        ///
        /// let inline = RSmallBox::<_, [usize; 3]>::new(RHashMap::<u8, ()>::new());
        /// assert!(!RSmallBox::is_heap_allocated(&inline));
        ///
        /// ```
        pub fn is_heap_allocated(this: &Self) -> bool {
            !this.ptr.is_null()
        }

        /// Unwraps this pointer into its owned value.
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::sabi_types::RSmallBox;
        ///
        /// let rbox = RSmallBox::<_, [usize; 3]>::new(vec![0, 1, 2]);
        /// assert_eq!(RSmallBox::into_inner(rbox), vec![0, 1, 2]);
        ///
        /// ```
        #[allow(clippy::redundant_closure)]
        pub fn into_inner(this: Self) -> T {
            Self::with_move_ptr(ManuallyDrop::new(this), |x| MovePtr::into_inner(x))
        }

        pub(super) unsafe fn drop_in_place(this: &mut Self, drop_referent: CallReferentDrop) {
            let (ptr, dealloc) = if this.ptr.is_null() {
                (
                    &mut this.inline as *mut ScratchSpace<(), Inline> as *mut T,
                    Deallocate::No,
                )
            } else {
                (this.ptr, Deallocate::Yes)
            };
            unsafe { (this.destroy)(ptr, drop_referent, dealloc) };
        }
    }

    /// Converts an RBox into an RSmallBox,currently this allocates.
    impl<T, Inline> From<RBox<T>> for RSmallBox<T, Inline>
    where
        Inline: InlineStorage,
    {
        fn from(this: RBox<T>) -> Self {
            RBox::with_move_ptr(ManuallyDrop::new(this), Self::from_move_ptr)
        }
    }

    /// Converts a RSmallBox into an RBox,currently this allocates.
    impl<T, Inline> From<RSmallBox<T, Inline>> for RBox<T>
    where
        Inline: InlineStorage,
    {
        fn from(this: RSmallBox<T, Inline>) -> RBox<T> {
            OwnedPointer::with_move_ptr(ManuallyDrop::new(this), |x| MovePtr::into_rbox(x))
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

unsafe impl<T, Inline> GetPointerKind for RSmallBox<T, Inline> {
    type Kind = PK_SmartPointer;

    type PtrTarget = T;
}

impl<T, Inline> Deref for RSmallBox<T, Inline> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*Self::as_ptr(self) }
    }
}

impl<T, Inline> DerefMut for RSmallBox<T, Inline> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *Self::as_mut_ptr(self) }
    }
}

unsafe impl<T, Inline> AsPtr for RSmallBox<T, Inline> {
    fn as_ptr(&self) -> *const T {
        Self::as_ptr(self)
    }
}

unsafe impl<T, Inline> AsMutPtr for RSmallBox<T, Inline> {
    fn as_mut_ptr(&mut self) -> *mut T {
        Self::as_mut_ptr(self)
    }
}

impl<T, Inline> Default for RSmallBox<T, Inline>
where
    T: Default,
    Inline: InlineStorage,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T, Inline> Clone for RSmallBox<T, Inline>
where
    T: Clone,
    Inline: InlineStorage,
{
    fn clone(&self) -> Self {
        RSmallBox::new((**self).clone())
    }
}

impl<T, Inline> Display for RSmallBox<T, Inline>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

shared_impls! {
    mod=box_impls
    new_type=RSmallBox[][T,Inline],
    original_type=Box,
}

unsafe impl<T, O, Inline> CanTransmuteElement<O> for RSmallBox<T, Inline> {
    type TransmutedPtr = RSmallBox<O, Inline>;

    unsafe fn transmute_element_(self) -> Self::TransmutedPtr {
        unsafe { core_extensions::utils::transmute_ignore_size(self) }
    }
}

unsafe impl<T: Send, Inline> Send for RSmallBox<T, Inline> {}
unsafe impl<T: Sync, Inline> Sync for RSmallBox<T, Inline> {}

///////////////////////////////////////////////////////////////////////////////

impl<T, Inline> Serialize for RSmallBox<T, Inline>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (**self).serialize(serializer)
    }
}

impl<'de, T, Inline> Deserialize<'de> for RSmallBox<T, Inline>
where
    Inline: InlineStorage,
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Self::new)
    }
}

//////////////////////////////////////////////////////////////////////////////

unsafe impl<T, Inline> OwnedPointer for RSmallBox<T, Inline> {
    #[inline]
    unsafe fn get_move_ptr(this: &mut ManuallyDrop<Self>) -> MovePtr<'_, T> {
        unsafe { MovePtr::new(&mut **this) }
    }

    #[inline]
    unsafe fn drop_allocation(this: &mut ManuallyDrop<Self>) {
        unsafe {
            Self::drop_in_place(&mut **this, CallReferentDrop::No);
        }
    }
}

impl<T, Inline> Drop for RSmallBox<T, Inline> {
    fn drop(&mut self) {
        unsafe {
            Self::drop_in_place(self, CallReferentDrop::Yes);
        }
    }
}

unsafe extern "C" fn destroy<T>(ptr: *mut T, drop_referent: CallReferentDrop, dealloc: Deallocate) {
    extern_fn_panic_handling! {no_early_return;
        if let CallReferentDrop::Yes=drop_referent{
            unsafe { ptr::drop_in_place(ptr); }
        }
        if let Deallocate::Yes = dealloc{
            unsafe { drop(Box::from_raw(ptr as *mut ManuallyDrop<T>)); }
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

#[cfg(all(test, not(feature = "only_new_tests")))]
mod tests;
