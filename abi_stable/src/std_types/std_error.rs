#![allow(clippy::missing_const_for_fn)]

use std::{
    error::Error as ErrorTrait,
    fmt::{self, Debug, Display},
    marker::PhantomData,
    mem,
};

#[allow(unused_imports)]
use core_extensions::SelfOps;

use crate::{
    erased_types::{
        c_functions::{adapt_std_fmt, debug_impl, display_impl},
        FormattingMode,
    },
    marker_type::{ErasedObject, SyncSend, UnsyncSend, UnsyncUnsend},
    pointer_trait::{AsMutPtr, AsPtr},
    prefix_type::WithMetadata,
    sabi_types::RRef,
    std_types::{
        utypeid::{new_utypeid, UTypeId},
        RBox, ROption, RResult, RStr, RString,
    },
    utils::transmute_reference,
};

#[cfg(test)]
// #[cfg(all(test, not(feature = "only_new_tests")))]
mod test;

/// Ffi-safe version of `Box<dyn std::error::Error + 'static>`
/// whose `Send + Sync`ness is determined by the `M` type parameter.
///
/// # Examples
///
/// ### Converting from and into `Box<dyn Error + ...>`
/// <span id = "from_to_conversion"></span>
/// ```
/// use std::{convert::TryFrom, error::Error as ErrorTrait};
///
/// use abi_stable::std_types::{RBox, RBoxError, SendRBoxError, UnsyncRBoxError};
///
/// {
///     let from: Box<dyn ErrorTrait> = "hello, error".into();
///     let rbox = UnsyncRBoxError::from_box(from);
///     let _: Box<dyn ErrorTrait> = rbox.into_box();
/// }
///
/// {
///     let arr_err = <[(); 0]>::try_from(&[()][..]).unwrap_err();
///     let from: Box<dyn ErrorTrait + Send> = Box::new(arr_err);
///     let rbox = SendRBoxError::from_box(from);
///     let _: Box<dyn ErrorTrait + Send> = rbox.into_box();
/// }
///
/// {
///     let arr_err = <[(); 0]>::try_from(&[()][..]).unwrap_err();
///     let from: Box<dyn ErrorTrait + Send + Sync> = Box::new(arr_err);
///     let rbox = RBoxError::from_box(from);
///     let _: Box<dyn ErrorTrait + Send + Sync> = rbox.into_box();
/// }
///
///
/// ```
///
/// ### Downcasting by value
///
/// ```
/// use std::num::{ParseFloatError, ParseIntError};
///
/// use abi_stable::std_types::{RBox, RBoxError};
///
/// // Constructing a `RBoxError` from a `Box<dyn Error>`, then downcasting to a `ParseIntError`.
/// {
///     let parse_err = "".parse::<u64>().unwrap_err();
///     let dyn_err: Box<dyn std::error::Error + Send + Sync + 'static> =
///         Box::new(parse_err.clone());
///     let err = RBoxError::from_box(dyn_err);
///
///     assert_eq!(
///         err.downcast::<ParseIntError>().unwrap(),
///         RBox::new(parse_err)
///     );
/// }
///
/// // Constructing a `RBoxError` from a `ParseFloatError`, then downcasting back to it.
/// {
///     let parse_err = "".parse::<f32>().unwrap_err();
///     let err = RBoxError::new(parse_err.clone());
///
///     assert_eq!(
///         err.downcast::<ParseFloatError>().unwrap(),
///         RBox::new(parse_err)
///     );
/// }
///
/// // Constructing a `RBoxError` from a `ParseFloatError`,
/// // then attempting to downcasting it to a `ParseIntError`.
/// {
///     let parse_err = "".parse::<f32>().unwrap_err();
///     let err = RBoxError::new(parse_err);
///
///     assert!(err.downcast::<ParseIntError>().is_err());
/// }
///
/// ```
///
/// ### Downcasting by reference
///
/// ```
/// use std::{convert::TryFrom, num::TryFromIntError, str::Utf8Error};
///
/// use abi_stable::std_types::{RBox, UnsyncRBoxError};
///
/// // Constructing a `UnsyncRBoxError` from a `Box<dyn Error>`,
/// // then downcasting to a `TryFromIntError`.
/// {
///     let int_err = u32::try_from(-1_i32).unwrap_err();
///     let dyn_err: Box<dyn std::error::Error + 'static> = Box::new(int_err.clone());
///     let err = UnsyncRBoxError::from_box(dyn_err);
///
///     assert_eq!(err.downcast_ref::<TryFromIntError>().unwrap(), &int_err);
/// }
///
/// // Constructing a `UnsyncRBoxError` from a `Utf8Error`, then downcasting back to it.
/// {
///     let utf8_err = std::str::from_utf8(&[255]).unwrap_err();
///     let err = UnsyncRBoxError::new(utf8_err.clone());
///
///     assert_eq!(err.downcast_ref::<Utf8Error>().unwrap(), &utf8_err);
/// }
///
/// // Constructing a `UnsyncRBoxError` from a `Utf8Error`,
/// // then attempting to downcasting it to a `TryFromIntError`.
/// {
///     let utf8_err = std::str::from_utf8(&[255]).unwrap_err();
///     let err = UnsyncRBoxError::new(utf8_err);
///
///     assert!(err.downcast_ref::<TryFromIntError>().is_none());
/// }
///
/// ```
///
///
/// ### Downcasting by mutable reference
///
/// ```
/// use std::string::{FromUtf16Error, FromUtf8Error};
///
/// use abi_stable::std_types::{RBox, SendRBoxError};
///
/// // Constructing a `SendRBoxError` from a `Box<dyn Error>`,
/// // then downcasting to a `FromUtf8Error`.
/// {
///     let str_err = || String::from_utf8(vec![255]).unwrap_err();
///     let dyn_err: Box<dyn std::error::Error + Send + 'static> = Box::new(str_err());
///     let mut err = SendRBoxError::from_box(dyn_err);
///
///     assert!(err.downcast_ref::<FromUtf8Error>().is_some(), "part A");
/// }
///
/// // Constructing a `SendRBoxError` from a `FromUtf8Error`, then downcasting back to it.
/// {
///     let str_err = || String::from_utf8(vec![255]).unwrap_err();
///     let mut err = SendRBoxError::new(str_err());
///
///     assert!(err.downcast_ref::<FromUtf8Error>().is_some(), "part B");
/// }
///
/// // Constructing a `SendRBoxError` from a `FromUtf16Error`,
/// // then attempting to downcasting it to a `FromUtf8Error`.
/// {
///     let str_err = || String::from_utf16(&[0xD834]).unwrap_err();
///     let mut err = SendRBoxError::new(str_err());
///
///     assert!(err.downcast_ref::<FromUtf8Error>().is_none(), "part C");
/// }
///
/// ```
///
///
///
#[repr(C)]
#[derive(StableAbi)]
pub struct RBoxError_<M = SyncSend> {
    value: RBox<ErasedObject>,
    vtable: RErrorVTable_Ref,
    _sync_send: PhantomData<M>,
}

/// Ffi safe equivalent to `Box<dyn std::error::Error>`.
pub type UnsyncRBoxError = RBoxError_<UnsyncUnsend>;

/// Ffi safe equivalent to `Box<dyn std::error::Error + Send>`.
pub type SendRBoxError = RBoxError_<UnsyncSend>;

/// Ffi safe equivalent to `Box<dyn std::error::Error + Send + Sync>`.
pub type RBoxError = RBoxError_<SyncSend>;

impl RBoxError_<SyncSend> {
    /// Constructs a `Send + Sync` `RBoxError_` from an error.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RBoxError;
    ///
    /// let str_err = String::from_utf8(vec![255]).unwrap_err();
    ///
    /// let err = RBoxError::new(str_err);
    /// ```
    pub fn new<T>(value: T) -> Self
    where
        T: ErrorTrait + Send + Sync + 'static,
    {
        Self::new_inner(value)
    }
}

impl RBoxError_<UnsyncSend> {
    /// Constructs a `Send + !Sync` `RBoxError_` from an error.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::SendRBoxError;
    ///
    /// let str_err = String::from_utf16(&[0xD834]).unwrap_err() ;
    ///
    /// let err = SendRBoxError::new(str_err);
    /// ```
    pub fn new<T>(value: T) -> Self
    where
        T: ErrorTrait + Send + 'static,
    {
        Self::new_inner(value)
    }
}

impl RBoxError_<UnsyncUnsend> {
    /// Constructs a `!Send + !Sync` `RBoxError_` from an error.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::UnsyncRBoxError;
    ///
    /// let str_err = std::str::from_utf8(&[255]).unwrap_err() ;
    ///
    /// let err = UnsyncRBoxError::new(str_err);
    /// ```
    pub fn new<T>(value: T) -> Self
    where
        T: ErrorTrait + 'static,
    {
        Self::new_inner(value)
    }
}

impl<M> RBoxError_<M> {
    /// Constructs an RBoxError from an error,
    /// storing the Debug and Display messages without storing the error value.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RBoxError;
    ///
    /// let int_error = "".parse::<u32>().unwrap_err();
    ///
    /// let display_fmt = int_error.to_string();
    /// let debug_fmt = format!("{:#?}", int_error);
    ///
    /// let err = RBoxError::from_fmt(&int_error);
    ///
    /// assert_eq!(display_fmt, err.to_string());
    /// assert_eq!(debug_fmt, format!("{:?}", err));
    /// ```
    pub fn from_fmt<T>(value: &T) -> Self
    where
        T: Display + Debug + ?Sized,
    {
        DebugDisplay {
            debug: format!("{:#?}", value),
            display: format!("{:#}", value),
        }
        .piped(Self::from_debug_display)
    }

    /// Constructs an RBoxError from a type that only implements Debug,
    /// storing the Debug message without storing the error value.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RBoxError;
    ///
    /// let int_error = "".parse::<u32>().unwrap_err();
    ///
    /// let debug_fmt = format!("{:#?}", int_error);
    /// let err = RBoxError::from_debug(&int_error);
    ///
    /// assert_eq!(debug_fmt, format!("{}", err));
    /// assert_eq!(debug_fmt, format!("{:#?}", err));
    /// ```
    pub fn from_debug<T>(value: &T) -> Self
    where
        T: Debug + ?Sized,
    {
        DebugDisplay {
            debug: format!("{:#?}", value),
            display: format!("{:#?}", value),
        }
        .piped(Self::from_debug_display)
    }

    fn from_debug_display(value: DebugDisplay) -> Self {
        unsafe { Self::new_with_vtable(value, MakeRErrorVTable::LIB_VTABLE_DEBUG_DISPLAY) }
    }

    fn new_inner<T>(value: T) -> Self
    where
        T: ErrorTrait + 'static,
    {
        unsafe { Self::new_with_vtable(value, MakeRErrorVTable::<T>::LIB_VTABLE) }
    }

    unsafe fn new_with_vtable<T>(value: T, vtable: RErrorVTable_Ref) -> Self {
        let value = value
            .piped(RBox::new)
            .piped(|x| unsafe { mem::transmute::<RBox<T>, RBox<ErasedObject>>(x) });

        Self {
            value,
            vtable,
            _sync_send: PhantomData,
        }
    }
}

impl<M> RBoxError_<M> {
    /// Converts this error to a formatted error
    ///
    /// This is used to decouple an `RBoxError` from the dynamic library that produced it,
    /// in order to unload the dynamic library.
    ///
    // This isn't strictly required anymore because abi_stable doesn't
    // unload libraries right now.
    ///
    pub fn to_formatted_error<N>(&self) -> RBoxError_<N> {
        if let Some(dd) = self.as_debug_display() {
            RBoxError_::from_debug_display(DebugDisplay {
                debug: dd.debug.into(),
                display: dd.display.into(),
            })
        } else {
            RBoxError_::from_fmt(self)
        }
    }

    fn as_debug_display(&self) -> Option<DebugDisplayRef<'_>> {
        unsafe { self.vtable.as_debug_display()(self.value.as_rref()).into_option() }
    }
}

impl<M> RBoxError_<M> {
    /// Returns the `UTypeId` of the error this wraps.
    pub fn type_id(&self) -> UTypeId {
        self.vtable.type_id()()
    }

    fn is_type<T: 'static>(&self) -> bool {
        let self_id = self.vtable.type_id()();
        let other_id = UTypeId::new::<T>();
        self_id == other_id
    }

    /// The address of the `Box<_>` this wraps
    pub fn heap_address(&self) -> usize {
        (self.value.as_ptr()) as *const _ as usize
    }

    /// Casts this `&RBoxError_<_>` to `&UnsyncRBoxError`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RBoxError, UnsyncRBoxError};
    /// use std::convert::TryFrom;
    ///
    /// let int_err = u32::try_from(-1_i32).unwrap_err();
    ///
    /// let err: RBoxError = RBoxError::new(int_err);
    ///
    /// let unsync_err: &UnsyncRBoxError = err.as_unsync();
    ///
    /// ```
    pub fn as_unsync(&self) -> &UnsyncRBoxError {
        unsafe { transmute_reference::<RBoxError_<M>, UnsyncRBoxError>(self) }
    }

    /// Converts this `RBoxError_<_>` to `UnsyncRBoxError`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RBoxError, UnsyncRBoxError};
    /// use std::convert::TryFrom;
    ///
    /// let int_err = u64::try_from(-1338_i32).unwrap_err();
    ///
    /// let err: RBoxError = RBoxError::new(int_err);
    ///
    /// let unsync_err: UnsyncRBoxError = err.into_unsync();
    ///
    /// ```
    pub fn into_unsync(self) -> UnsyncRBoxError {
        unsafe { mem::transmute::<RBoxError_<M>, UnsyncRBoxError>(self) }
    }
}

impl RBoxError_<SyncSend> {
    /// Casts this `&RBoxError_<_>` to `&SendRBoxError`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RBoxError, SendRBoxError};
    /// use std::convert::TryFrom;
    ///
    /// let slice: &mut [u32] = &mut [];
    /// let arr_err=<&mut [u32;10]>::try_from(slice).unwrap_err();
    ///
    /// let err: RBoxError = RBoxError::new(arr_err);
    ///
    /// let unsync_err: &SendRBoxError = err.as_send();
    ///
    /// ```
    pub fn as_send(&self) -> &SendRBoxError {
        unsafe { transmute_reference::<RBoxError_<SyncSend>, SendRBoxError>(self) }
    }

    /// Converts this `RBoxError_<_>` to `SendRBoxError`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RBoxError, SendRBoxError};
    /// use std::convert::TryFrom;
    ///
    /// let slice: &[u32] = &[];
    /// let arr_err=<&[u32;10]>::try_from(slice).unwrap_err();
    ///
    /// let err: RBoxError = RBoxError::new(arr_err);
    ///
    /// let unsync_err: SendRBoxError = err.into_send();
    ///
    /// ```
    pub fn into_send(self) -> SendRBoxError {
        unsafe { mem::transmute::<RBoxError_<SyncSend>, SendRBoxError>(self) }
    }
}

impl<M> ErrorTrait for RBoxError_<M> {}

impl<M> Display for RBoxError_<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe { adapt_std_fmt(self.value.as_rref(), self.vtable.display(), f) }
    }
}

impl<M> Debug for RBoxError_<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe { adapt_std_fmt(self.value.as_rref(), self.vtable.debug(), f) }
    }
}

////////////////////////////////////////////////////////////////////////

macro_rules! from_impls {
    (
        $first_docs: expr,
        $boxdyn: ty,
        $marker: ty,
    ) => {
        impl From<$boxdyn> for RBoxError_<$marker> {
            #[doc = $first_docs]
            ///
            /// # Behavior
            ///
            /// If the contents of the Box<_> is an erased `RBoxError_<_>`
            /// it will be returned directly,
            /// otherwise the `Box<_>` will be converted into an `RBoxError_<_>`
            /// using `RBoxError_::new`.
            ///
            fn from(this: $boxdyn) -> RBoxError_<$marker> {
                Self::from_box(this)
            }
        }

        impl RBoxError_<$marker> {
            #[doc = $first_docs]
            ///
            /// `RBoxError::from_box( RBoxError::into_box( err ) )`
            /// is a no-op with respect to the heap address of the RBoxError_<_>.
            ///
            /// # Behavior
            ///
            /// If the contents of the `Box<_>` is an erased `RBoxError_<_>`
            /// it will be returned directly,
            /// otherwise the `Box<_>` will be converted into an `RBoxError_<_>`
            /// using `RBoxError_::new`.
            ///
            /// # Example
            ///
            /// For an example of converting back and forth to a `Box<dyn Error>`
            /// [here is the example](#from_to_conversion).
            pub fn from_box(this: $boxdyn) -> Self {
                match this.downcast::<Self>() {
                    Ok(e) => *e,
                    Err(e) => unsafe {
                        Self::new_with_vtable::<$boxdyn>(
                            e,
                            MakeBoxedRErrorVTable::<$boxdyn>::LIB_VTABLE,
                        )
                    },
                }
            }

            /// Converts an `RBoxError_<_>` to a `Box<dyn Error>`.
            ///
            /// `RBoxError::from_box( RBoxError::into_box( err ) )`
            /// is a no-op with respect to the heap address of the RBoxError_<_>.
            ///
            /// # Behavior
            ///
            /// If the contents of the `RBoxError_<_>` is an erased `Box<dyn Error + ... >`
            /// it will be returned directly,
            /// otherwise the `RBoxError_<_>` will be converted into an `Box<dyn Error + ... >`
            /// using `Box::new`.
            ///
            /// # Example
            ///
            /// For an example of converting back and forth to a `Box<dyn Error>`
            /// [here is the example](#from_to_conversion).
            pub fn into_box(self) -> $boxdyn {
                if self.is_type::<$boxdyn>() {
                    unsafe {
                        let box_ = mem::transmute::<RBox<ErasedObject>, RBox<$boxdyn>>(self.value);
                        RBox::into_inner(box_)
                    }
                } else {
                    Box::new(self)
                }
            }

            /// Converts this `RBoxError_<_>` to an `RBox<T>`.
            ///
            /// If was constructed from a `Box<dyn Error + ... >`,
            /// and this is being casted to another type,
            /// it'll first downcast to `Box<dyn Error + ... >`,
            /// then it'll downcast the `Box<dyn Error + ... >` into `RBox<T>`.
            ///
            /// # Errors
            ///
            /// This returns `Err(self)` in any of these cases:
            ///
            /// - The `RBoxError_` wasn't constructed in the current dynamic library.
            ///
            /// - The `RBoxError_` was constructed with a different type than `T`.
            ///
            /// # Example
            ///
            /// Look at the type level documentation for
            /// [the example](#downcasting-by-value).
            ///
            pub fn downcast<T>(self) -> Result<RBox<T>, Self>
            where
                T: ErrorTrait + 'static,
            {
                match (self.is_type::<T>(), self.is_type::<$boxdyn>()) {
                    (true, _) => unsafe {
                        Ok(mem::transmute::<RBox<ErasedObject>, RBox<T>>(self.value))
                    },
                    (false, true) if self.downcast_ref::<T>().is_some() => unsafe {
                        let x = mem::transmute::<RBox<ErasedObject>, RBox<$boxdyn>>(self.value);
                        let x = RBox::into_inner(x);
                        Ok(RBox::from_box(x.downcast::<T>().unwrap()))
                    },
                    (false, _) => Err(self),
                }
            }

            /// Converts this `&RBoxError_<_>` to a `&T`.
            ///
            /// If was constructed from a `Box<dyn Error + ... >`,
            /// and this is being casted to another type,
            /// it'll first downcast to `&dyn Error + ... `,
            /// then it'll downcast the `&dyn Error + ... ` into `&T`.
            ///
            /// # Errors
            ///
            /// This returns `None` in any of these cases:
            ///
            /// - The `RBoxError_` wasn't constructed in the current dynamic library.
            ///
            /// - The `RBoxError_` was constructed with a different type than `T`.
            ///
            /// # Example
            ///
            /// Look at the type level documentation for the example.
            /// [the example](#downcasting-by-reference).
            ///
            pub fn downcast_ref<T>(&self) -> Option<&T>
            where
                T: ErrorTrait + 'static,
            {
                match (self.is_type::<T>(), self.is_type::<$boxdyn>()) {
                    (true, _) => unsafe { Some(&*(self.value.as_ptr() as *const T)) },
                    (false, true) => unsafe {
                        let ref_box = &*(self.value.as_ptr() as *const $boxdyn);
                        (&**ref_box).downcast_ref::<T>()
                    },
                    (false, false) => None,
                }
            }

            /// Converts this `&mut RBoxError_<_>` to a `&mut T`.
            ///
            /// If was constructed from a `Box<dyn Error + ... >`,
            /// and this is being casted to another type,
            /// it'll first downcast to `&mut dyn Error + ... `,
            /// then it'll downcast the `&mut dyn Error + ... ` into `&mut T`.
            ///
            /// # Errors
            ///
            /// This returns `None` in any of these cases:
            ///
            /// - The `RBoxError_` wasn't constructed in the current dynamic library.
            ///
            /// - The `RBoxError_` was constructed with a different type than `T`.
            ///
            /// # Example
            ///
            /// Look at the type level documentation for the example.
            /// [the example](#downcasting-by-mutable-reference).
            ///
            pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
            where
                T: ErrorTrait + 'static,
            {
                match (self.is_type::<T>(), self.is_type::<$boxdyn>()) {
                    (true, _) => unsafe { Some(&mut *(self.value.as_mut_ptr() as *mut T)) },
                    (false, true) => unsafe {
                        let mut_box = &mut *(self.value.as_mut_ptr() as *mut $boxdyn);
                        (&mut **mut_box).downcast_mut::<T>()
                    },
                    (false, false) => None,
                }
            }
        }
    };
}

from_impls! {
    "Converts a `Box<dyn Error + Send + Sync>` to a `Send + Sync` `RBoxError_`.",
    Box<dyn ErrorTrait + Send + Sync + 'static> ,
    SyncSend,
}
from_impls! {
    "Converts a `Box<dyn Error + Send>` to a `Send + !Sync` `RBoxError_`.",
    Box<dyn ErrorTrait + Send + 'static> ,
    UnsyncSend,
}
from_impls! {
    "Converts a `Box<dyn Error>` to a `!Send + !Sync` `RBoxError_`.",
    Box<dyn ErrorTrait + 'static> ,
    UnsyncUnsend,
}

////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
struct RErrorVTable {
    debug: unsafe extern "C" fn(
        RRef<'_, ErasedObject>,
        FormattingMode,
        &mut RString,
    ) -> RResult<(), ()>,

    display: unsafe extern "C" fn(
        RRef<'_, ErasedObject>,
        FormattingMode,
        &mut RString,
    ) -> RResult<(), ()>,

    as_debug_display: unsafe extern "C" fn(RRef<'_, ErasedObject>) -> ROption<DebugDisplayRef<'_>>,

    #[sabi(last_prefix_field)]
    type_id: extern "C" fn() -> UTypeId,
}

///////////////////

struct MakeRErrorVTable<T>(T);

impl<T> MakeRErrorVTable<T>
where
    T: ErrorTrait + 'static,
{
    const VALUE: RErrorVTable = RErrorVTable {
        debug: debug_impl::<T>,
        display: display_impl::<T>,
        as_debug_display: not_as_debug_display,
        type_id: new_utypeid::<T>,
    };

    const VALUE_MD: &'static WithMetadata<RErrorVTable> = &WithMetadata::new(Self::VALUE);

    const LIB_VTABLE: RErrorVTable_Ref = { RErrorVTable_Ref(Self::VALUE_MD.static_as_prefix()) };
}

impl MakeRErrorVTable<DebugDisplay> {
    const WM_DEBUG_DISPLAY: &'static WithMetadata<RErrorVTable> = {
        &WithMetadata::new(RErrorVTable {
            debug: debug_impl::<DebugDisplay>,
            display: display_impl::<DebugDisplay>,
            as_debug_display,
            type_id: new_utypeid::<DebugDisplay>,
        })
    };

    const LIB_VTABLE_DEBUG_DISPLAY: RErrorVTable_Ref =
        { RErrorVTable_Ref(Self::WM_DEBUG_DISPLAY.static_as_prefix()) };
}

///////////////////

struct MakeBoxedRErrorVTable<T>(T);

impl<T> MakeBoxedRErrorVTable<Box<T>>
where
    T: ?Sized + ErrorTrait + 'static,
{
    const VALUE: RErrorVTable = RErrorVTable {
        debug: debug_impl::<Box<T>>,
        display: display_impl::<Box<T>>,
        as_debug_display: not_as_debug_display,
        type_id: new_utypeid::<Box<T>>,
    };

    const WM_VTABLE: &'static WithMetadata<RErrorVTable> = &WithMetadata::new(Self::VALUE);

    const LIB_VTABLE: RErrorVTable_Ref = RErrorVTable_Ref(Self::WM_VTABLE.static_as_prefix());
}

////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Clone)]
struct DebugDisplay {
    debug: String,
    display: String,
}

impl Display for DebugDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.display, f)
    }
}

impl Debug for DebugDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.debug, f)
    }
}

impl ErrorTrait for DebugDisplay {}

////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Debug, StableAbi, PartialEq)]
struct DebugDisplayRef<'a> {
    debug: RStr<'a>,
    display: RStr<'a>,
}

////////////////////////////////////////////////////////////////////////

unsafe extern "C" fn as_debug_display(
    this: RRef<'_, ErasedObject>,
) -> ROption<DebugDisplayRef<'_>> {
    extern_fn_panic_handling! {
        let this = unsafe{ this.transmute_into_ref::<DebugDisplay>() };
        ROption::RSome(DebugDisplayRef{
            debug: this.debug.as_str().into(),
            display: this.display.as_str().into(),
        })
    }
}

unsafe extern "C" fn not_as_debug_display(
    _: RRef<'_, ErasedObject>,
) -> ROption<DebugDisplayRef<'_>> {
    ROption::RNone
}
