//! This is an example `interface crate`,
//! where all publically available modules(structs of function pointers) and types are declared,
//!
//! These crate test a few of the errors that are returned when loading dynamic libraries

use abi_stable::{
    library::RootModule, package_version_strings, sabi_types::VersionStrings, StableAbi,
};

impl RootModule for TestingMod_Ref {
    abi_stable::declare_root_module_statics! {TestingMod_Ref}

    const BASE_NAME: &'static str = "testing_1_loading_errors";
    const NAME: &'static str = "testing_1_loading_errors";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = "TestingMod_Ref")))]
#[sabi(missing_field(panic))]
pub struct TestingMod {
    #[sabi(last_prefix_field)]
    pub a: u32,
    pub b: u32,
    pub c: u32,
}

////////////////////////////////////////////////////////////////////////////////

/// This type is used to test that errors from types with an incompatble ABI can be printed.
///
/// The reason that needs to be printed is because the
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = "WithIncompatibleLayout_Ref")))]
pub struct WithIncompatibleLayout {
    #[sabi(last_prefix_field)]
    pub __foo: u64,
}

impl RootModule for WithIncompatibleLayout_Ref {
    abi_stable::declare_root_module_statics! {WithIncompatibleLayout_Ref}

    const BASE_NAME: &'static str = "testing_1_loading_errors";
    const NAME: &'static str = "testing_1_loading_errors";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}

////////////////////////////////////////////////////////////////////////////////

/// This type is used to test that errors from types with an incompatble ABI can be printed.
///
/// The reason that needs to be printed is because the
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = "NonAbiStableLib_Ref")))]
pub struct NonAbiStableLib {
    #[sabi(last_prefix_field)]
    pub __foo: u64,
}

impl RootModule for NonAbiStableLib_Ref {
    abi_stable::declare_root_module_statics! {NonAbiStableLib_Ref}

    const BASE_NAME: &'static str = "non_abi_stable_lib";
    const NAME: &'static str = "non_abi_stable_lib";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}

////////////////////////////////////////////////////////////////////////////////

/// Parameters for the program passed through environment variables.
///
/// The reason that env vars are used instead of command line arguments is because
/// both the dynamic library and the executable can see the env vars.
#[derive(Debug)]
pub struct EnvVars {
    /// Whether the dynamic library returns an error.
    pub return_what: ReturnWhat,
}

#[derive(Debug)]
pub enum ReturnWhat {
    Ok,
    Error,
    Panic,
}

#[derive(Debug)]
pub struct ParseReturnWhatError(String);

impl std::str::FromStr for ReturnWhat {
    type Err = ParseReturnWhatError;

    fn from_str(str: &str) -> Result<Self, ParseReturnWhatError> {
        let trimmed = str.trim();
        if trimmed.starts_with("ok") {
            Ok(ReturnWhat::Ok)
        } else if trimmed.starts_with("error") {
            Ok(ReturnWhat::Error)
        } else if trimmed.starts_with("panic") {
            Ok(ReturnWhat::Panic)
        } else {
            Err(ParseReturnWhatError(str.to_string()))
        }
    }
}

/// Returns the parameters passed through environment variables
pub fn get_env_vars() -> EnvVars {
    EnvVars {
        return_what: std::env::var("RETURN").unwrap().parse().unwrap(),
    }
}
