/*!
Contains the `InlineStorage` trait,and related items.
*/

/**
Type used as the inline storage of a RSmallBox<>/NonExhaustive<>.

# Safety

Implementors must:

- Be types for which all bitpatterns are valid.

- Not implement Drop,and have no drop glue.

*/
pub unsafe trait InlineStorage{}


macro_rules! impl_for_arrays {
    ( ty=$ty:ty , len[ $($len:expr),* $(,)* ] ) => (
        $(
            unsafe impl InlineStorage for [$ty;$len] {}
        )*
    )
}


impl_for_arrays!{
    ty=u8,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,25,26,27,28,29,
        30,31,32,33,34,35,36,37,38,39,
        40,41,42,43,44,45,46,47,48,49,
        50,51,52,53,54,55,56,57,58,59,
        60,61,62,63,64,
    ]
}

impl_for_arrays!{
    ty=u32,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,25,26,27,28,29,
        30,31,32,33,34,35,36,37,38,39,
        40,41,42,43,44,45,46,47,48,
    ]
}

impl_for_arrays!{
    ty=u64,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,
    ]
}

impl_for_arrays!{
    ty=usize,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,25,26,27,28,29,
        30,31,32,33,34,35,36,37,38,39,
        40,41,42,43,44,45,46,47,48,
    ]
}


macro_rules! declare_alignments {
    (
        $(( $aligner:ident, $alignment:expr ),)*
    ) => (
        $(
            #[repr(C)]
            #[repr(align($alignment))]
            pub struct $aligner<Inline>{
                inline:Inline,
            }
            
            unsafe impl<Inline> InlineStorage for $aligner<Inline>
            where
                Inline:InlineStorage,
            {}
        )*
    )
}


/// Helper types related to the alignemnt of inline storage.
pub mod alignment{
    use super::*;
    
    declare_alignments!{
        ( AlignTo1,1 ),
        ( AlignTo2,2 ),
        ( AlignTo4,4 ),
        ( AlignTo8,8 ),
        ( AlignTo16,16 ),
        ( AlignTo32,32 ),
        ( AlignTo64,64 ),
        ( AlignTo128,128 ),
    }
}



///////////////////////////////////////////////////////////////////////////////

#[cfg(not(any(rust_1_36,feature="rust_1_36")))]
pub(crate) type UsedUninit<Inline>=std::mem::ManuallyDrop<Inline>;

#[cfg(any(rust_1_36,feature="rust_1_36"))]
pub(crate) type UsedUninit<Inline>=std::mem::MaybeUninit<Inline>;


/// Used internally to avoid requiring Rust 1.36.0 .
#[repr(transparent)]
pub(crate) struct ScratchSpace<Inline>{
    #[allow(dead_code)]
    storage:UsedUninit<Inline>,
}

impl<Inline> ScratchSpace<Inline>{
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn new<T>(value:T)->Self
    where
        Inline:InlineStorage
    {
        Self::assert_fits_within_storage::<T>();
        unsafe{
            Self::new_unchecked(value)
        }
    }

/**
# Safety

You must ensure that `T` has a compatible size/alignement with `Inline`,
and that `Inline` si valid for all bitpatterns.
*/
    #[inline]
    #[allow(dead_code)]
    pub(crate) unsafe fn new_unchecked<T>(value:T)->Self{
        let mut this=Self::uninit_unbounded();
        (&mut this as *mut Self as *mut T).write(value);
        this
    }
    #[inline]
    pub(crate) fn uninit()->Self
    where
        Inline:InlineStorage
    {
        unsafe{
            Self::uninit_unbounded()
        }
    }

    /// Asserts that `T` fits within `Inline`,with the correct alignment and size.
    fn assert_fits_within_storage<T>(){
        let align_val=std::mem::align_of::<T>();
        let align_storage=std::mem::align_of::<Inline>();
        assert!(
            align_val <= align_storage,
            "The alignment of the storage is lower than the value:\n\t{} < {}",
            align_storage,align_val,
        );
        let size_val=std::mem::size_of::<T>();
        let size_storage=std::mem::size_of::<Inline>();
        assert!(
            size_val <= size_storage,
            "The size of the storage is smaller than the value:\n\t{} < {}",
            size_storage,size_val,
        );
    }
}

#[cfg(not(any(rust_1_36,feature="rust_1_36")))]
impl<Inline> ScratchSpace<Inline>{

/**
# Safety

You must ensure that `Inline` is valid for all bitpatterns,ie:it implements `InlineStorage`.
*/
    #[inline]
    pub(crate) unsafe fn uninit_unbounded()->Self{
        unsafe{
            std::mem::uninitialized()
        }
    }
}

#[cfg(any(rust_1_36,feature="rust_1_36"))]
impl<Inline> ScratchSpace<Inline>{
/**
# Safety

You must ensure that `Inline` is valid for all bitpatterns,ie:it implements `InlineStorage`.
*/
    #[inline]
    pub(crate) unsafe fn uninit_unbounded()->Self{
        unsafe{
            Self{
                storage:std::mem::MaybeUninit::uninit()
            }
        }
    }
}

