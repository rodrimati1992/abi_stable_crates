use super::*;


////////////////////////////////////////////////////////////////////////////////


/// The parameters for `TypeLayout::from_params`.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TypeLayoutParams {
    pub name: &'static str,
    pub item_info:ItemInfo,
    pub data: TLData,
    pub generics: GenericParams,
}


#[doc(hidden)]
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct _private_TypeLayoutDerive {
    pub name: &'static str,
    pub item_info:ItemInfo,
    pub data: TLData,
    pub generics: GenericParams,
    pub phantom_fields: &'static [TLField],
    pub tag:Tag,
    pub mod_refl_mode:ModReflMode,
    pub repr_attr:ReprAttr,
}


#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq,StableAbi)]
pub struct ItemInfo{
    pub package:StaticStr,
    pub package_version: VersionStrings,
    pub file:StaticStr,
    pub line:u32,
    pub mod_path:ModPath,
}

impl ItemInfo{
    pub const fn new(
        package:&'static str,
        package_version: VersionStrings,
        file:&'static str,
        line:u32,
        mod_path:ModPath,
    )->Self{
        Self{
            package: StaticStr::new(package),
            package_version,
            file:StaticStr::new(file),
            line,
            mod_path,
        }
    }

    pub const fn primitive()->Self{
        Self{
            package: StaticStr::new("std"),
            package_version: VersionStrings {
                major: StaticStr::new("1"),
                minor: StaticStr::new("0"),
                patch: StaticStr::new("0"),
            },
            file:StaticStr::new("<standard_library>"),
            line:0,
            mod_path:ModPath::Prelude,
        }
    }

    pub const fn std_type_in(path:&'static str)->Self{
        Self{
            package: StaticStr::new("std"),
            package_version: VersionStrings {
                major: StaticStr::new("1"),
                minor: StaticStr::new("0"),
                patch: StaticStr::new("0"),
            },
            file:StaticStr::new("<standard_library>"),
            line:0,
            mod_path:ModPath::inside(path),
        }
    }

    /// mod_path must include the crate name.
    pub const fn package_and_mod(package:&'static str,mod_path:&'static str)->Self{
        Self{
            package: StaticStr::new(package),
            package_version: VersionStrings {
                major: StaticStr::new("0"),
                minor: StaticStr::new("0"),
                patch: StaticStr::new("0"),
            },
            file:StaticStr::new("<unavailable>"),
            line:0,
            mod_path:ModPath::inside(mod_path),
        }
    }
}

