pub(super) mod single_integer {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    // #[sabi(debug_print)]
    pub struct Struct<const A: usize>;
}

pub(super) mod two_integer {
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Struct<const A: usize, const B: usize>;
}

pub(super) mod single_integer_one_phantom {
    use abi_stable::{const_utils::AssocStr, marker_type::UnsafeIgnoredType};

    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    #[sabi(bound(T: AssocStr), phantom_const_param = T::STR)]
    pub struct Struct<T, const A: usize>(UnsafeIgnoredType<T>);
}
