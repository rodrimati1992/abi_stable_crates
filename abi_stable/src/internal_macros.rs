#[doc(hidden)]
#[macro_export]
macro_rules! _sabi_type_layouts {
    (internal; $ty:ty )=>{{
        $crate::pmr::get_type_layout::<$ty>
    }};
    (internal; $ty:ty = SABI_OPAQUE_FIELD)=>{
        $crate::pmr::__sabi_opaque_field_type_layout::<$ty>
    };
    (internal; $ty:ty = OPAQUE_FIELD)=>{
        $crate::pmr::__opaque_field_type_layout::<$ty>
    };
    (
        $( $ty:ty $( = $assoc_const:ident )? ,)*
    ) => {{
        $crate::rslice![
            $(
                $crate::_sabi_type_layouts!(internal; $ty $( = $assoc_const )? ),
            )*
        ]
    }};
}
