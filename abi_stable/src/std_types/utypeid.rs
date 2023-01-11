//! An ffi-safe equivalent of `std::any::TypeId`.
//!
//! No types coming from different dynamic libraries compare equal.

use std::{
    any::TypeId,
    hash::{Hash, Hasher},
    mem,
    sync::atomic::AtomicUsize,
};

use crate::{sabi_types::MaybeCmp, EXECUTABLE_IDENTITY};

///////////////////////////////////////////////////////////////////////////////

/// `extern "C" fn` version of `UTypeId::new`.
///
/// # Example
///
/// ```
/// use abi_stable::std_types::utypeid::new_utypeid;
/// use std::collections::HashMap;
///
/// let hashmap_id = new_utypeid::<HashMap<String, String>>();
/// let vec_id = new_utypeid::<Vec<String>>();
/// let u32_id = new_utypeid::<u32>();
///
/// assert_eq!(hashmap_id, hashmap_id);
/// assert_eq!(vec_id, vec_id);
/// assert_eq!(u32_id, u32_id);
///
/// assert_ne!(vec_id, hashmap_id);
/// assert_ne!(u32_id, hashmap_id);
/// assert_ne!(vec_id, u32_id);
///
/// ```
pub extern "C" fn new_utypeid<T>() -> UTypeId
where
    T: 'static,
{
    UTypeId::new::<T>()
}

#[doc(hidden)]
pub extern "C" fn some_utypeid<T>() -> MaybeCmp<UTypeId>
where
    T: 'static,
{
    MaybeCmp::Just(UTypeId::new::<T>())
}

#[doc(hidden)]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn no_utypeid() -> MaybeCmp<UTypeId> {
    MaybeCmp::Nothing
}

/// An ffi-safe equivalent of `std::any::TypeId` that
/// can compare types across dynamic libraries.
///
/// No `UTypeId` constructed in different dynamic libraries compare equal.
///
/// # Example
///
/// ```
/// use abi_stable::std_types::UTypeId;
///
/// assert_eq!(UTypeId::new::<()>(), UTypeId::new::<()>());
/// assert_eq!(UTypeId::new::<Box<String>>(), UTypeId::new::<Box<String>>());
///
/// assert_ne!(UTypeId::new::<()>(), UTypeId::new::<Vec<()>>());
/// assert_ne!(UTypeId::new::<Box<String>>(), UTypeId::new::<&str>());
/// ```
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash, StableAbi)]
pub struct UTypeId {
    /// A dummy AtomicUsize used as the identity of the dynamic-library/executable
    executable_identity: *const AtomicUsize,
    /// An array containing the butes of a TypeId,
    /// with `MAX_TYPE_ID_SIZE` bytes of space in case it becomes larger.
    type_id_array: [u8; MAX_TYPE_ID_SIZE],
}

unsafe impl Send for UTypeId {}
unsafe impl Sync for UTypeId {}

impl UTypeId {
    /// Constructs `UTypeId` from a type that satisfies the `'static` bound.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::UTypeId;
    /// use std::collections::HashMap;
    ///
    /// let id = UTypeId::new::<HashMap<String, String>>();
    /// # drop(id);
    /// ```
    #[inline(always)]
    pub fn new<T>() -> Self
    where
        T: 'static,
    {
        Self {
            executable_identity: &EXECUTABLE_IDENTITY,
            type_id_array: get_typeid::<T>(),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////

type TypeIdArray = [u8; mem::size_of::<TypeId>()];

const MAX_TYPE_ID_SIZE: usize = 16;

#[inline(always)]
fn get_typeid<T: 'static>() -> [u8; MAX_TYPE_ID_SIZE] {
    let mut hasher = TypeIdHasher {
        value: [0; MAX_TYPE_ID_SIZE],
        written: 0,
    };
    TypeId::of::<T>().hash(&mut hasher);
    hasher.value
}

#[derive(Default)]
struct TypeIdHasher {
    value: [u8; MAX_TYPE_ID_SIZE],
    written: usize,
}

impl TypeIdHasher {
    #[inline(never)]
    #[cold]
    fn overflow_msg() -> ! {
        eprintln!(
            "TypeId requires writing more than {} bytes to the hasher.",
            MAX_TYPE_ID_SIZE
        );
        ::std::process::abort();
    }
}

impl Hasher for TypeIdHasher {
    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        let _: [u8; MAX_TYPE_ID_SIZE - mem::size_of::<TypeId>()];
        if bytes.len() == mem::size_of::<TypeId>() {
            unsafe {
                let into = (&mut self.value) as *mut _ as *mut TypeIdArray;
                let from = bytes.as_ptr() as *const TypeIdArray;
                *into = *from;
            }
            self.written = mem::size_of::<TypeId>();
            return;
        }
        let old_pos = self.written;
        self.written += bytes.len();
        if self.written <= MAX_TYPE_ID_SIZE {
            self.value[old_pos..self.written].copy_from_slice(bytes);
        } else {
            Self::overflow_msg()
        }
    }

    #[inline(always)]
    fn finish(&self) -> u64 {
        // I'm not gonna call this
        0
    }
}
