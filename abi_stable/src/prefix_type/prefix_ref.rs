use crate::{
    abi_stability::{GetStaticEquivalent_, GetStaticEquivalent, PrefixStableAbi, StableAbi},
    pointer_trait::NonNullPointer,
    prefix_type::{WithMetadata_, PrefixMetadata},
    reexports::True,
    reflection::ModReflMode,
    type_layout::{
        CompTLField, GenericTLData, LifetimeRange, MonoTLData, MonoTypeLayout, ReprAttr,
        TypeLayout,
    },
};

use std::{
    fmt::{self, Debug},
    ptr::NonNull,
};


/// A reference to a prefix type.
#[repr(transparent)]
pub struct PrefixRef<P>{
    ptr: NonNull<WithMetadata_<P, P>>,
}

impl<P> Clone for PrefixRef<P>{
    #[inline]
    fn clone(&self)->Self{
        *self
    }
}

impl<P> Copy for PrefixRef<P>{}

unsafe impl<'a, P:'a> Sync for PrefixRef<P>
where &'a WithMetadata_<P, P>:Sync
{}

unsafe impl<'a, P:'a> Send for PrefixRef<P>
where &'a WithMetadata_<P, P>:Send
{}

impl<P> Debug for PrefixRef<P> {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        let metadata = self.metadata();
        f.debug_struct("PrefixRef")
         .field("metadata", &metadata)
         .field("value_type", &std::any::type_name::<P>())
         .finish()
    }
}

impl<P> PrefixRef<P>{
    #[inline]
    pub const unsafe fn new<T>(ptr: *const WithMetadata_<T, P>) -> Self {
        Self{
            ptr: NonNull::new_unchecked(
                ptr as *const WithMetadata_<P, P>
                    as *mut WithMetadata_<P, P>
            )
        }
    }

    #[inline]
    pub fn metadata(self) -> PrefixMetadata<P, P> {
        unsafe{
            (*self.ptr.as_ptr()).metadata
        }
    }

    #[inline]
    pub fn prefix<'a>(self)-> &'a P {
        unsafe{
            &(*self.ptr.as_ptr()).value
        }
    }

    #[inline]
    pub const fn as_ptr(self)-> *const WithMetadata_<P, P> {
        self.ptr.as_ptr()
    }

    /// 
    /// # Safety
    /// 
    /// The pointer must come from `PrefixRef::as_ptr`.
    #[inline]
    pub const unsafe fn from_raw(ptr: *const WithMetadata_<P, P> )-> Self {
        Self{
            ptr: NonNull::new_unchecked(
                ptr as *mut WithMetadata_<P, P>
            )
        }
    }

    pub const unsafe fn cast<T>(self)->PrefixRef<T>{
        PrefixRef{
            ptr: self.ptr.cast()
        }
    }
}


unsafe impl<P> GetStaticEquivalent_ for  PrefixRef<P>
where
    P: GetStaticEquivalent_,
{
    type StaticEquivalent = PrefixRef<GetStaticEquivalent<P>>;
}

unsafe impl<P> StableAbi for PrefixRef<P>
where 
    P: PrefixStableAbi,
{
    type IsNonZeroType = True;

    const LAYOUT: &'static TypeLayout = {
        const MONO_TYPE_LAYOUT:&'static MonoTypeLayout=&MonoTypeLayout::new(
            *mono_shared_vars,
            rstr!("PrefixRef"),
            make_item_info!(),
            MonoTLData::struct_(rslice![]),
            tl_genparams!('a;0;),
            ReprAttr::Transparent,
            ModReflMode::DelegateDeref{layout_index:0},
            rslice![CompTLField::std_field(field0,LifetimeRange::EMPTY,0)],
        );

        make_shared_vars!{
            let (mono_shared_vars,shared_vars)={
                strings={ field0:"0", },
                prefix_type_layouts=[P],
            };
        }

        &TypeLayout::from_std::<Self>(
            shared_vars,
            MONO_TYPE_LAYOUT,
            Self::ABI_CONSTS,
            GenericTLData::Struct,
        )
    };
}

unsafe impl<P> NonNullPointer for PrefixRef<P> {
    type Target = WithMetadata_<P, P>;

    #[inline(always)]
    fn to_nonnull(self) -> NonNull<WithMetadata_<P, P>> {
        self.ptr
    }

    #[inline(always)]
    unsafe fn from_nonnull(ptr: NonNull<WithMetadata_<P, P>>)-> Self {
        Self{ptr}
    }
}
