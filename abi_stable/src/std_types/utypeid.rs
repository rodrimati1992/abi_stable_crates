/*!
An ffi-safe equivalent of ::std::any::TypeId.

No types coming from different dynamic libraries compare equal.
*/

use std::{
    any::TypeId,
    hash::{Hash, Hasher},
    mem,
    sync::atomic::AtomicUsize,
};

use crate::{
    EXECUTABLE_IDENTITY,
    sabi_types::MaybeCmp,
};

///////////////////////////////////////////////////////////////////////////////

/// `extern "C" fn` version of UTypeId::new.
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
pub extern "C" fn no_utypeid() -> MaybeCmp<UTypeId>{
    MaybeCmp::Nothing
}


/// A TypeId that can compare types across dynamic libraries.
///
/// No types coming from different dynamic libraries compare equal.
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash, StableAbi)]
pub struct UTypeId {
    /// A dummy AtomicUsize used as the identity of the dynamic-library/executable
    executable_identity: *const AtomicUsize,
    /// An array containing the butes of a TypeId,
    /// with `MAX_TYPE_ID_SIZE` bytes of space in case it becomes larger.
    type_id_array: [u8; MAX_TYPE_ID_SIZE],
}

impl UTypeId {
    /// Constructs UTypeId from a type that satisfies the `'static` bound.
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

