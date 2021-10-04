use crate::library::{
    ROOT_MODULE_LOADER_NAME, ROOT_MODULE_LOADER_NAME_NULSTR, ROOT_MODULE_LOADER_NAME_WITH_NUL,
};
use abi_stable_shared::mangled_root_module_loader_name;

#[test]
fn root_module_loader_name_test() {
    let name = mangled_root_module_loader_name();
    let with_nul = format!("{}\0", name);

    assert_eq!(ROOT_MODULE_LOADER_NAME, name);
    assert_eq!(ROOT_MODULE_LOADER_NAME_WITH_NUL, with_nul);
    assert_eq!(ROOT_MODULE_LOADER_NAME_NULSTR.to_str(), name);
    assert_eq!(ROOT_MODULE_LOADER_NAME_NULSTR.to_str_with_nul(), with_nul);
}
