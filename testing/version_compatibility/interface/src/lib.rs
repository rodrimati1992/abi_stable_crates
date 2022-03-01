#[cfg(feature = "new")]
extern crate new_abi_stable as abi_stable;

#[cfg(not(feature = "new"))]
extern crate old_abi_stable as abi_stable;

use abi_stable::{library::RootModule, marker_type::NonOwningPhantom, sabi_types::VersionStrings};

mod many_types {
    use std::{marker::PhantomData, sync::atomic};

    use abi_stable::{
        external_types::{
            crossbeam_channel::{RReceiver, RSender},
            RMutex, ROnce, RRwLock,
        },
        marker_type,
        prefix_type::PrefixRef,
        std_types::*,
    };

    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct ManyTypes(
        &'static mut (),
        &'static mut i32,
        &'static (),
        &'static i32,
        &'static &'static (),
        &'static mut &'static (),
        &'static &'static mut (),
        atomic::AtomicPtr<()>,
        atomic::AtomicPtr<i32>,
        *const (),
        *const i32,
        *mut (),
        *mut i32,
        [(); 0],
        [(); 1],
        [(); 2],
        [(); 3],
        [u32; 3],
        i32,
        u32,
        bool,
        atomic::AtomicBool,
        atomic::AtomicIsize,
        atomic::AtomicUsize,
        std::num::NonZeroU32,
        std::num::NonZeroU16,
        std::ptr::NonNull<()>,
        std::ptr::NonNull<i32>,
        RHashMap<RString, RString>,
        RHashMap<RString, i32>,
        RHashMap<i32, RString>,
        RHashMap<i32, i32>,
        RVec<()>,
        RVec<i32>,
        RSlice<'static, ()>,
        RSlice<'static, i32>,
        RSliceMut<'static, ()>,
        RSliceMut<'static, i32>,
        Option<&'static ()>,
        Option<&'static u32>,
        Option<extern "C" fn()>,
        ROption<()>,
        ROption<u32>,
        RCowStr<'static>,
        RCowSlice<'static, u32>,
        RArc<()>,
        RArc<u32>,
        RBox<()>,
        RBox<u32>,
        RBoxError,
        SendRBoxError,
        UnsyncRBoxError,
        RCmpOrdering,
        PhantomData<()>,
        PhantomData<RString>,
        RMutex<()>,
        RMutex<RString>,
        RRwLock<()>,
        RRwLock<RString>,
        RSender<()>,
        RSender<RString>,
        RReceiver<()>,
        RReceiver<RString>,
        ROnce,
        marker_type::SyncSend,
        marker_type::UnsyncUnsend,
        marker_type::UnsyncSend,
        marker_type::SyncUnsend,
        marker_type::NotCopyNotClone,
        marker_type::ErasedObject<u32>,
        marker_type::ErasedObject<RString>,
        PrefixRef<marker_type::ErasedPrefix>,
        marker_type::UnsafeIgnoredType<u32>,
        marker_type::NonOwningPhantom<u32>,
        marker_type::NonOwningPhantom<RString>,
        f32,
        f64,
    );
}

pub use many_types::ManyTypes;

#[repr(C)]
#[derive(abi_stable::StableAbi)]
#[sabi(kind(Prefix(prefix_ref = "RootMod_Ref")))]
#[sabi(missing_field(panic))]
pub struct RootMod {
    pub abi_stable_version: VersionStrings,
    pub _marker: NonOwningPhantom<many_types::ManyTypes>,
    #[cfg(feature = "new_abi_stable")]
    pub _marker2: NonOwningPhantom<many_types::ManyTypes2>,
}

impl RootModule for RootMod_Ref {
    abi_stable::declare_root_module_statics! {RootMod_Ref}

    const BASE_NAME: &'static str = "version_compatibility_impl";
    const NAME: &'static str = "version_compatibility_impl";
    const VERSION_STRINGS: VersionStrings = abi_stable::package_version_strings!();
}
