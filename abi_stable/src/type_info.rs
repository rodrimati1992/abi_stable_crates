use std::{
    cmp::{Eq, PartialEq},
    fmt,
    mem,
    marker::PhantomData,
};

use crate::{
    utypeid::{UTypeId,new_utypeid}, 
    version::VersionStrings, 
    std_types::StaticStr,
    traits::{ImplType,InterfaceType},
};

pub trait GetTypeInfo {
    const INFO: &'static TypeInfo;
}

#[derive(Debug, Eq, PartialEq)]
#[repr(C)]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct TypeInfo {
    pub size: usize,
    pub alignment: usize,
    pub uid: ReturnValueEquality<UTypeId>,
    pub name: StaticStr,
    pub file: StaticStr,
    pub package: StaticStr,
    pub package_version: VersionStrings,
    #[doc(hidden)]
    pub _private_field: (),
}

impl TypeInfo {
    pub fn is_compatible(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "type:{}\n\
             size:{} alignment:{}\n\
             path:'{}'\n\
             package:'{}'\n\
             package_version:{}\n\
             ",
            self.name, self.size, self.alignment, self.file, self.package, self.package_version
        )
    }
}


//////////////////////////////////////////////////////////////////


/// Helper struct for Wrapping any type in a VirtualWrapper<PointerType<T>>.
pub struct InterfaceFor<T,Interface>(
    PhantomData<fn()->(T,Interface)>
);

impl<T,Interface> GetTypeInfo for InterfaceFor<T,Interface> 
where T:'static
{
    const INFO:&'static TypeInfo=&TypeInfo{
        size:mem::size_of::<T>(),
        alignment:mem::align_of::<T>(),
        uid:ReturnValueEquality{
            function:new_utypeid::<T>
        },
        name:StaticStr::new("<erased>"),
        file:StaticStr::new("<unavailable>"),
        package:StaticStr::new("<unavailable>"),
        package_version:VersionStrings{
            major:StaticStr::new("99"),
            minor:StaticStr::new("99"),
            patch:StaticStr::new("99"),
        },
        _private_field:(),
    };
}


impl<T,Interface> ImplType for InterfaceFor<T,Interface>
where 
    Interface:InterfaceType,
    T:'static,
{
    type Interface=Interface;
}


//////////////////////////////////////////////////////////////////



///
#[macro_export]
macro_rules! impl_get_type_info {
    (
        impl$([$($impl_header:tt)*])?
            GetTypeInfo
        for $type:ident $([$($params:tt)*])?
        $(where[ $($where_clause:tt)* ])?

        version= $major:expr , $minor:expr , $patch:expr ;
    ) => (
        impl $( <$($impl_header)*> )?
            $crate::type_info::GetTypeInfo
        for $type$(<$($params)*>)?
        where
            Self:'static,
            $( $($where_clause)* )?
        {
            const INFO:&'static $crate::type_info::TypeInfo={
                use std::mem;
                use $crate::{
                    type_info::{TypeInfo,ReturnValueEquality},
                    version::{VersionStrings},
                    utypeid::new_utypeid,
                    std_types::StaticStr,
                };

                &TypeInfo{
                    size:mem::size_of::<Self>(),
                    alignment:mem::align_of::<Self>(),
                    uid:ReturnValueEquality{
                        function:new_utypeid::<Self>
                    },
                    name:StaticStr::new(stringify!($type)),
                    file:StaticStr::new(file!()),
                    package:StaticStr::new(env!("CARGO_PKG_NAME")),
                    package_version:VersionStrings{
                        major:StaticStr::new(env!("CARGO_PKG_VERSION_MAJOR")),
                        minor:StaticStr::new(env!("CARGO_PKG_VERSION_MINOR")),
                        patch:StaticStr::new(env!("CARGO_PKG_VERSION_PATCH")),
                    },
                    _private_field:(),
                }
            };
        }
    )
}

////////////////////////////////////////////

#[repr(transparent)]
#[derive(Debug, StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct ReturnValueEquality<T> {
    pub function: extern "C" fn() -> T,
}

impl<T: Eq> Eq for ReturnValueEquality<T> {}

impl<T: PartialEq> PartialEq for ReturnValueEquality<T> {
    fn eq(&self, other: &Self) -> bool {
        (self.function)() == (other.function)()
    }
}
