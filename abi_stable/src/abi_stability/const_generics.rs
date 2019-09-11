//! This module implements the trait object used to check const generics.

use crate::{
    abi_stability::{TypeCheckerMut,check_layout_compatibility},
    erased_types::{
        c_functions::{adapt_std_fmt,debug_impl,partial_eq_impl},
        FormattingMode,
    },
    marker_type::ErasedObject,
    prefix_type::{PrefixTypeTrait,WithMetadata},
    std_types::{RBoxError,RString,RResult,ROk,RErr},
    sabi_types::StaticRef,
    type_layout::TypeLayout,
    StableAbi,
};

use std::{
    cmp::{Eq,PartialEq},
    fmt::{self,Debug},
    marker::PhantomData,
};


///////////////////////////////////////////////////////////////////////////////


/// A trait object used to check equality between const generic parameters.
#[repr(C)]
#[derive(Copy,Clone,StableAbi)]
pub struct ConstGeneric{
    ptr:*const ErasedObject,
    vtable:StaticRef<ConstGenericVTable>,
}

impl ConstGeneric{
    pub const fn new<T>(this:&'static T, vtable_for:ConstGenericVTableFor<T>)->Self{
        Self{
            ptr: this as *const T as *const ErasedObject,
            vtable: vtable_for.vtable,
        }
    }

    pub fn is_equal(&self,other:&Self,mut checker:TypeCheckerMut<'_>)->Result<bool,RBoxError> {
        match checker.check_compatibility(self.vtable.layout(),other.vtable.layout()) {
            ROk(_)=>unsafe{
                Ok(self.vtable.partial_eq()( &*self.ptr, &*other.ptr ))
            },
            RErr(e)=>{
                Err(RBoxError::new(e))
            }
        }
    }
}

impl Debug for ConstGeneric{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        unsafe{
            adapt_std_fmt::<ErasedObject>(
                &*self.ptr,
                self.vtable.debug(),
                f
            )
        }
    }
}


// Make sure that this isn't called within `check_layout_compatibility` itself,
// since it would cause infinite recursion.
impl PartialEq for ConstGeneric{
    fn eq(&self,other:&Self)->bool{
        if check_layout_compatibility(self.vtable.layout(),other.vtable.layout()).is_err() { 
            false
        }else{
            unsafe{
                self.vtable.partial_eq()( &*self.ptr, &*other.ptr )
            }
        }
    }
}

impl Eq for ConstGeneric{}



///////////////////////////////////////////////////////////////////////////////


#[doc(hidden)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_struct="ConstGenericVTable")))]
#[sabi(missing_field(panic))]
pub struct ConstGenericVTableVal{
    layout:&'static TypeLayout,
    partial_eq:unsafe extern fn(&ErasedObject,&ErasedObject)->bool,
    debug:unsafe extern "C" fn(&ErasedObject,FormattingMode,&mut RString)->RResult<(),()>,
}

/// A type that contains the vtable stored in the `ConstGeneric` constructed from a `T`.
/// This is used as a workaround for `const fn` not allowing trait bounds,
/// `GetConstGenericVTable` is the trait in this case .
pub struct ConstGenericVTableFor<T>{
    vtable:StaticRef<ConstGenericVTable>,
    _marker:PhantomData<T>,
}


///////////////////////////////////////////////////////////////////////////////


pub trait GetConstGenericVTable:Sized {
    #[doc(hidden)]
    const _VTABLE_STATIC: StaticRef<WithMetadata<ConstGenericVTableVal>> ;
    const VTABLE:ConstGenericVTableFor<Self>;
}



impl<This> GetConstGenericVTable for This
where
    This:StableAbi+Eq+PartialEq+Debug
{
    #[doc(hidden)]
    const _VTABLE_STATIC: StaticRef<WithMetadata<ConstGenericVTableVal>> = {
        StaticRef::from_ref(&WithMetadata::new(
            PrefixTypeTrait::METADATA,
            ConstGenericVTableVal{
                layout: <Self as StableAbi>::LAYOUT,
                partial_eq: partial_eq_impl::<Self>,
                debug: debug_impl::<Self>,
            }
        ))
    };

    const VTABLE:ConstGenericVTableFor<Self>=ConstGenericVTableFor{
        vtable: WithMetadata::staticref_as_prefix(Self::_VTABLE_STATIC),
        _marker: PhantomData,
    };
}
