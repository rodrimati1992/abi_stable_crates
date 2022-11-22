use super::*;

////////////////////////////////////////////////////////////////////////////////

#[allow(non_camel_case_types)]
#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct _private_TypeLayoutDerive {
    pub shared_vars: &'static SharedVars,
    pub mono: &'static MonoTypeLayout,
    pub abi_consts: AbiConsts,
    pub data: GenericTLData,
    pub tag: Option<&'static Tag>,
    pub extra_checks: Option<&'static ManuallyDrop<StoredExtraChecks>>,
}

#[allow(non_camel_case_types)]
#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct _private_MonoTypeLayoutDerive {
    pub name: RStr<'static>,
    pub item_info: ItemInfo,
    pub data: MonoTLData,
    pub generics: CompGenericParams,
    pub repr_attr: ReprAttr,
    pub mod_refl_mode: ModReflMode,
    pub phantom_fields: RSlice<'static, CompTLFieldRepr>,
    pub shared_vars: MonoSharedVars,
}

/// Information about where a type was declared.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct ItemInfo {
    /// The package where the type was defined,and the version string.
    /// With the `package;version_number` format.
    package_and_version: RStr<'static>,
    /// The line in the file where the type was defined.
    pub line: u32,
    /// The full path to the module where the type was defined,
    /// including the package name
    pub mod_path: ModPath,
}

impl ItemInfo {
    #[doc(hidden)]
    pub const fn new(package_and_version: &'static str, line: u32, mod_path: ModPath) -> Self {
        Self {
            package_and_version: RStr::from_str(package_and_version),
            line,
            mod_path,
        }
    }

    /// Constructs an ItemInfo for a std primitive
    pub const fn primitive() -> Self {
        Self {
            package_and_version: RStr::from_str("std;1.0.0"),
            line: 0,
            mod_path: ModPath::PRELUDE,
        }
    }

    /// Constructs an ItemInfo for an std type with a path.
    pub const fn std_type_in(mod_path: NulStr<'static>) -> Self {
        Self {
            package_and_version: RStr::from_str("std;1.0.0"),
            line: 0,
            mod_path: ModPath::inside(mod_path),
        }
    }

    /// Constructs an ItemInfo for a type in a package and the path to its module.
    ///
    /// `package_and_version` must be formatted like this:`package_name;major.minor.patch`
    ///
    /// `mod_path` must include the crate name.
    pub const fn package_and_mod(
        package_and_version: &'static str,
        mod_path: NulStr<'static>,
    ) -> Self {
        Self {
            package_and_version: RStr::from_str(package_and_version),
            line: 0,
            mod_path: ModPath::inside(mod_path),
        }
    }

    /// Gets the package name and an unparsed package version.
    pub fn package_and_version(&self) -> (&'static str, &'static str) {
        let pav = self.package_and_version.as_str();
        match pav.find(';') {
            Some(separator) => (&pav[..separator], &pav[(separator + 1)..]),
            None => (pav, ""),
        }
    }

    /// Gets the package name.
    pub fn package(&self) -> &'static str {
        self.package_and_version().0
    }

    /// Gets the unparsed package version.
    pub fn version(&self) -> &'static str {
        self.package_and_version().1
    }
}
