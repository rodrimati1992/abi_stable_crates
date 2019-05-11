/*! 
Ffi-safe version of 
`Box<::std::error::Error+Sync+Send+'static>` and `Box<::std::error::Error+'static>`
*/
use std::{
    error::Error as ErrorTrait,
    fmt::{self, Debug, Display},
    marker::PhantomData,
    mem,
};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use crate::{
    erased_types::{
        c_functions::{adapt_std_fmt, debug_impl, display_impl},
        FormattingMode,
    },
    marker_type::{SyncSend, UnsyncUnsend,UnsyncSend,ErasedObject},
    prefix_type::{PrefixTypeTrait,WithMetadata},
    std_types::{
        RBox, RResult, RString,
        utypeid::{UTypeId,new_utypeid}
    },
    utils::{transmute_reference,transmute_mut_reference},
};

#[cfg(all(test,not(feature="only_new_tests")))]
mod test;

/// Ffi-safe version of `Box<::std::error::Error+'static>` 
/// whose `Send+Sync`ness is determined by the `M` type parameter.
///
/// It cannot be converted back to `Box<::std::error::Error>`,
/// requiring wrapping `RBoxError_<_>` itself to be wrapped in a `Box<_>`.
///
/// Unwrapping a `Box<Error+?Send+?Sync+'static>` back to
/// `RBoxError_<_>` does not incurr an allocation.
/// 
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct RBoxError_<M = SyncSend> {
    value: RBox<ErasedObject>,
    vtable: &'static RErrorVTable,
    _sync_send: PhantomData<M>,
}

/// Ffi safe equivalent to Box<::std::error::Error>.
pub type UnsyncRBoxError = RBoxError_<UnsyncUnsend>;

/// Ffi safe equivalent to Box<::std::error::Error+Send>.
pub type SendRBoxError = RBoxError_<UnsyncSend>;

/// Ffi safe equivalent to Box<::std::error::Error+Send+Sync>.
pub type RBoxError = RBoxError_<SyncSend>;


impl<M> RBoxError_<M> {
    /// Constructs an RBoxError from an error,
    /// storing the Debug and Display messages without storing the error value.
    pub fn from_fmt<T>(value: T) -> Self
    where
        T: Display + Debug,
    {
        DebugDisplay {
            debug: format!("{:#?}", value),
            display: format!("{:#}", value),
        }
        .piped(Self::new_inner)
    }

    fn new_inner<T>(value: T) -> Self
    where
        T: ErrorTrait + 'static,
    {
        unsafe {
            Self::new_with_vtable(
                value,
                MakeRErrorVTable::<T>::LIB_VTABLE.as_prefix()
            )
        }
    }

    fn new_with_vtable<T>(value: T,vtable:&'static RErrorVTable) -> Self{
        unsafe {
            let value = value
                .piped(RBox::new)
                .piped(|x| mem::transmute::<RBox<T>, RBox<ErasedObject>>(x));

            Self {
                value,
                vtable,
                _sync_send: PhantomData,
            }
        }
    }
}

impl<M> RBoxError_<M> {
    /// Returns the UTypeId of the error this wraps.
    pub fn type_id(&self)->UTypeId{
        self.vtable.type_id()()
    }

    fn is_type<T:'static>(&self)->bool{
        let self_id=self.vtable.type_id()();
        let other_id=UTypeId::new::<T>();
        self_id==other_id
    }

    /// The address of the `Box<_>` this wraps
    pub fn heap_address(&self)->usize{
        (&*self.value)as *const _ as usize
    }

    /// Converts this `RBoxError_<_>` to an `RBox<T>`.
    ///
    /// # Errors
    ///
    /// This returns `Err(self)` under the same conditions where `DynTrait<_>`
    /// cannot be unerased.
    ///
    pub fn downcast<T:'static>(self)->Result<RBox<T>,Self>{
        if self.is_type::<T>() {
            unsafe{
                Ok(mem::transmute::<RBox<ErasedObject>, RBox<T>>(self.value))
            }
        }else{
            Err(self)
        }
    }

    /// Converts this `&RBoxError_<_>` to an `Option<&T>`.
    ///
    /// # Errors
    ///
    /// This returns `Err(self)` under the same conditions where `DynTrait<_>`
    /// cannot be unerased.
    ///
    pub fn downcast_ref<T:'static>(&self)->Option<&T>{
        if self.is_type::<T>() {
            unsafe{
                Some(transmute_reference::<ErasedObject,T>(&*self.value))
            }
        }else{
            None
        }
    }

    /// Converts this `&mut RBoxError_<_>` to an `Option<&mut T>`.
    ///
    /// # Errors
    ///
    /// This returns `Err(self)` under the same conditions where `DynTrait<_>`
    /// cannot be unerased.
    ///
    pub fn downcast_mut<T:'static>(&mut self)->Option<&mut T>{
        if self.is_type::<T>() {
            unsafe{
                Some(transmute_mut_reference::<ErasedObject,T>(&mut *self.value))
            }
        }else{
            None
        }
    }

    /// Casts this `&RBoxError_<_>` to `&UnsyncRBoxError`.
    pub fn as_unsync(&self)->&UnsyncRBoxError{
        unsafe{
            transmute_reference::<RBoxError_<M>,UnsyncRBoxError>(&self)
        }
    }

    /// Converts this `RBoxError_<_>` to `UnsyncRBoxError`.
    pub fn into_unsync(self)->UnsyncRBoxError{
        unsafe{
            mem::transmute::<RBoxError_<M>,UnsyncRBoxError>(self)
        }
    }
}


impl RBoxError_<SyncSend>{
    /// Casts this `&RBoxError_<_>` to `&SendRBoxError`.
    pub fn as_send(&self)->&SendRBoxError{
        unsafe{
            transmute_reference::<RBoxError_<SyncSend>,SendRBoxError>(&self)
        }
    }

    /// Converts this `RBoxError_<_>` to `SendRBoxError`.
    pub fn into_send(self)->SendRBoxError{
        unsafe{
            mem::transmute::<RBoxError_<SyncSend>,SendRBoxError>(self)
        }
    }
}


macro_rules! declare_constructor {
    ( 
        bounds=( $($bounds:tt)* ) , 
        marker=$marker:ident,
    ) => (
        impl RBoxError_<$marker> {
            /// Constructs an RBoxError from an error.
            pub fn new<T>(value: T) -> Self
            where
                T: $($bounds)*,
            {
                Self::new_inner(value)
            }
        }
    )
}


declare_constructor!{
    bounds=(ErrorTrait + Send + Sync + 'static) , 
    marker=SyncSend,
}
declare_constructor!{
    bounds=(ErrorTrait + Send + 'static) , 
    marker=UnsyncSend,
}
declare_constructor!{
    bounds=(ErrorTrait + 'static) , 
    marker=UnsyncUnsend,
}


impl<M> ErrorTrait for RBoxError_<M> {}

impl<M> Display for RBoxError_<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        adapt_std_fmt(&*self.value, self.vtable.display(), f)
    }
}

impl<M> Debug for RBoxError_<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        adapt_std_fmt(&*self.value, self.vtable.debug(), f)
    }
}

////////////////////////////////////////////////////////////////////////


macro_rules! from_impls {
    ($boxdyn:ty,$marker:ty) => (
        impl From<$boxdyn> for RBoxError_<$marker>{
            /// Converts a Box<dyn Error> to an RBoxError_<_>.
            ///
            /// # Behavior 
            ///
            /// If the contents of the Box<_> is an erased `RBoxError_<_>`
            /// it will be returned directly,
            /// otherwise the `Box<_>` will be converted into an `RBoxError_<_>`
            /// using `RBoxError_::new`.
            fn from(this:$boxdyn)->RBoxError_<$marker>{
                Self::from_box(this)
            }
        }

        impl RBoxError_<$marker>{
            /// Converts a Box<dyn Error> to an RBoxError_<_>.
            ///
            /// `RBoxError::from_box( RBoxError::into_box( err ) )` 
            /// is a no-op with respect to the heap address of the RBoxError_<_>.
            ///
            /// # Behavior 
            ///
            /// If the contents of the Box<_> is an erased `RBoxError_<_>`
            /// it will be returned directly,
            /// otherwise the `Box<_>` will be converted into an `RBoxError_<_>`
            /// using `RBoxError_::new`.
            pub fn from_box(this:$boxdyn)->Self{
                match this.downcast::<Self>() {
                    Ok(e)=>{
                        *e
                    }
                    Err(e)=>{
                        Self::new_with_vtable::<$boxdyn>(
                            e,
                            MakeBoxedRErrorVTable::<$boxdyn>::LIB_VTABLE.as_prefix(),
                        )
                    }
                }
            }

            /// Converts an `RBoxError_<_>` to a `Box<dyn Error>`.
            ///
            /// `RBoxError::from_box( RBoxError::into_box( err ) )` 
            /// is a no-op with respect to the heap address of the RBoxError_<_>.
            ///
            /// # Behavior 
            ///
            /// If the contents of the RBoxError_<_> is an erased `Box<dyn Error + ... >`
            /// it will be returned directly,
            /// otherwise the RBoxError_<_> will be converted into an `Box<dyn Error + ... >`
            /// using `Box::new`.
            pub fn into_box(self)->$boxdyn{
                match self.downcast::<$boxdyn>() {
                    Ok(e)=>e.piped(RBox::into_inner),
                    Err(e)=>Box::new(e),
                }
            }
        }
    )
}


from_impls!{ Box<dyn ErrorTrait + Send + Sync + 'static> , SyncSend }
from_impls!{ Box<dyn ErrorTrait + Send + 'static> , UnsyncSend }
from_impls!{ Box<dyn ErrorTrait + 'static> , UnsyncUnsend }



////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
#[sabi(kind(Prefix(prefix_struct="RErrorVTable")))]
struct RErrorVTableVal {
    debug: extern "C" fn(&ErasedObject, FormattingMode, &mut RString) -> RResult<(), ()>,
    display: extern "C" fn(&ErasedObject, FormattingMode, &mut RString) -> RResult<(), ()>,
    #[sabi(last_prefix_field)]
    type_id: extern "C" fn()->UTypeId,
}

///////////////////

struct MakeRErrorVTable<T>(T);


impl<T> MakeRErrorVTable<T>
where T:ErrorTrait+'static
{
    const VALUE:RErrorVTableVal=RErrorVTableVal{
        debug: debug_impl::<T>,
        display: display_impl::<T>,
        type_id: new_utypeid::<T>,
    };

    const LIB_VTABLE: &'static WithMetadata<RErrorVTableVal> = {
        &WithMetadata::new(PrefixTypeTrait::METADATA,Self::VALUE)
    };
}

///////////////////

struct MakeBoxedRErrorVTable<T>(T);


impl<T> MakeBoxedRErrorVTable<Box<T>>
where T:?Sized+ErrorTrait+'static
{
    const VALUE:RErrorVTableVal=RErrorVTableVal{
        debug: debug_impl::<Box<T>>,
        display: display_impl::<Box<T>>,
        type_id: new_utypeid::<Box<T>>,
    };

    const LIB_VTABLE: &'static WithMetadata<RErrorVTableVal> = {
        &WithMetadata::new(PrefixTypeTrait::METADATA,Self::VALUE)
    };
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
