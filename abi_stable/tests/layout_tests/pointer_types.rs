// Types to test
use abi_stable::{
    for_examples::{Module_Ref, Module_Prefix},
    prefix_type::PrefixRef,
    sabi_types::{Constructor, MovePtr, RRef, NulStr, StaticRef},
};

use abi_stable::{
    reexports::True,
    StableAbi,
};

use std::ptr::NonNull;

const PTR_SIZE: usize = std::mem::size_of::<*const ()>();
const PTR_ALIGN: usize = std::mem::align_of::<*const ()>();

fn test_case<T>()
where
    T: StableAbi<IsNonZeroType = True>,
    Option<T>: StableAbi,
{
    let name = std::any::type_name::<T>();

    assert_eq!(std::mem::size_of::<T>(), PTR_SIZE, "{}", name);
    assert_eq!(std::mem::size_of::<Option<T>>(), PTR_SIZE, "{}", name);

    assert_eq!(std::mem::align_of::<T>(), PTR_ALIGN, "{}", name);
    assert_eq!(std::mem::align_of::<Option<T>>(), PTR_ALIGN, "{}", name);
}


#[test]
fn test_nonnullable() {
    test_case::<Module_Ref>();
    
    test_case::<PrefixRef<Module_Prefix>>();
    
    test_case::<Constructor<u8>>();
    test_case::<MovePtr<'_,u8>>();
    test_case::<RRef<'_,u8>>();
    test_case::<NulStr<'_>>();
    test_case::<StaticRef<u8>>();

    test_case::<NonNull<u8>>();
}