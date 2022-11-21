//! Contains the `InlineStorage` trait,and related items.

use std::{marker::PhantomData, mem::ManuallyDrop};

/// Type used as the inline storage of a RSmallBox<>/NonExhaustive<>.
///
/// # Safety
///
/// Implementors must:
///
/// - Be types for which all bitpatterns are valid.
///
/// - Not implement Drop,and have no drop glue.
///
pub unsafe trait InlineStorage {}

macro_rules! impl_for_arrays {
    ( ty=$ty:ty , len[ $($len:expr),* $(,)* ] ) => (
        $(
            unsafe impl InlineStorage for [$ty;$len] {}
        )*
    )
}

impl_for_arrays! {
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

impl_for_arrays! {
    ty=u32,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,25,26,27,28,29,
        30,31,32,33,34,35,36,37,38,39,
        40,41,42,43,44,45,46,47,48,
    ]
}

impl_for_arrays! {
    ty=u64,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,
    ]
}

impl_for_arrays! {
    ty=usize,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,25,26,27,28,29,
        30,31,32,33,34,35,36,37,38,39,
        40,41,42,43,44,45,46,47,48,
    ]
}

mod private {
    use super::*;
    pub struct Private<T, const ALIGNMENT: usize>(pub(super) PhantomData<T>);
}
use private::Private;

/// For getting the `AlignTo*` type which aligns `Self` to `ALIGNMENT`.
pub trait AlignerFor<const ALIGNMENT: usize>: Sized {
    // prevents implementations outside this crate.
    #[doc(hidden)]
    const __PRIVATE_12350662443733019984: Private<Self, ALIGNMENT>;

    /// The `AlignTo*` type which aligns `Self` to `ALIGNMENT`.
    type Aligner;
}

/// For getting the `AlignTo*` type which aligns `T` to `ALIGNMENT`.
pub type GetAlignerFor<T, const ALIGNMENT: usize> = <T as AlignerFor<ALIGNMENT>>::Aligner;

macro_rules! declare_alignments {
    (
        $(( $aligner:ident, $alignment:expr),)*
    ) => (
        $(
            #[doc = concat!(
                "Aligns its contents to an address at a multiple of ",
                $alignment,
                " bytes."
            )]
            #[derive(StableAbi, Debug, PartialEq, Eq, Copy, Clone)]
            #[repr(C)]
            #[repr(align($alignment))]
            pub struct $aligner<Inline>(pub Inline);

            unsafe impl<Inline> InlineStorage for $aligner<Inline>
            where
                Inline: InlineStorage,
            {}

            impl<T> AlignerFor<$alignment> for T {
                #[doc(hidden)]
                const __PRIVATE_12350662443733019984: Private<T, $alignment> = Private(PhantomData);

                type Aligner = $aligner<T>;
            }
        )*
    )
}

/// Helper types related to the alignemnt of inline storage.
pub mod alignment {
    use super::*;

    /*
        fn main(){
            for pow in 0..=16 {
                let val = 1u32 << pow;
                println!("        (AlignTo{val}, {val}),")
            }
        }
    */
    declare_alignments! {
        (AlignTo1, 1),
        (AlignTo2, 2),
        (AlignTo4, 4),
        (AlignTo8, 8),
        (AlignTo16, 16),
        (AlignTo32, 32),
        (AlignTo64, 64),
        (AlignTo128, 128),
        (AlignTo256, 256),
        (AlignTo512, 512),
        (AlignTo1024, 1024),
        (AlignTo2048, 2048),
        (AlignTo4096, 4096),
        (AlignTo8192, 8192),
        (AlignTo16384, 16384),
        (AlignTo32768, 32768),
    }

    /// Aligns its contents to an address to an address at
    /// a multiple of the size of a pointer.
    #[repr(C)]
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(target_pointer_width = "128", repr(C, align(16)))]
    #[cfg_attr(target_pointer_width = "64", repr(C, align(8)))]
    #[cfg_attr(target_pointer_width = "32", repr(C, align(4)))]
    #[cfg_attr(target_pointer_width = "16", repr(C, align(2)))]
    pub struct AlignToUsize<Inline>(pub Inline);

    unsafe impl<Inline> InlineStorage for AlignToUsize<Inline> where Inline: InlineStorage {}
}

///////////////////////////////////////////////////////////////////////////////

#[repr(transparent)]
pub(crate) struct ScratchSpace<T, Inline> {
    #[allow(dead_code)]
    inner: ScratchSpaceInner<T, Inline>,
}

#[repr(C)]
union ScratchSpaceInner<T, Inline> {
    value: ManuallyDrop<T>,
    storage: ManuallyDrop<Inline>,
    uninit: (),
}

// These constructors don't require `Inline: InlineStorage` because
// the `storage` field is only used for its side/alignment,
// it is never actually constructed.
impl<T, Inline> ScratchSpace<T, Inline> {
    #[inline]
    #[allow(dead_code)]
    #[track_caller]
    pub(crate) const fn uninit() -> Self {
        Self::assert_fits_within_storage();
        Self {
            inner: ScratchSpaceInner { uninit: () },
        }
    }

    #[inline]
    #[allow(dead_code)]
    #[track_caller]
    pub(crate) const fn new(value: T) -> Self {
        Self::assert_fits_within_storage();
        Self {
            inner: ScratchSpaceInner {
                value: ManuallyDrop::new(value),
            },
        }
    }

    /// Asserts that `T` fits within `Inline`,with the correct alignment and size.
    #[track_caller]
    const fn assert_fits_within_storage() {
        use crate::nonexhaustive_enum::AssertCsArgs;

        crate::nonexhaustive_enum::assert_correct_storage::<T, Inline>(AssertCsArgs::UNKNOWN)
    }
}
