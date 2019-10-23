
#[cfg(feature="new")]
extern crate new_abi_stable as abi_stable;

#[cfg(feature="old")]
extern crate old_abi_stable as abi_stable;

#[cfg(any(
    not(any(feature="new",feature="old")),
    all(feature="new",feature="old")
))]
compile_error!{"either the new or old feature has to be enabled"}


use std::marker::PhantomData;

use abi_stable::{
    sabi_types::VersionStrings,
    library::RootModule,
};


mod many_types{
    use std::{
        marker::PhantomData,
        sync::atomic,
    };
    
    use abi_stable::{
        external_types::{
            crossbeam_channel::{RReceiver,RSender},
            RMutex,RRwLock,ROnce
        },
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
        RHashMap<RString,RString>,
        RHashMap<RString,i32>,
        RHashMap<i32,RString>,
        RHashMap<i32,i32>,
        RVec<()>,
        RVec<i32>,
        RSlice<'static, ()>,
        RSlice<'static, i32>,
        RSliceMut<'static, ()>,
        RSliceMut<'static, i32>,
        StaticSlice<()>,
        StaticSlice<i32>,
        StaticStr,
        Option<&'static ()>,
        Option<&'static u32>,
        Option<extern "C" fn()>,
        ROption<()>,
        ROption<u32>,
        RCow<'static, str>,
        RCow<'static, [u32]>,
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
    );    
}

pub use many_types::ManyTypes;

#[repr(C)]
#[derive(abi_stable::StableAbi)]
#[sabi(kind(Prefix(prefix_struct="RootMod")))]
#[sabi(missing_field(panic))]
pub struct RootModVal{
    pub abi_stable_version:VersionStrings,
    pub _marker:PhantomData<many_types::ManyTypes>,
}

impl RootModule for RootMod {
    abi_stable::declare_root_module_statics!{RootMod}

    const BASE_NAME: &'static str = "version_compatibility_impl";
    const NAME: &'static str = "version_compatibility_impl";
    const VERSION_STRINGS: VersionStrings = abi_stable::package_version_strings!();
}

