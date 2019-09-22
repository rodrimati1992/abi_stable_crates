#[doc(hidden)]
#[macro_export]
macro_rules! _sabi_type_layouts {
    (internal; $ty:ty )=>{
        __GetTypeLayoutCtor::<$ty>::STABLE_ABI
    };
    (internal; $ty:ty=$assoc_const:ident )=>{
        __GetTypeLayoutCtor::<$ty>::$assoc_const
    };
    (
        $( $ty:ty $( = $assoc_const:ident )? ,)*
    ) => {{        
        use ::abi_stable::derive_macro_reexports::renamed::__GetTypeLayoutCtor;
        
        $crate::rslice![
            $( 
                $crate::_sabi_type_layouts!(internal; $ty $( = $assoc_const )? ), 
            )*
        ]
    }};
}