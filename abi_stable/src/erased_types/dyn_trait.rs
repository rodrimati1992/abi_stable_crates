//! Contains the `DynTrait` type, and related traits/type aliases.

use std::{
    fmt::{self, Write as fmtWrite},
    io,
    mem::ManuallyDrop,
    ptr,
    rc::Rc,
};

use serde::{de, ser, Deserialize, Deserializer};

use crate::{
    abi_stability::StableAbi,
    marker_type::{ErasedObject, NonOwningPhantom, UnsafeIgnoredType},
    pointer_trait::{
        AsMutPtr, AsPtr, CanTransmuteElement, GetPointerKind, OwnedPointer, PK_Reference,
        PK_SmartPointer, PointerKind, TransmuteElement,
    },
    prefix_type::PrefixRef,
    sabi_types::{MovePtr, RMut, RRef},
    std_types::{RBox, RIoError, RStr, RVec},
    type_level::{
        downcasting::{TD_CanDowncast, TD_Opaque},
        impl_enum::{Implemented, Unimplemented},
        trait_marker,
    },
};

#[allow(unused_imports)]
use crate::std_types::Tuple2;

use super::{
    c_functions::adapt_std_fmt,
    trait_objects::*,
    traits::{DeserializeDyn, GetSerializeProxyType},
    type_info::TypeInfoFor,
    vtable::{MakeVTable, VTable_Ref},
    IteratorItemOrDefault, *,
};

// #[cfg(test)]
#[cfg(all(test, not(feature = "only_new_tests")))]
mod tests;

#[cfg(doctest)]
pub mod doctests;

mod priv_ {
    use super::*;

    /// DynTrait implements ffi-safe trait objects, for a selection of traits.
    ///
    /// # Passing opaque values around with `DynTrait<_>`
    ///
    /// One can pass non-StableAbi types around by using type erasure, using this type.
    ///
    /// It generally looks like `DynTrait<'borrow, Pointer<()>, Interface>`, where:
    ///
    /// - `'borrow` is the borrow that the type that was erased had.
    ///
    /// - `Pointer` is a pointer type that implements [`AsPtr`].
    ///
    /// - `Interface` is an [`InterfaceType`], which describes what traits are
    ///     required when constructing the `DynTrait<_>` and which ones it implements.
    ///
    /// ###  Construction
    ///
    /// To construct a `DynTrait<_>` one can use these associated functions:
    ///     
    /// - [`from_value`](#method.from_value):
    ///     Can be constructed from the value directly.Requires a `'static` value.
    ///     
    /// - [`from_ptr`](#method.from_ptr)
    ///     Can be constructed from a pointer of a value.Requires a `'static` value.
    ///
    /// - [`from_borrowing_value`](#method.from_borrowing_value):
    ///     Can be constructed from the value directly.Cannot downcast the DynTrait afterwards.
    ///     
    /// - [`from_borrowing_ptr`](#method.from_borrowing_ptr)
    ///     Can be constructed from a pointer of a value.Cannot downcast the DynTrait afterwards.
    ///
    /// DynTrait uses the impls of the value in methods,
    /// which means that the pointer itself does not have to implement those traits,
    ///
    /// ###  Trait object
    ///
    /// `DynTrait<'borrow, Pointer<()>, Interface>`
    /// can be used as a trait object for any combination of
    /// the traits listed below.
    ///
    /// These are the traits:
    ///
    /// - [`Send`]
    ///
    /// - [`Sync`]
    ///
    /// - [`Unpin`]
    ///
    /// - [`Iterator`]
    ///
    /// - [`DoubleEndedIterator`]
    ///
    /// - [`std::fmt::Write`]
    ///
    /// - [`std::io::Write`]
    ///
    /// - [`std::io::Seek`]
    ///
    /// - [`std::io::Read`]
    ///
    /// - [`std::io::BufRead`]
    ///
    /// - [`Clone`]
    ///
    /// - [`Display`]
    ///
    /// - [`Debug`]
    ///
    /// - [`std::error::Error`]
    ///
    /// - [`Default`]: Can only be called as an inherent method.
    ///
    /// - [`Eq`]
    ///
    /// - [`PartialEq`]
    ///
    /// - [`Ord`]
    ///
    /// - [`PartialOrd`]
    ///
    /// - [`Hash`]
    ///
    /// - [`serde::Deserialize`]:
    ///     first deserializes from a string, and then calls the objects' Deserialize impl.
    ///
    /// - [`serde::Serialize`]:
    ///     first calls the objects' Deserialize impl, then serializes that as a string.
    ///
    /// ###  Deconstruction
    ///
    /// `DynTrait<_>` can then be unwrapped into a concrete type,
    /// within the same dynamic library/executable that constructed it,
    /// using these (fallible) conversion methods:
    ///
    /// - [`downcast_into`](#method.downcast_into):
    /// Unwraps into a pointer to `T`.Requires `T:'static`.
    ///
    /// - [`downcast_as`](#method.downcast_as):
    /// Unwraps into a `&T`.Requires `T:'static`.
    ///
    /// - [`downcast_as_mut`](#method.downcast_as_mut):
    /// Unwraps into a `&mut T`.Requires `T:'static`.
    ///
    ///
    /// `DynTrait` cannot be converted back if it was created
    /// using `DynTrait::from_borrowing_*`.
    ///
    /// # Passing DynTrait between dynamic libraries
    ///
    /// Passing DynTrait between dynamic libraries
    /// (as in between the dynamic libraries directly loaded by the same binary/dynamic library)
    /// may cause the program to panic at runtime with an error message stating that
    /// the trait is not implemented for the specific interface.
    ///
    /// This can only happen if you are passing DynTrait between dynamic libraries,
    /// or if DynTrait was instantiated in the parent passed to a child,
    /// a DynTrait instantiated in a child dynamic library passed to the parent
    /// should not cause a panic, it would be a bug.
    ///
    /// ```text
    ///         binary
    ///   _________|___________
    /// lib0      lib1      lib2
    ///   |         |         |
    /// lib00    lib10      lib20
    /// ```
    ///
    /// In this diagram passing a DynTrait constructed in lib00 to anything other than
    /// the binary or lib0 will cause the panic to happen if:
    ///
    /// - The [`InterfaceType`] requires extra traits in the version of the Interface
    ///     that lib1 and lib2 know about (that the binary does not require).
    ///
    /// - lib1 or lib2 attempt to call methods that require the traits that were added
    ///     to the [`InterfaceType`], in versions of that interface that only they know about.
    ///
    /// # Examples
    ///
    /// ###  In the Readme
    ///
    /// The primary example using `DynTrait<_>` is in the readme.
    ///
    /// Readme is in
    /// [the repository for this crate](https://github.com/rodrimati1992/abi_stable_crates),
    /// [crates.io](https://crates.io/crates/abi_stable),
    /// [lib.rs](https://lib.rs/crates/abi_stable).
    ///
    /// ### Serialization/Deserialization
    ///
    /// The [`DeserializeDyn`] and [`SerializeType`] demonstrate how `DynTrait`
    /// can be de/serialized.
    ///
    /// ###  Comparing DynTraits
    ///
    /// This is only possible if the erased types don't contain borrows,
    /// and they are not constructed using `DynTrait::from_borrowing_*` methods.
    ///
    /// DynTraits wrapping different pointer types can be compared with each other,
    /// it simply uses the values' implementation of PartialEq.
    ///
    /// ```
    /// use abi_stable::{
    ///     erased_types::interfaces::PartialEqInterface,
    ///     std_types::{RArc, RBox},
    ///     DynTrait, RMut, RRef,
    /// };
    ///
    /// {
    ///     // `DynTrait`s constructed from `&` are `DynTrait<'_, RRef<'_, ()>, _>`
    ///     // since `&T` can't soundly be transmuted back and forth into `&()`
    ///     let left: DynTrait<'static, RRef<'_, ()>, PartialEqInterface> =
    ///         DynTrait::from_ptr(&100);
    ///
    ///     let mut n100 = 100;
    ///     // `DynTrait`s constructed from `&mut` are `DynTrait<'_, RMut<'_, ()>, _>`
    ///     // since `&mut T` can't soundly be transmuted back and forth into `&mut ()`
    ///     let right: DynTrait<'static, RMut<'_, ()>, PartialEqInterface> =
    ///         DynTrait::from_ptr(&mut n100).interface(PartialEqInterface);
    ///
    ///     assert_eq!(left, right);
    /// }
    /// {
    ///     let left: DynTrait<'static, RBox<()>, PartialEqInterface> =
    ///         DynTrait::from_value(200);
    ///
    ///     let right: DynTrait<'static, RArc<()>, _> =
    ///         DynTrait::from_ptr(RArc::new(200)).interface(PartialEqInterface);
    ///
    ///     assert_eq!(left, right);
    /// }
    ///
    /// ```
    ///
    /// ###  Writing to a DynTrait
    ///
    /// This is an example of using the `write!()` macro with DynTrait.
    ///
    /// ```
    /// use abi_stable::{erased_types::interfaces::FmtWriteInterface, DynTrait, RMut};
    ///
    /// use std::fmt::Write;
    ///
    /// let mut buffer = String::new();
    ///
    /// let mut wrapped: DynTrait<'static, RMut<'_, ()>, FmtWriteInterface> =
    ///     DynTrait::from_ptr(&mut buffer);
    ///
    /// write!(wrapped, "Foo").unwrap();
    /// write!(wrapped, "Bar").unwrap();
    /// write!(wrapped, "Baz").unwrap();
    ///
    /// drop(wrapped);
    ///
    /// assert_eq!(&buffer[..], "FooBarBaz");
    ///
    ///
    /// ```
    ///
    ///
    /// ###  Iteration
    ///
    /// Using `DynTrait` as an `Iterator` and `DoubleEndedIterator`.
    ///
    /// ```
    /// use abi_stable::{erased_types::interfaces::DEIteratorInterface, DynTrait};
    ///
    /// let mut wrapped = DynTrait::from_value(0..=10).interface(DEIteratorInterface::NEW);
    ///
    /// assert_eq!(
    ///     wrapped.by_ref().take(5).collect::<Vec<_>>(),
    ///     vec![0, 1, 2, 3, 4]
    /// );
    ///
    /// assert_eq!(wrapped.rev().collect::<Vec<_>>(), vec![10, 9, 8, 7, 6, 5]);
    ///
    /// ```
    ///
    ///
    /// # Making pointers compatible with DynTrait
    ///
    /// To make pointers compatible with DynTrait, they must imlement the
    /// `abi_stable::pointer_trait::{`
    /// [`GetPointerKind`]`, `[`AsPtr`]`, `[`AsMutPtr`]`, `[`CanTransmuteElement`]`}`
    /// traits as shown in the example.
    ///
    /// [`GetPointerKind`] should generally be implemented with
    /// `type Kind = `[`PK_SmartPointer`].
    /// The exception is in the case that it is a `#[repr(transparent)]`
    /// wrapper around a [`RRef`]/[`RMut`]/`*const T`/`*mut T`/[`NonNull`],
    /// in which case it should implement [`GetPointerKind`]`<Kind = `[`PK_Reference`]`>`
    /// (when it has shared reference semantics)
    /// or [`GetPointerKind`]`<Kind = `[`PK_MutReference`]`>`
    /// (when it has mutable reference semantics).
    ///
    /// ###  Example
    ///
    /// This is an example of a newtype wrapping an [`RBox`],
    /// demonstrating that the pointer type doesn't have to implement
    /// the traits in the [`InterfaceType`], it's the value it points to.
    ///
    /// ```rust
    ///     
    /// use abi_stable::DynTrait;
    ///
    /// fn main() {
    ///     let lines = "line0\nline1\nline2";
    ///     let mut iter = NewtypeBox::new(lines.lines());
    ///
    ///     // The type annotation here is just to show the type, it's not necessary.
    ///     let mut wrapper: DynTrait<'_, NewtypeBox<()>, IteratorInterface> =
    ///         DynTrait::from_borrowing_ptr(iter);
    ///
    ///     // You can clone the DynTrait!
    ///     let clone = wrapper.clone();
    ///
    ///     assert_eq!(wrapper.next(), Some("line0"));
    ///     assert_eq!(wrapper.next(), Some("line1"));
    ///     assert_eq!(wrapper.next(), Some("line2"));
    ///     assert_eq!(wrapper.next(), None);
    ///
    ///     assert_eq!(
    ///         clone.rev().collect::<Vec<_>>(),
    ///         vec!["line2", "line1", "line0"],
    ///     )
    /// }
    ///
    /// #[repr(C)]
    /// #[derive(StableAbi)]
    /// #[sabi(impl_InterfaceType(
    ///     Sync,
    ///     Send,
    ///     Iterator,
    ///     DoubleEndedIterator,
    ///     Clone,
    ///     Debug
    /// ))]
    /// pub struct IteratorInterface;
    ///
    /// impl<'a> IteratorItem<'a> for IteratorInterface {
    ///     type Item = &'a str;
    /// }
    ///
    /// /////////////////////////////////////////
    ///
    /// use std::ops::{Deref, DerefMut};
    ///
    /// use abi_stable::{
    ///     erased_types::IteratorItem,
    ///     pointer_trait::{
    ///         AsMutPtr, AsPtr, CanTransmuteElement, GetPointerKind, PK_SmartPointer,
    ///     },
    ///     std_types::RBox,
    ///     type_level::bools::True,
    ///     InterfaceType, StableAbi,
    /// };
    ///
    /// #[repr(transparent)]
    /// #[derive(Default, Clone, StableAbi)]
    /// pub struct NewtypeBox<T> {
    ///     box_: RBox<T>,
    /// }
    ///
    /// impl<T> NewtypeBox<T> {
    ///     pub fn new(value: T) -> Self {
    ///         Self {
    ///             box_: RBox::new(value),
    ///         }
    ///     }
    /// }
    ///
    /// unsafe impl<T> GetPointerKind for NewtypeBox<T> {
    ///     // This is a smart pointer because `RBox` is one.
    ///     type Kind = PK_SmartPointer;
    ///     type PtrTarget = T;
    /// }
    ///
    /// // safety: Does not create an intermediate `&T` to get a pointer to `T`.
    /// unsafe impl<T> AsPtr for NewtypeBox<T> {
    ///     fn as_ptr(&self) -> *const T {
    ///         self.box_.as_ptr()
    ///     }
    /// }
    ///
    /// // safety: Does not create an intermediate `&mut T` to get a pointer to `T`
    /// unsafe impl<T> AsMutPtr for NewtypeBox<T> {
    ///     fn as_mut_ptr(&mut self) -> *mut T {
    ///         self.box_.as_mut_ptr()
    ///     }
    /// }
    ///
    /// // safety:
    /// // NewtypeBox is safe to transmute, because RBox (the pointer type it wraps)
    /// // is safe to transmute
    /// unsafe impl<T, O> CanTransmuteElement<O> for NewtypeBox<T> {
    ///     type TransmutedPtr = NewtypeBox<O>;
    ///
    ///     unsafe fn transmute_element_(self) -> Self::TransmutedPtr {
    ///         let box_: RBox<O> = self.box_.transmute_element_();
    ///         NewtypeBox { box_ }
    ///     }
    /// }
    ///
    /// ```
    ///
    /// [`AsPtr`]: crate::pointer_trait::AsPtr
    /// [`InterfaceType`]: crate::InterfaceType
    /// [`NonNull`]: https://doc.rust-lang.org/std/ptr/struct.NonNull.html
    /// [`SerializeProxyType`]: crate::erased_types::SerializeProxyType
    /// [`DeserializeDyn`]: crate::erased_types::DeserializeDyn
    /// [`AsMutPtr`]: crate::pointer_trait::AsMutPtr
    /// [`CanTransmuteElement`]: crate::pointer_trait::CanTransmuteElement
    /// [`GetPointerKind`]: crate::pointer_trait::GetPointerKind
    /// [`RRef`]: crate::sabi_types::RRef
    /// [`RMut`]: crate::sabi_types::RMut
    /// [`RBox`]: crate::std_types::RBox
    /// [`PK_Reference`]: crate::pointer_trait::PK_Reference
    /// [`PK_MutReference`]: crate::pointer_trait::PK_MutReference
    /// [`PK_SmartPointer`]: crate::pointer_trait::PK_SmartPointer
    /// [`SerializeType`]: crate::erased_types::SerializeType
    ///
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(
        // debug_print,
        bound(I: InterfaceType),
        bound(VTable_Ref<'borr, P, I>: StableAbi),
        extra_checks = <I as MakeRequiredTraits>::MAKE,
    )]
    pub struct DynTrait<'borr, P, I, EV = ()>
    where
        P: GetPointerKind,
    {
        pub(super) object: ManuallyDrop<P>,
        vtable: VTable_Ref<'borr, P, I>,
        extra_value: EV,
        _marker: NonOwningPhantom<(I, RStr<'borr>)>,
        _marker2: UnsafeIgnoredType<Rc<()>>,
    }

    impl<I> DynTrait<'static, RBox<()>, I> {
        /// Constructs the `DynTrait<_>` from a type that doesn't borrow anything.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{
        ///     erased_types::interfaces::DebugDisplayInterface, std_types::RBox, DynTrait,
        /// };
        ///
        /// // DebugDisplayInterface is `Debug + Display + Sync + Send`
        /// let to: DynTrait<'static, RBox<()>, DebugDisplayInterface> =
        ///     DynTrait::from_value(3u8);
        ///
        /// assert_eq!(format!("{}", to), "3");
        /// assert_eq!(format!("{:?}", to), "3");
        ///
        /// ```
        pub fn from_value<T>(object: T) -> Self
        where
            T: 'static,
            VTable_Ref<'static, RBox<()>, I>: MakeVTable<'static, T, RBox<T>, TD_CanDowncast>,
        {
            let object = RBox::new(object);
            DynTrait::from_ptr(object)
        }
    }

    impl<P, I> DynTrait<'static, P, I>
    where
        P: GetPointerKind,
    {
        /// Constructs the `DynTrait<_>` from a pointer to a
        /// type that doesn't borrow anything.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{
        ///     erased_types::interfaces::DebugDisplayInterface,
        ///     std_types::{RArc, RBox},
        ///     DynTrait, RMut, RRef,
        /// };
        ///
        /// // Constructing a DynTrait from a `&T`
        /// {
        ///     // `DynTrait`s constructed from `&` are `DynTrait<'_, RRef<'_, ()>, _>`
        ///     // since `&T` can't soundly be transmuted back and forth into `&()`
        ///     let rref: DynTrait<'static, RRef<'_, ()>, DebugDisplayInterface> =
        ///         DynTrait::from_ptr(&21i32);
        ///
        ///     assert_eq!(format!("{:?}", rref), "21");
        ///     assert_eq!(format!("{}", rref), "21");
        /// }
        /// // Constructing a DynTrait from a `&mut T`
        /// {
        ///     let mmut = &mut "hello";
        ///     // `DynTrait`s constructed from `&mut` are `DynTrait<'_, RMut<'_, ()>, _>`
        ///     // since `&mut T` can't soundly be transmuted back and forth into `&mut ()`
        ///     let rmut: DynTrait<'static, RMut<'_, ()>, DebugDisplayInterface> =
        ///         DynTrait::from_ptr(mmut).interface(DebugDisplayInterface);
        ///
        ///     assert_eq!(format!("{:?}", rmut), r#""hello""#);
        ///     assert_eq!(format!("{}", rmut), "hello");
        /// }
        /// // Constructing a DynTrait from a `RBox<T>`
        /// {
        ///     let boxed: DynTrait<'static, RBox<()>, DebugDisplayInterface> =
        ///         DynTrait::from_ptr(RBox::new(false));
        ///
        ///     assert_eq!(format!("{:?}", boxed), "false");
        ///     assert_eq!(format!("{}", boxed), "false");
        /// }
        /// // Constructing a DynTrait from an `RArc<T>`
        /// {
        ///     let arc: DynTrait<'static, RArc<()>, DebugDisplayInterface> =
        ///         DynTrait::from_ptr(RArc::new(30u32)).interface(DebugDisplayInterface);
        ///
        ///     assert_eq!(format!("{:?}", arc), "30");
        /// }
        ///
        /// ```
        pub fn from_ptr<OrigPtr>(object: OrigPtr) -> Self
        where
            OrigPtr: GetPointerKind,
            OrigPtr::PtrTarget: 'static,
            OrigPtr: CanTransmuteElement<(), TransmutedPtr = P>,
            VTable_Ref<'static, P, I>:
                MakeVTable<'static, OrigPtr::PtrTarget, OrigPtr, TD_CanDowncast>,
        {
            DynTrait {
                object: unsafe { ManuallyDrop::new(object.transmute_element::<()>()) },
                vtable: VTable_Ref::VTABLE_REF,
                extra_value: (),
                _marker: NonOwningPhantom::NEW,
                _marker2: UnsafeIgnoredType::DEFAULT,
            }
        }
    }

    impl<'borr, I> DynTrait<'borr, RBox<()>, I> {
        /// Constructs the `DynTrait<_>` from a value with a `'borr` borrow.
        ///
        /// Cannot downcast the DynTrait afterwards.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{
        ///     erased_types::interfaces::DebugDisplayInterface, std_types::RBox, DynTrait,
        /// };
        ///
        /// // DebugDisplayInterface is `Debug + Display + Sync + Send`
        /// let to: DynTrait<'static, RBox<()>, DebugDisplayInterface> =
        ///     DynTrait::from_borrowing_value(3u8);
        ///
        /// assert_eq!(format!("{}", to), "3");
        /// assert_eq!(format!("{:?}", to), "3");
        ///
        /// // `DynTrait`s constructed using the `from_borrowing_*` constructors
        /// // can't be downcasted.
        /// assert_eq!(to.downcast_as::<u8>().ok(), None);
        ///
        /// ```
        pub fn from_borrowing_value<T>(object: T) -> Self
        where
            T: 'borr,
            VTable_Ref<'borr, RBox<()>, I>: MakeVTable<'borr, T, RBox<T>, TD_Opaque>,
        {
            let object = RBox::new(object);
            DynTrait::from_borrowing_ptr(object)
        }
    }

    impl<'borr, P, I> DynTrait<'borr, P, I>
    where
        P: GetPointerKind,
    {
        /// Constructs the `DynTrait<_>` from a pointer to the erased type
        /// with a `'borr` borrow.
        ///
        /// Cannot downcast the DynTrait afterwards.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{
        ///     erased_types::interfaces::DebugDisplayInterface,
        ///     std_types::{RArc, RBox},
        ///     DynTrait, RMut, RRef,
        /// };
        ///
        /// // Constructing a DynTrait from a `&T`
        /// {
        ///     // `DynTrait`s constructed from `&` are `DynTrait<'_, RRef<'_, ()>, _>`
        ///     // since `&T` can't soundly be transmuted back and forth into `&()`
        ///     let rref: DynTrait<'_, RRef<'_, ()>, DebugDisplayInterface> =
        ///         DynTrait::from_borrowing_ptr(&34i32);
        ///
        ///     assert_eq!(format!("{:?}", rref), "34");
        ///     assert_eq!(format!("{}", rref), "34");
        /// }
        /// // Constructing a DynTrait from a `&mut T`
        /// {
        ///     let mmut = &mut "world";
        ///     // `DynTrait`s constructed from `&mut` are `DynTrait<'_, RMut<'_, ()>, _>`
        ///     // since `&mut T` can't soundly be transmuted back and forth into `&mut ()`
        ///     let rmut: DynTrait<'_, RMut<'_, ()>, DebugDisplayInterface> =
        ///         DynTrait::from_borrowing_ptr(mmut).interface(DebugDisplayInterface);
        ///
        ///     assert_eq!(format!("{:?}", rmut), r#""world""#);
        ///     assert_eq!(format!("{}", rmut), "world");
        /// }
        /// // Constructing a DynTrait from a `RBox<T>`
        /// {
        ///     let boxed: DynTrait<'_, RBox<()>, DebugDisplayInterface> =
        ///         DynTrait::from_borrowing_ptr(RBox::new(true));
        ///
        ///     assert_eq!(format!("{:?}", boxed), "true");
        ///     assert_eq!(format!("{}", boxed), "true");
        /// }
        /// // Constructing a DynTrait from an `RArc<T>`
        /// {
        ///     let arc: DynTrait<'_, RArc<()>, _> =
        ///         DynTrait::from_borrowing_ptr(RArc::new('a')).interface(DebugDisplayInterface);
        ///
        ///     assert_eq!(format!("{:?}", arc), "'a'");
        ///     assert_eq!(format!("{}", arc), "a");
        /// }
        ///
        /// ```
        pub fn from_borrowing_ptr<OrigPtr>(object: OrigPtr) -> Self
        where
            OrigPtr: GetPointerKind + 'borr,
            OrigPtr::PtrTarget: 'borr,
            OrigPtr: CanTransmuteElement<(), TransmutedPtr = P>,
            VTable_Ref<'borr, P, I>: MakeVTable<'borr, OrigPtr::PtrTarget, OrigPtr, TD_Opaque>,
        {
            DynTrait {
                object: unsafe { ManuallyDrop::new(object.transmute_element::<()>()) },
                vtable: VTable_Ref::VTABLE_REF,
                extra_value: (),
                _marker: NonOwningPhantom::NEW,
                _marker2: UnsafeIgnoredType::DEFAULT,
            }
        }
    }

    impl<'borr, P, I, EV> DynTrait<'borr, P, I, EV>
    where
        P: AsPtr<PtrTarget = ()>,
    {
        /// Constructs an DynTrait from an erasable pointer and an extra value.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{
        ///     erased_types::{interfaces::DebugDisplayInterface, TD_Opaque},
        ///     DynTrait, RRef,
        /// };
        ///
        /// // DebugDisplayInterface is `Debug + Display + Sync + Send`
        /// let to: DynTrait<'static, RRef<()>, DebugDisplayInterface, usize> =
        ///     DynTrait::with_extra_value::<_, TD_Opaque>(&55u8, 100usize);
        ///
        /// assert_eq!(format!("{}", to), "55");
        /// assert_eq!(format!("{:?}", to), "55");
        ///
        /// assert_eq!(to.sabi_extra_value(), &100);
        ///
        /// ```
        pub fn with_extra_value<OrigPtr, Downcasting>(
            ptr: OrigPtr,
            extra_value: EV,
        ) -> DynTrait<'borr, P, I, EV>
        where
            OrigPtr: GetPointerKind,
            OrigPtr::PtrTarget: 'borr,
            OrigPtr: CanTransmuteElement<(), TransmutedPtr = P>,
            VTable_Ref<'borr, P, I>: MakeVTable<'borr, OrigPtr::PtrTarget, OrigPtr, Downcasting>,
        {
            DynTrait {
                object: unsafe { ManuallyDrop::new(ptr.transmute_element::<()>()) },
                vtable: VTable_Ref::VTABLE_REF,
                extra_value,
                _marker: NonOwningPhantom::NEW,
                _marker2: UnsafeIgnoredType::DEFAULT,
            }
        }

        #[doc(hidden)]
        pub fn with_vtable<OrigPtr, Downcasting>(
            ptr: OrigPtr,
            extra_vtable: EV,
        ) -> DynTrait<'borr, P, I, EV>
        where
            OrigPtr: GetPointerKind,
            OrigPtr::PtrTarget: 'borr,
            I: InterfaceType,
            OrigPtr: CanTransmuteElement<(), TransmutedPtr = P>,
            VTable_Ref<'borr, P, I>: MakeVTable<'borr, OrigPtr::PtrTarget, OrigPtr, Downcasting>,
        {
            Self::with_extra_value(ptr, extra_vtable)
        }
    }

    impl<'borr, 'a, I, EV> DynTrait<'borr, RRef<'a, ()>, I, EV> {
        /// This function allows constructing a DynTrait in a constant/static.
        ///
        /// # Parameters
        ///
        /// `ptr`: a reference to the value.
        ///
        /// <br>
        ///
        /// `can_it_downcast` can be either:
        ///
        /// - [`TD_CanDowncast`]:
        ///     Which allows the trait object to be downcasted, requires that the value implements any.
        ///
        /// - [`TD_Opaque`]:
        ///     Which does not allow the trait object to be downcasted.
        ///
        /// <br>
        ///
        /// `extra_value`:
        /// This is used by [`#[sabi_trait]`](macro@crate::sabi_trait)
        /// trait objects to store their vtable inside DynTrait.
        ///
        ///
        /// # Example
        ///
        /// ```
        /// use abi_stable::{
        ///     erased_types::{
        ///         interfaces::DebugDisplayInterface, DynTrait, TD_Opaque,
        ///     },
        ///     sabi_types::RRef,
        /// };
        ///
        /// static STRING: &str = "What the heck";
        ///
        /// static DYN: DynTrait<'static, RRef<'static, ()>, DebugDisplayInterface, ()> =
        ///     DynTrait::from_const(&STRING, TD_Opaque, ());
        ///
        /// fn main() {
        ///     assert_eq!(format!("{}", DYN), format!("{}", STRING));
        ///     assert_eq!(format!("{:?}", DYN), format!("{:?}", STRING));
        /// }
        ///
        /// ```
        ///
        /// [`TD_CanDowncast`]: ./type_level/downcasting/struct.TD_CanDowncast.html
        /// [`TD_Opaque`]: ./type_level/downcasting/struct.TD_Opaque.html
        pub const fn from_const<T, Downcasting>(
            ptr: &'a T,
            can_it_downcast: Downcasting,
            extra_value: EV,
        ) -> Self
        where
            VTable_Ref<'borr, RRef<'a, ()>, I>: MakeVTable<'borr, T, &'a T, Downcasting>,
            T: 'borr,
        {
            // Must wrap can_it_downcast in a ManuallyDrop because otherwise this
            // errors with `constant functions cannot evaluate destructors`.
            let _ = ManuallyDrop::new(can_it_downcast);
            DynTrait {
                object: unsafe {
                    let x = RRef::from_raw(ptr as *const T as *const ());
                    ManuallyDrop::new(x)
                },
                vtable: VTable_Ref::VTABLE_REF,
                extra_value,
                _marker: NonOwningPhantom::NEW,
                _marker2: UnsafeIgnoredType::DEFAULT,
            }
        }
    }

    impl<'borr, P, I, EV> DynTrait<'borr, P, I, EV>
    where
        P: GetPointerKind,
    {
        /// For inferring the `I` type parameter.
        pub const fn interface(self, _interface: I) -> Self
        where
            I: InterfaceType,
        {
            std::mem::forget(_interface);
            self
        }
    }

    impl<P, I, EV> DynTrait<'static, P, I, EV>
    where
        P: GetPointerKind,
    {
        /// Allows checking whether 2 `DynTrait<_>`s have a value of the same type.
        ///
        /// Notes:
        ///
        /// - Types from different dynamic libraries/executables are
        /// never considered equal.
        ///
        /// - `DynTrait`s constructed using `DynTrait::from_borrowing_*`
        /// are never considered to wrap the same type.
        pub fn sabi_is_same_type<Other, I2, EV2>(
            &self,
            other: &DynTrait<'static, Other, I2, EV2>,
        ) -> bool
        where
            I2: InterfaceType,
            Other: GetPointerKind,
        {
            self.sabi_vtable_address() == other.sabi_vtable_address()
                || self
                    .sabi_vtable()
                    .type_info()
                    .is_compatible(other.sabi_vtable().type_info())
        }
    }

    impl<'borr, P, I, EV> DynTrait<'borr, P, I, PrefixRef<EV>>
    where
        P: GetPointerKind,
    {
        /// A vtable used by `#[sabi_trait]` derived trait objects.
        #[inline]
        pub const fn sabi_et_vtable(&self) -> PrefixRef<EV> {
            self.extra_value
        }
    }

    impl<'borr, P, I, EV> DynTrait<'borr, P, I, EV>
    where
        P: GetPointerKind,
    {
        /// Gets access to the extra value that was stored in this DynTrait in the
        /// `with_extra_value` constructor.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{erased_types::TD_Opaque, DynTrait, RRef};
        ///
        /// let to: DynTrait<'static, RRef<()>, (), char> =
        ///     DynTrait::with_extra_value::<_, TD_Opaque>(&55u8, 'Z');
        ///
        /// assert_eq!(to.sabi_extra_value(), &'Z');
        ///
        /// ```
        #[inline]
        pub const fn sabi_extra_value(&self) -> &EV {
            &self.extra_value
        }

        #[inline]
        pub(super) const fn sabi_vtable(&self) -> VTable_Ref<'borr, P, I> {
            self.vtable
        }

        #[inline]
        pub(super) fn sabi_vtable_address(&self) -> usize {
            self.vtable.0.to_raw_ptr() as usize
        }

        /// Returns the address of the wrapped object.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{erased_types::TD_Opaque, DynTrait, RRef};
        ///
        /// let reff = &55u8;
        ///
        /// let to: DynTrait<'static, RRef<()>, ()> = DynTrait::from_ptr(reff);
        ///
        /// assert_eq!(to.sabi_object_address(), reff as *const _ as usize);
        ///
        /// ```
        pub fn sabi_object_address(&self) -> usize
        where
            P: AsPtr,
        {
            self.object.as_ptr() as *const () as usize
        }

        // Safety: Only call this in unerasure functions
        unsafe fn sabi_object_as<T>(&self) -> &T
        where
            P: AsPtr,
        {
            unsafe { &*(self.object.as_ptr() as *const P::PtrTarget as *const T) }
        }

        // Safety: Only call this in unerasure functions
        unsafe fn sabi_object_as_mut<T>(&mut self) -> &mut T
        where
            P: AsMutPtr,
        {
            unsafe { &mut *(self.object.as_mut_ptr() as *mut P::PtrTarget as *mut T) }
        }

        /// Gets a reference pointing to the erased object.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{std_types::RBox, DynTrait};
        ///
        /// let to: DynTrait<'static, RBox<()>, ()> = DynTrait::from_value(66u8);
        ///
        /// unsafe {
        ///     assert_eq!(to.sabi_erased_ref().transmute_into_ref::<u8>(), &66);
        /// }
        /// ```
        pub fn sabi_erased_ref(&self) -> RRef<'_, ErasedObject>
        where
            P: AsPtr,
        {
            unsafe { RRef::from_raw(self.object.as_ptr() as *const ErasedObject) }
        }

        pub(super) unsafe fn sabi_erased_ref_unbounded_lifetime<'a>(&self) -> RRef<'a, ErasedObject>
        where
            P: AsPtr,
        {
            unsafe { RRef::from_raw(self.object.as_ptr() as *const ErasedObject) }
        }

        /// Gets a mutable reference pointing to the erased object.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{std_types::RBox, DynTrait};
        ///
        /// let mut to: DynTrait<'static, RBox<()>, ()> = DynTrait::from_value("hello");
        ///
        /// unsafe {
        ///     assert_eq!(
        ///         to.sabi_erased_mut().transmute_into_mut::<&str>(),
        ///         &mut "hello"
        ///     );
        /// }
        /// ```
        #[inline]
        pub fn sabi_erased_mut(&mut self) -> RMut<'_, ErasedObject>
        where
            P: AsMutPtr,
        {
            unsafe { RMut::from_raw(self.object.as_mut_ptr() as *mut ErasedObject) }
        }

        /// Gets an `RRef` pointing to the erased object.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{std_types::RBox, DynTrait};
        ///
        /// let to: DynTrait<'static, RBox<()>, ()> = DynTrait::from_value(66u8);
        ///
        /// unsafe {
        ///     assert_eq!(to.sabi_as_rref().transmute_into_ref::<u8>(), &66);
        /// }
        /// ```
        pub fn sabi_as_rref(&self) -> RRef<'_, ()>
        where
            P: AsPtr,
        {
            unsafe { RRef::from_raw(self.object.as_ptr() as *const ()) }
        }

        /// Gets an `RMut` pointing to the erased object.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{std_types::RBox, DynTrait};
        ///
        /// let mut to: DynTrait<'static, RBox<()>, ()> = DynTrait::from_value("hello");
        ///
        /// unsafe {
        ///     assert_eq!(to.sabi_as_rmut().transmute_into_mut::<&str>(), &mut "hello");
        /// }
        /// ```
        pub fn sabi_as_rmut(&mut self) -> RMut<'_, ()>
        where
            P: AsMutPtr,
        {
            unsafe { RMut::from_raw(self.object.as_mut_ptr() as *mut ()) }
        }

        #[inline]
        fn sabi_into_erased_ptr(self) -> ManuallyDrop<P> {
            let this = ManuallyDrop::new(self);
            unsafe { ptr::read(&this.object) }
        }

        /// Calls the `f` callback with an `MovePtr` pointing to the erased object.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{
        ///     sabi_types::MovePtr,
        ///     std_types::{RBox, RString, RVec},
        ///     DynTrait,
        /// };
        ///
        /// let to: DynTrait<'static, RBox<()>, ()> =
        ///     DynTrait::from_value(RString::from("foobarbaz"));
        ///
        /// let string = to.sabi_with_value(|x| {
        ///     // SAFETY: the erased object is an RString constructed in the current binary.
        ///     unsafe {
        ///         MovePtr::into_inner(MovePtr::transmute::<RString>(x))
        ///     }
        /// });
        ///
        /// assert_eq!(string, "foobarbaz");
        /// ```
        #[inline]
        pub fn sabi_with_value<F, R>(self, f: F) -> R
        where
            P: OwnedPointer<PtrTarget = ()>,
            F: FnOnce(MovePtr<'_, ()>) -> R,
        {
            OwnedPointer::with_move_ptr(self.sabi_into_erased_ptr(), f)
        }
    }

    impl<'borr, P, I, EV> DynTrait<'borr, P, I, EV>
    where
        P: GetPointerKind,
    {
        /// The uid in the vtable has to be the same as the one for T,
        /// otherwise it was not created from that T in the library that declared the opaque type.
        pub(super) fn sabi_check_same_destructor<T>(&self) -> Result<(), UneraseError<()>>
        where
            T: 'static,
        {
            let t_info = &TypeInfoFor::<T, (), TD_CanDowncast>::INFO;
            if self.sabi_vtable().type_info().is_compatible(t_info) {
                Ok(())
            } else {
                Err(UneraseError {
                    dyn_trait: (),
                    expected_type_info: t_info,
                    found_type_info: self.sabi_vtable().type_info(),
                })
            }
        }

        /// Unwraps the `DynTrait<_>` into a pointer of
        /// the concrete type that it was constructed with.
        ///
        /// `T` is required to not borrow anything.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - The DynTrait was constructed using a `from_borrowing_*` method
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{
        ///     std_types::{RArc, RBox},
        ///     DynTrait,
        /// };
        ///
        /// {
        ///     fn to() -> DynTrait<'static, RBox<()>, ()> {
        ///         DynTrait::from_value(b'A')
        ///     }
        ///
        ///     assert_eq!(to().downcast_into::<u8>().ok(), Some(RBox::new(b'A')));
        ///     assert_eq!(to().downcast_into::<u16>().ok(), None);
        /// }
        /// {
        ///     fn to() -> DynTrait<'static, RArc<()>, ()> {
        ///         DynTrait::from_ptr(RArc::new(b'B'))
        ///     }
        ///
        ///     assert_eq!(to().downcast_into::<u8>().ok(), Some(RArc::new(b'B')));
        ///     assert_eq!(to().downcast_into::<u16>().ok(), None);
        /// }
        ///
        /// ```
        pub fn downcast_into<T>(self) -> Result<P::TransmutedPtr, UneraseError<Self>>
        where
            T: 'static,
            P: CanTransmuteElement<T>,
        {
            check_unerased!(self, self.sabi_check_same_destructor::<T>());
            unsafe {
                let this = ManuallyDrop::new(self);
                Ok(ptr::read(&*this.object).transmute_element::<T>())
            }
        }

        /// Unwraps the `DynTrait<_>` into a reference of
        /// the concrete type that it was constructed with.
        ///
        /// `T` is required to not borrow anything.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - The DynTrait was constructed using a `from_borrowing_*` method
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{std_types::RArc, DynTrait, RMut, RRef};
        ///
        /// {
        ///     let to: DynTrait<'static, RRef<'_, ()>, ()> = DynTrait::from_ptr(&9u8);
        ///
        ///     assert_eq!(to.downcast_as::<u8>().ok(), Some(&9u8));
        ///     assert_eq!(to.downcast_as::<u16>().ok(), None);
        /// }
        /// {
        ///     let mut val = 7u8;
        ///
        ///     let to: DynTrait<'static, RMut<'_, ()>, ()> =
        ///         DynTrait::from_ptr(&mut val);
        ///
        ///     assert_eq!(to.downcast_as::<u8>().ok(), Some(&7));
        ///     assert_eq!(to.downcast_as::<u16>().ok(), None);
        /// }
        /// {
        ///     let to: DynTrait<'static, RArc<()>, ()> =
        ///         DynTrait::from_ptr(RArc::new(1u8));
        ///
        ///     assert_eq!(to.downcast_as::<u8>().ok(), Some(&1u8));
        ///     assert_eq!(to.downcast_as::<u16>().ok(), None);
        /// }
        ///
        /// ```
        pub fn downcast_as<T>(&self) -> Result<&T, UneraseError<&Self>>
        where
            T: 'static,
            P: AsPtr,
        {
            check_unerased!(self, self.sabi_check_same_destructor::<T>());
            unsafe { Ok(self.sabi_object_as()) }
        }

        /// Unwraps the `DynTrait<_>` into a mutable reference of
        /// the concrete type that it was constructed with.
        ///
        /// `T` is required to not borrow anything.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - The DynTrait was constructed using a `from_borrowing_*` method
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{std_types::RBox, DynTrait, RMut};
        ///
        /// {
        ///     let mut val = 7u8;
        ///
        ///     let mut to: DynTrait<'static, RMut<'_, ()>, ()> =
        ///         DynTrait::from_ptr(&mut val);
        ///
        ///     assert_eq!(to.downcast_as_mut::<u8>().ok(), Some(&mut 7));
        ///     assert_eq!(to.downcast_as_mut::<u16>().ok(), None);
        /// }
        /// {
        ///     let mut to: DynTrait<'static, RBox<()>, ()> =
        ///         DynTrait::from_ptr(RBox::new(1u8));
        ///
        ///     assert_eq!(to.downcast_as_mut::<u8>().ok(), Some(&mut 1u8));
        ///     assert_eq!(to.downcast_as_mut::<u16>().ok(), None);
        /// }
        ///
        /// ```
        pub fn downcast_as_mut<T>(&mut self) -> Result<&mut T, UneraseError<&mut Self>>
        where
            T: 'static,
            P: AsMutPtr,
        {
            check_unerased!(self, self.sabi_check_same_destructor::<T>());
            unsafe { Ok(self.sabi_object_as_mut()) }
        }

        /// Unwraps the `DynTrait<_>` into a pointer to T,
        /// without checking whether `T` is the type that the DynTrait was constructed with.
        ///
        /// # Safety
        ///
        /// You must check that `T` is the type that DynTrait was constructed
        /// with through other means.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{
        ///     std_types::{RArc, RBox},
        ///     DynTrait,
        /// };
        ///
        /// unsafe {
        ///     fn to() -> DynTrait<'static, RBox<()>, ()> {
        ///         DynTrait::from_value(b'A')
        ///     }
        ///
        ///     assert_eq!(to().unchecked_downcast_into::<u8>(), RBox::new(b'A'));
        /// }
        /// unsafe {
        ///     fn to() -> DynTrait<'static, RArc<()>, ()> {
        ///         DynTrait::from_ptr(RArc::new(b'B'))
        ///     }
        ///
        ///     assert_eq!(to().unchecked_downcast_into::<u8>(), RArc::new(b'B'));
        /// }
        ///
        /// ```
        #[inline]
        pub unsafe fn unchecked_downcast_into<T>(self) -> P::TransmutedPtr
        where
            P: AsPtr + CanTransmuteElement<T>,
        {
            let this = ManuallyDrop::new(self);
            unsafe { ptr::read(&*this.object).transmute_element::<T>() }
        }

        /// Unwraps the `DynTrait<_>` into a reference to T,
        /// without checking whether `T` is the type that the DynTrait was constructed with.
        ///
        /// # Safety
        ///
        /// You must check that `T` is the type that DynTrait was constructed
        /// with through other means.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{std_types::RArc, DynTrait, RMut, RRef};
        ///
        /// unsafe {
        ///     let to: DynTrait<'static, RRef<'_, ()>, ()> = DynTrait::from_ptr(&9u8);
        ///
        ///     assert_eq!(to.unchecked_downcast_as::<u8>(), &9u8);
        /// }
        /// unsafe {
        ///     let mut val = 7u8;
        ///
        ///     let to: DynTrait<'static, RMut<'_, ()>, ()> =
        ///         DynTrait::from_ptr(&mut val);
        ///
        ///     assert_eq!(to.unchecked_downcast_as::<u8>(), &7);
        /// }
        /// unsafe {
        ///     let to: DynTrait<'static, RArc<()>, ()> =
        ///         DynTrait::from_ptr(RArc::new(1u8));
        ///
        ///     assert_eq!(to.unchecked_downcast_as::<u8>(), &1u8);
        /// }
        ///
        /// ```
        #[inline]
        pub unsafe fn unchecked_downcast_as<T>(&self) -> &T
        where
            P: AsPtr,
        {
            unsafe { self.sabi_object_as() }
        }

        /// Unwraps the `DynTrait<_>` into a mutable reference to T,
        /// without checking whether `T` is the type that the DynTrait was constructed with.
        ///
        /// # Safety
        ///
        /// You must check that `T` is the type that DynTrait was constructed
        /// with through other means.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{std_types::RBox, DynTrait, RMut};
        ///
        /// unsafe {
        ///     let mut val = 7u8;
        ///
        ///     let mut to: DynTrait<'static, RMut<'_, ()>, ()> =
        ///         DynTrait::from_ptr(&mut val);
        ///
        ///     assert_eq!(to.unchecked_downcast_as_mut::<u8>(), &mut 7);
        /// }
        /// unsafe {
        ///     let mut to: DynTrait<'static, RBox<()>, ()> =
        ///         DynTrait::from_ptr(RBox::new(1u8));
        ///
        ///     assert_eq!(to.unchecked_downcast_as_mut::<u8>(), &mut 1u8);
        /// }
        ///
        /// ```
        #[inline]
        pub unsafe fn unchecked_downcast_as_mut<T>(&mut self) -> &mut T
        where
            P: AsMutPtr,
        {
            unsafe { self.sabi_object_as_mut() }
        }
    }

    mod private_struct {
        pub struct PrivStruct;
    }
    use self::private_struct::PrivStruct;

    /// This is used to make sure that reborrowing does not change
    /// the Send-ness or Sync-ness of the pointer.
    pub trait ReborrowBounds<SendNess, SyncNess> {}

    // If it's reborrowing, it must have either both Sync+Send or neither.
    impl ReborrowBounds<Unimplemented<trait_marker::Send>, Unimplemented<trait_marker::Sync>>
        for PrivStruct
    {
    }

    impl ReborrowBounds<Implemented<trait_marker::Send>, Implemented<trait_marker::Sync>>
        for PrivStruct
    {
    }

    impl<'borr, P, I, EV> DynTrait<'borr, P, I, EV>
    where
        P: GetPointerKind,
        I: InterfaceType,
    {
        /// Creates a shared reborrow of this DynTrait.
        ///
        /// The reborrowed DynTrait cannot use these methods:
        ///
        /// - DynTrait::default
        ///
        /// This is only callable if `DynTrait` is either `Send + Sync` or `!Send + !Sync`.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{
        ///     erased_types::interfaces::DebugDisplayInterface,
        ///     std_types::RBox,
        ///     type_level::{impl_enum::Implemented, trait_marker},
        ///     DynTrait, InterfaceType, RRef,
        /// };
        ///
        /// let to: DynTrait<'static, RBox<()>, DebugDisplayInterface> =
        ///     DynTrait::from_value(1337_u16);
        ///
        /// assert_eq!(debug_string(to.reborrow()), "1337");
        ///
        /// fn debug_string<I>(to: DynTrait<'_, RRef<'_, ()>, I>) -> String
        /// where
        ///     I: InterfaceType<Debug = Implemented<trait_marker::Debug>>,
        /// {
        ///     format!("{:?}", to)
        /// }
        ///
        /// ```
        pub fn reborrow<'re>(&'re self) -> DynTrait<'borr, RRef<'re, ()>, I, EV>
        where
            P: AsPtr<PtrTarget = ()>,
            PrivStruct: ReborrowBounds<I::Send, I::Sync>,
            EV: Copy,
        {
            // Reborrowing will break if I add extra functions that operate on `P`.
            DynTrait {
                object: ManuallyDrop::new(self.object.as_rref()),
                vtable: unsafe { VTable_Ref(self.vtable.0.cast()) },
                extra_value: *self.sabi_extra_value(),
                _marker: NonOwningPhantom::NEW,
                _marker2: UnsafeIgnoredType::DEFAULT,
            }
        }

        /// Creates a mutable reborrow of this DynTrait.
        ///
        /// The reborrowed DynTrait cannot use these methods:
        ///
        /// - DynTrait::default
        ///
        /// - DynTrait::clone
        ///
        /// This is only callable if `DynTrait` is either `Send + Sync` or `!Send + !Sync`.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{
        ///     erased_types::interfaces::DEIteratorInterface, std_types::RBox, DynTrait,
        /// };
        ///
        /// let mut to = DynTrait::from_value(0_u8..=255).interface(DEIteratorInterface::NEW);
        ///
        /// assert_eq!(both_ends(to.reborrow_mut()), (Some(0), Some(255)));
        /// assert_eq!(both_ends(to.reborrow_mut()), (Some(1), Some(254)));
        /// assert_eq!(both_ends(to.reborrow_mut()), (Some(2), Some(253)));
        /// assert_eq!(both_ends(to.reborrow_mut()), (Some(3), Some(252)));
        ///
        /// fn both_ends<I>(mut to: I) -> (Option<I::Item>, Option<I::Item>)
        /// where
        ///     I: DoubleEndedIterator,
        /// {
        ///     (to.next(), to.next_back())
        /// }
        ///
        /// ```
        pub fn reborrow_mut<'re>(&'re mut self) -> DynTrait<'borr, RMut<'re, ()>, I, EV>
        where
            P: AsMutPtr<PtrTarget = ()>,
            PrivStruct: ReborrowBounds<I::Send, I::Sync>,
            EV: Copy,
        {
            let extra_value = *self.sabi_extra_value();
            // Reborrowing will break if I add extra functions that operate on `P`.
            DynTrait {
                object: ManuallyDrop::new(self.object.as_rmut()),
                vtable: unsafe { VTable_Ref(self.vtable.0.cast()) },
                extra_value,
                _marker: NonOwningPhantom::NEW,
                _marker2: UnsafeIgnoredType::DEFAULT,
            }
        }
    }

    impl<'borr, P, I, EV> DynTrait<'borr, P, I, EV>
    where
        P: AsPtr,
    {
        /// Constructs a DynTrait<P, I> with a `P`, using the same vtable.
        ///
        /// `P` must come from a function in the vtable,
        /// or come from a copy of `P: Copy+GetPointerKind<Kind = PK_Reference>`,
        /// to ensure that it is compatible with the functions in it.
        #[allow(clippy::wrong_self_convention)]
        pub(super) const fn from_new_ptr(&self, object: P, extra_value: EV) -> Self {
            Self {
                object: ManuallyDrop::new(object),
                vtable: self.vtable,
                extra_value,
                _marker: NonOwningPhantom::NEW,
                _marker2: UnsafeIgnoredType::DEFAULT,
            }
        }
    }

    impl<'borr, P, I, EV> DynTrait<'borr, P, I, EV>
    where
        I: InterfaceType + 'borr,
        EV: 'borr,
        P: AsPtr,
    {
        /// Constructs a `DynTrait<P, I>` with the default value for `P`.
        ///
        /// # Reborrowing
        ///
        /// This cannot be called with a reborrowed DynTrait:
        ///
        /// ```compile_fail
        /// # use abi_stable::{
        /// #     DynTrait,
        /// #     erased_types::interfaces::DefaultInterface,
        /// # };
        /// let object = DynTrait::from_value(()).interface(DefaultInterface);
        /// let borrow = object.reborrow();
        /// let _ = borrow.default();
        ///
        /// ```
        ///
        /// ```compile_fail
        /// # use abi_stable::{
        /// #     DynTrait,
        /// #     erased_types::interfaces::DefaultInterface,
        /// # };
        /// let object = DynTrait::from_value(()).interface(DefaultInterface);
        /// let borrow = object.reborrow_mut();
        /// let _ = borrow.default();
        ///
        /// ```
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::{erased_types::interfaces::DebugDefEqInterface, DynTrait};
        ///
        /// {
        ///     let object = DynTrait::from_value(true).interface(DebugDefEqInterface);
        ///
        ///     assert_eq!(
        ///         object.default(),
        ///         DynTrait::from_value(false).interface(DebugDefEqInterface)
        ///     );
        /// }
        /// {
        ///     let object = DynTrait::from_value(123u8).interface(DebugDefEqInterface);
        ///
        ///     assert_eq!(
        ///         object.default(),
        ///         DynTrait::from_value(0u8).interface(DebugDefEqInterface)
        ///     );
        /// }
        ///
        /// ```
        pub fn default(&self) -> Self
        where
            P: AsPtr + GetPointerKind<Kind = PK_SmartPointer>,
            I: InterfaceType<Default = Implemented<trait_marker::Default>>,
            EV: Copy,
        {
            unsafe {
                let new = self.sabi_vtable().default_ptr()();
                self.from_new_ptr(new, *self.sabi_extra_value())
            }
        }

        /// It serializes a `DynTrait<_>` into a string by using
        /// `<ConcreteType as SerializeType>::serialize_impl`.
        // I'm using the lifetime in the where clause, clippy <_<
        #[allow(clippy::needless_lifetimes)]
        pub fn serialize_into_proxy<'a>(&'a self) -> Result<I::ProxyType, RBoxError>
        where
            P: AsPtr,
            I: InterfaceType<Serialize = Implemented<trait_marker::Serialize>>,
            I: GetSerializeProxyType<'a>,
        {
            unsafe { self.sabi_vtable().serialize()(self.sabi_erased_ref()).into_result() }
        }
        /// Deserializes a `DynTrait<'borr, _>` from a proxy type, by using
        /// `<I as DeserializeDyn<'borr, Self>>::deserialize_dyn`.
        pub fn deserialize_from_proxy<'de>(proxy: I::Proxy) -> Result<Self, RBoxError>
        where
            P: 'borr + AsPtr,
            I: DeserializeDyn<'de, Self>,
        {
            I::deserialize_dyn(proxy)
        }
    }

    impl<'lt, I, EV: Clone> DynTrait<'lt, crate::std_types::RArc<()>, I, EV> {
        /// Does a shallow clone of the object, just incrementing the reference counter
        pub fn shallow_clone(&self) -> Self {
            Self {
                object: self.object.clone(),
                vtable: self.vtable,
                extra_value: self.extra_value.clone(),
                _marker: self._marker,
                _marker2: self._marker2,
            }
        }
    }

    impl<'borr, P, I, EV> Drop for DynTrait<'borr, P, I, EV>
    where
        P: GetPointerKind,
    {
        fn drop(&mut self) {
            unsafe {
                let vtable = self.sabi_vtable();

                if <P as GetPointerKind>::KIND == PointerKind::SmartPointer {
                    vtable.drop_ptr()(RMut::<P>::new(&mut self.object));
                }
            }
        }
    }
}

pub use self::priv_::DynTrait;

//////////////////////

mod clone_impl {
    pub trait CloneImpl<PtrKind> {
        fn clone_impl(&self) -> Self;
    }
}
use self::clone_impl::CloneImpl;

/// This impl is for smart pointers.
impl<'borr, P, I, EV> CloneImpl<PK_SmartPointer> for DynTrait<'borr, P, I, EV>
where
    P: AsPtr,
    I: InterfaceType<Clone = Implemented<trait_marker::Clone>> + 'borr,
    EV: Copy + 'borr,
{
    fn clone_impl(&self) -> Self {
        unsafe {
            let vtable = self.sabi_vtable();
            let new = vtable.clone_ptr()(RRef::<P>::new(&*self.object));
            self.from_new_ptr(new, *self.sabi_extra_value())
        }
    }
}

/// This impl is for references.
impl<'borr, P, I, EV> CloneImpl<PK_Reference> for DynTrait<'borr, P, I, EV>
where
    P: AsPtr + Copy,
    I: InterfaceType<Clone = Implemented<trait_marker::Clone>> + 'borr,
    EV: Copy + 'borr,
{
    fn clone_impl(&self) -> Self {
        self.from_new_ptr(*self.object, *self.sabi_extra_value())
    }
}

/// Clone is implemented for references and smart pointers,
/// using `GetPointerKind` to decide whether `P` is a smart pointer or a reference.
///
/// DynTrait does not implement Clone if P ==`RMut<'_, ()>` :
///
/// ```compile_fail
/// # use abi_stable::{
/// #     DynTrait,
/// #     erased_types::interfaces::CloneInterface,
/// # };
///
/// let mut object = DynTrait::from_value(()).interface(());
/// let borrow = object.reborrow_mut();
/// let _ = borrow.clone();
///
/// ```
///
impl<'borr, P, I, EV> Clone for DynTrait<'borr, P, I, EV>
where
    P: AsPtr,
    I: InterfaceType,
    Self: CloneImpl<<P as GetPointerKind>::Kind>,
{
    fn clone(&self) -> Self {
        self.clone_impl()
    }
}

//////////////////////

impl<'borr, P, I, EV> Display for DynTrait<'borr, P, I, EV>
where
    P: AsPtr,
    I: InterfaceType<Display = Implemented<trait_marker::Display>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            adapt_std_fmt::<ErasedObject>(self.sabi_erased_ref(), self.sabi_vtable().display(), f)
        }
    }
}

impl<'borr, P, I, EV> Debug for DynTrait<'borr, P, I, EV>
where
    P: AsPtr,
    I: InterfaceType<Debug = Implemented<trait_marker::Debug>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            adapt_std_fmt::<ErasedObject>(self.sabi_erased_ref(), self.sabi_vtable().debug(), f)
        }
    }
}

impl<'borr, P, I, EV> std::error::Error for DynTrait<'borr, P, I, EV>
where
    P: AsPtr,
    I: InterfaceType<
        Display = Implemented<trait_marker::Display>,
        Debug = Implemented<trait_marker::Debug>,
        Error = Implemented<trait_marker::Error>,
    >,
{
}

/// For an example of how to serialize DynTrait,
/// [look here](crate::erased_types::SerializeType#example)
///
impl<'borr, P, I, EV> Serialize for DynTrait<'borr, P, I, EV>
where
    P: AsPtr,
    I: InterfaceType<Serialize = Implemented<trait_marker::Serialize>>,
    I: GetSerializeProxyType<'borr>,
    I::ProxyType: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unsafe {
            self.sabi_vtable().serialize()(self.sabi_erased_ref_unbounded_lifetime())
                .into_result()
                .map_err(ser::Error::custom)?
                .serialize(serializer)
        }
    }
}

/// For an example of how to deserialize DynTrait,
/// [look here](crate::erased_types::DeserializeDyn#example)
///
impl<'de, 'borr: 'de, P, I, EV> Deserialize<'de> for DynTrait<'borr, P, I, EV>
where
    EV: 'borr,
    P: AsPtr + 'borr,
    I: InterfaceType + 'borr,
    I: DeserializeDyn<'de, Self>,
    <I as DeserializeDyn<'de, Self>>::Proxy: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <<I as DeserializeDyn<'de, Self>>::Proxy>::deserialize(deserializer)?;
        I::deserialize_dyn(s).map_err(de::Error::custom)
    }
}

impl<P, I, EV> Eq for DynTrait<'static, P, I, EV>
where
    Self: PartialEq,
    P: AsPtr,
    I: InterfaceType<Eq = Implemented<trait_marker::Eq>>,
{
}

impl<P, P2, I, EV, EV2> PartialEq<DynTrait<'static, P2, I, EV2>> for DynTrait<'static, P, I, EV>
where
    P: AsPtr,
    P2: AsPtr,
    I: InterfaceType<PartialEq = Implemented<trait_marker::PartialEq>>,
{
    fn eq(&self, other: &DynTrait<'static, P2, I, EV2>) -> bool {
        // unsafe: must check that the vtable is the same, otherwise return a sensible value.
        if !self.sabi_is_same_type(other) {
            return false;
        }

        unsafe { self.sabi_vtable().partial_eq()(self.sabi_erased_ref(), other.sabi_erased_ref()) }
    }
}

impl<P, I, EV> Ord for DynTrait<'static, P, I, EV>
where
    P: AsPtr,
    I: InterfaceType<Ord = Implemented<trait_marker::Ord>>,
    Self: PartialOrd + Eq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        // unsafe: must check that the vtable is the same, otherwise return a sensible value.
        if !self.sabi_is_same_type(other) {
            return self.sabi_vtable_address().cmp(&other.sabi_vtable_address());
        }

        unsafe { self.sabi_vtable().cmp()(self.sabi_erased_ref(), other.sabi_erased_ref()).into() }
    }
}

impl<P, P2, I, EV, EV2> PartialOrd<DynTrait<'static, P2, I, EV2>> for DynTrait<'static, P, I, EV>
where
    P: AsPtr,
    P2: AsPtr,
    I: InterfaceType<PartialOrd = Implemented<trait_marker::PartialOrd>>,
    Self: PartialEq<DynTrait<'static, P2, I, EV2>>,
{
    fn partial_cmp(&self, other: &DynTrait<'static, P2, I, EV2>) -> Option<Ordering> {
        // unsafe: must check that the vtable is the same, otherwise return a sensible value.
        if !self.sabi_is_same_type(other) {
            return Some(self.sabi_vtable_address().cmp(&other.sabi_vtable_address()));
        }

        unsafe {
            self.sabi_vtable().partial_cmp()(self.sabi_erased_ref(), other.sabi_erased_ref())
                .map(IntoReprRust::into_rust)
                .into()
        }
    }
}

impl<'borr, P, I, EV> Hash for DynTrait<'borr, P, I, EV>
where
    P: AsPtr,
    I: InterfaceType<Hash = Implemented<trait_marker::Hash>>,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        unsafe { self.sabi_vtable().hash()(self.sabi_erased_ref(), HasherObject::new(state)) }
    }
}

//////////////////////////////////////////////////////////////////

impl<'borr, P, I, Item, EV> Iterator for DynTrait<'borr, P, I, EV>
where
    P: AsMutPtr,
    I: IteratorItemOrDefault<'borr, Item = Item>,
    I: InterfaceType<Iterator = Implemented<trait_marker::Iterator>>,
    Item: 'borr,
{
    type Item = Item;

    fn next(&mut self) -> Option<Item> {
        unsafe {
            let vtable = self.sabi_vtable();
            (vtable.iter().next)(self.sabi_erased_mut()).into_rust()
        }
    }

    fn nth(&mut self, nth: usize) -> Option<Item> {
        unsafe {
            let vtable = self.sabi_vtable();
            (vtable.iter().nth)(self.sabi_erased_mut(), nth).into_rust()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        unsafe {
            let vtable = self.sabi_vtable();
            let tuple = (vtable.iter().size_hint)(self.sabi_erased_ref()).into_rust();
            (tuple.0, tuple.1.into_rust())
        }
    }

    fn count(mut self) -> usize {
        unsafe {
            let vtable = self.sabi_vtable();
            (vtable.iter().count)(self.sabi_erased_mut())
        }
    }

    fn last(mut self) -> Option<Item> {
        unsafe {
            let vtable = self.sabi_vtable();
            (vtable.iter().last)(self.sabi_erased_mut()).into_rust()
        }
    }
}

impl<'borr, P, I, Item, EV> DynTrait<'borr, P, I, EV>
where
    P: AsMutPtr,
    I: IteratorItemOrDefault<'borr, Item = Item>,
    I: InterfaceType<Iterator = Implemented<trait_marker::Iterator>>,
    Item: 'borr,
{
    /// Eagerly skips n elements from the iterator.
    ///
    /// This method is faster than using `Iterator::skip`.
    ///
    /// # Example
    ///
    /// ```
    /// # use abi_stable::{
    /// #     DynTrait,
    /// #     erased_types::interfaces::IteratorInterface,
    /// #     std_types::RVec,
    /// #     traits::IntoReprC,
    /// # };
    ///
    /// let mut iter = 0..20;
    /// let mut wrapped = DynTrait::from_ptr(&mut iter).interface(IteratorInterface::NEW);
    ///
    /// assert_eq!(wrapped.next(), Some(0));
    ///
    /// wrapped.skip_eager(2);
    ///
    /// assert_eq!(wrapped.next(), Some(3));
    /// assert_eq!(wrapped.next(), Some(4));
    /// assert_eq!(wrapped.next(), Some(5));
    ///
    /// wrapped.skip_eager(2);
    ///
    /// assert_eq!(wrapped.next(), Some(8));
    /// assert_eq!(wrapped.next(), Some(9));
    ///
    /// wrapped.skip_eager(9);
    ///
    /// assert_eq!(wrapped.next(), Some(19));
    /// assert_eq!(wrapped.next(), None);
    ///
    ///
    /// ```
    ///
    pub fn skip_eager(&mut self, n: usize) {
        unsafe {
            let vtable = self.sabi_vtable();
            (vtable.iter().skip_eager)(self.sabi_erased_mut(), n);
        }
    }

    /// Extends the `RVec<Item>` with the `self` Iterator.
    ///
    /// Extends `buffer` with as many elements of the iterator as `taking` specifies:
    ///
    /// - RNone: Yields all elements.Use this with care, since Iterators can be infinite.
    ///
    /// - RSome(n): Yields n elements.
    ///
    /// ###  Example
    ///
    /// ```
    /// # use abi_stable::{
    /// #     DynTrait,
    /// #     erased_types::interfaces::IteratorInterface,
    /// #     std_types::{RVec, RSome},
    /// #     traits::IntoReprC,
    /// # };
    ///
    /// let mut wrapped = DynTrait::from_value(0..).interface(IteratorInterface::NEW);
    ///
    /// let mut buffer = vec![101, 102, 103].into_c();
    /// wrapped.extending_rvec(&mut buffer, RSome(5));
    /// assert_eq!(&buffer[..], &*vec![101, 102, 103, 0, 1, 2, 3, 4]);
    ///
    /// assert_eq!(wrapped.next(), Some(5));
    /// assert_eq!(wrapped.next(), Some(6));
    /// assert_eq!(wrapped.next(), Some(7));
    ///
    /// ```
    pub fn extending_rvec(&mut self, buffer: &mut RVec<Item>, taking: ROption<usize>) {
        unsafe {
            let vtable = self.sabi_vtable();
            (vtable.iter().extending_rvec)(self.sabi_erased_mut(), buffer, taking);
        }
    }
}

//////////////////////////////////////////////////////////////////

impl<'borr, P, I, Item, EV> DoubleEndedIterator for DynTrait<'borr, P, I, EV>
where
    Self: Iterator<Item = Item>,
    P: AsMutPtr,
    I: IteratorItemOrDefault<'borr, Item = Item>,
    I: InterfaceType<DoubleEndedIterator = Implemented<trait_marker::DoubleEndedIterator>>,
    Item: 'borr,
{
    fn next_back(&mut self) -> Option<Item> {
        unsafe {
            let vtable = self.sabi_vtable();
            (vtable.back_iter().next_back)(self.sabi_erased_mut()).into_rust()
        }
    }
}

impl<'borr, P, I, Item, EV> DynTrait<'borr, P, I, EV>
where
    Self: Iterator<Item = Item>,
    P: AsMutPtr,
    I: IteratorItemOrDefault<'borr, Item = Item>,
    I: InterfaceType<DoubleEndedIterator = Implemented<trait_marker::DoubleEndedIterator>>,
    Item: 'borr,
{
    /// Gets teh `nth` element from the back of the iterator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{erased_types::interfaces::DEIteratorCloneInterface, DynTrait};
    ///
    /// let to = || DynTrait::from_value(7..=10).interface(DEIteratorCloneInterface::NEW);
    ///
    /// assert_eq!(to().nth_back_(0), Some(10));
    /// assert_eq!(to().nth_back_(1), Some(9));
    /// assert_eq!(to().nth_back_(2), Some(8));
    /// assert_eq!(to().nth_back_(3), Some(7));
    /// assert_eq!(to().nth_back_(4), None);
    /// assert_eq!(to().nth_back_(5), None);
    ///
    /// ```
    ///
    pub fn nth_back_(&mut self, nth: usize) -> Option<Item> {
        unsafe {
            let vtable = self.sabi_vtable();
            (vtable.back_iter().nth_back)(self.sabi_erased_mut(), nth).into_rust()
        }
    }

    /// Extends the `RVec<Item>` with the back of the `self` DoubleEndedIterator.
    ///
    /// Extends `buffer` with as many elements of the iterator as `taking` specifies:
    ///
    /// - RNone: Yields all elements.Use this with care, since Iterators can be infinite.
    ///
    /// - RSome(n): Yields n elements.
    ///
    /// ###  Example
    ///
    /// ```
    /// # use abi_stable::{
    /// #     DynTrait,
    /// #     erased_types::interfaces::DEIteratorInterface,
    /// #     std_types::{RVec, RNone},
    /// #     traits::IntoReprC,
    /// # };
    ///
    /// let mut wrapped = DynTrait::from_value(0..=3).interface(DEIteratorInterface::NEW);
    ///
    /// let mut buffer = vec![101, 102, 103].into_c();
    /// wrapped.extending_rvec_back(&mut buffer, RNone);
    /// assert_eq!(&buffer[..], &*vec![101, 102, 103, 3, 2, 1, 0])
    ///
    /// ```
    ///
    pub fn extending_rvec_back(&mut self, buffer: &mut RVec<Item>, taking: ROption<usize>) {
        unsafe {
            let vtable = self.sabi_vtable();
            (vtable.back_iter().extending_rvec_back)(self.sabi_erased_mut(), buffer, taking);
        }
    }
}

//////////////////////////////////////////////////////////////////

impl<'borr, P, I, EV> fmtWrite for DynTrait<'borr, P, I, EV>
where
    P: AsMutPtr,
    I: InterfaceType<FmtWrite = Implemented<trait_marker::FmtWrite>>,
{
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        let vtable = self.sabi_vtable();

        unsafe {
            match vtable.fmt_write_str()(self.sabi_erased_mut(), s.into()) {
                ROk(_) => Ok(()),
                RErr(_) => Err(fmt::Error),
            }
        }
    }
}

//////////////////////////////////////////////////////////////////

#[inline]
fn to_io_result<T, U>(res: RResult<T, RIoError>) -> io::Result<U>
where
    T: Into<U>,
{
    match res {
        ROk(v) => Ok(v.into()),
        RErr(e) => Err(e.into()),
    }
}

/////////////

impl<'borr, P, I, EV> io::Write for DynTrait<'borr, P, I, EV>
where
    P: AsMutPtr,
    I: InterfaceType<IoWrite = Implemented<trait_marker::IoWrite>>,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let vtable = self.sabi_vtable().io_write();

        unsafe { to_io_result((vtable.write)(self.sabi_erased_mut(), buf.into())) }
    }
    fn flush(&mut self) -> io::Result<()> {
        let vtable = self.sabi_vtable().io_write();

        unsafe { to_io_result((vtable.flush)(self.sabi_erased_mut())) }
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        let vtable = self.sabi_vtable().io_write();

        unsafe { to_io_result((vtable.write_all)(self.sabi_erased_mut(), buf.into())) }
    }
}

/////////////

impl<'borr, P, I, EV> io::Read for DynTrait<'borr, P, I, EV>
where
    P: AsMutPtr,
    I: InterfaceType<IoRead = Implemented<trait_marker::IoRead>>,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let vtable = self.sabi_vtable().io_read();

            to_io_result((vtable.read)(self.sabi_erased_mut(), buf.into()))
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        unsafe {
            let vtable = self.sabi_vtable().io_read();

            to_io_result((vtable.read_exact)(self.sabi_erased_mut(), buf.into()))
        }
    }
}

/////////////

impl<'borr, P, I, EV> io::BufRead for DynTrait<'borr, P, I, EV>
where
    P: AsMutPtr,
    I: InterfaceType<
        IoRead = Implemented<trait_marker::IoRead>,
        IoBufRead = Implemented<trait_marker::IoBufRead>,
    >,
{
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        unsafe {
            let vtable = self.sabi_vtable().io_bufread();

            to_io_result((vtable.fill_buf)(self.sabi_erased_mut()))
        }
    }

    fn consume(&mut self, amount: usize) {
        unsafe {
            let vtable = self.sabi_vtable().io_bufread();

            (vtable.consume)(self.sabi_erased_mut(), amount)
        }
    }
}

/////////////

impl<'borr, P, I, EV> io::Seek for DynTrait<'borr, P, I, EV>
where
    P: AsMutPtr,
    I: InterfaceType<IoSeek = Implemented<trait_marker::IoSeek>>,
{
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        unsafe {
            let vtable = self.sabi_vtable();

            to_io_result(vtable.io_seek()(self.sabi_erased_mut(), pos.into()))
        }
    }
}

//////////////////////////////////////////////////////////////////

unsafe impl<'borr, P, I, EV> Send for DynTrait<'borr, P, I, EV>
where
    P: Send + GetPointerKind,
    I: InterfaceType<Send = Implemented<trait_marker::Send>>,
{
}

unsafe impl<'borr, P, I, EV> Sync for DynTrait<'borr, P, I, EV>
where
    P: Sync + GetPointerKind,
    I: InterfaceType<Sync = Implemented<trait_marker::Sync>>,
{
}

impl<'borr, P, I, EV> Unpin for DynTrait<'borr, P, I, EV>
where
    // `Unpin` is a property of the referent
    P: GetPointerKind,
    I: InterfaceType<Unpin = Implemented<trait_marker::Unpin>>,
{
}

//////////////////////////////////////////////////////////////////

/// Error for `DynTrait<_>` being downcasted into the wrong type
/// with one of the `*downcasted*` methods.
#[derive(Copy, Clone)]
pub struct UneraseError<T> {
    dyn_trait: T,
    expected_type_info: &'static TypeInfo,
    found_type_info: &'static TypeInfo,
}

#[allow(clippy::missing_const_for_fn)]
impl<T> UneraseError<T> {
    fn map<F, U>(self, f: F) -> UneraseError<U>
    where
        F: FnOnce(T) -> U,
    {
        UneraseError {
            dyn_trait: f(self.dyn_trait),
            expected_type_info: self.expected_type_info,
            found_type_info: self.found_type_info,
        }
    }

    /// Extracts the DynTrait, to handle the failure to downcast it.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.dyn_trait
    }
}

impl<D> fmt::Debug for UneraseError<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UneraseError")
            .field("dyn_trait", &"<not shown>")
            .field("expected_type_info", &self.expected_type_info)
            .field("found_type_info", &self.found_type_info)
            .finish()
    }
}

impl<D> fmt::Display for UneraseError<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl<D> ::std::error::Error for UneraseError<D> {}

//////////////////////////////////////////////////////////////////
