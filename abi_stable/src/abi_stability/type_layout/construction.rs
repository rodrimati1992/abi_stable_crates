use super::*;


////////////////////////////////////////////////////////////////////////////////


/// The parameters for `TypeLayout::from_params`.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TypeLayoutParams {
    /// The name of the type,without generic parameters.
    pub name: &'static str,
    /// Information about where the type was declared,
    /// generally created with `make_item_info!()`.
    pub item_info:ItemInfo,
    /// The definition of the type.
    pub data: TLData,
    /// The generic parameters of the type,
    /// generally constructed with the `tl_genparams` macro.
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


/// Information about where a type was declared.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq,StableAbi)]
pub struct ItemInfo{
    /// The package where the type was defined.
    pub package:StaticStr,
    /// The version of the package where the type was defined.
    pub package_version: VersionStrings,
    /// The file where the type was defined.
    pub file:StaticStr,
    /// The line in the file where the type was defined.
    pub line:u32,
    /// The full path to the module where the type was defined,
    /// including the package name
    pub mod_path:ModPath,
}

impl ItemInfo{
    #[doc(hidden)]
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

    /// Constructs an ItemInfo for a std primitive
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

    /// Constructs an ItemInfo for an std type with a path.
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

    /// Constructs an ItemInfo for a type in a package and the path to its module.
    ///
    /// `mod_path` must include the crate name.
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

