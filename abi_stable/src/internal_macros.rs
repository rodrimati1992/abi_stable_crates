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
        $crate::rslice![
            $( 
                $crate::_sabi_type_layouts!(internal; $ty $( = $assoc_const )? ), 
            )*
        ]
    }};
}


/*
macro_rules! with_shared_attrs {
    (
        internal-0;
        $shared_attrs:tt;

        $((
            $( #[$before_shared:meta] )*;
            $( #[$after_shared:meta] )*;
            $($item:tt)*
        ))*
    ) => {
        $(
            with_shared_attrs!{
                internal-1;
                $shared_attrs;

                $( #[$before_shared] )*;
                $( #[$after_shared] )*;
                $($item)*
            }
        )*
    };
    (
        internal-1;
        ($(#[$shared_attrs:meta])*);

        $( #[$before_shared:meta] )*;
        $( #[$after_shared:meta] )*;
        $($item:tt)*
    ) => {
        $( #[$before_shared] )*
        $( #[$shared_attrs] )*
        $( #[$after_shared] )*
        $($item)*
    };
    (
        $(#[$shared_attrs:meta])*

        $((
            $( #[$before_shared:meta] )*;
            $( #[$after_shared:meta] )*;
            $($item:tt)*
        ))*
    ) => {
        with_shared_attrs!{
            internal-0;
            ($( #[$shared_attrs] )*);

            $((
                $( #[$before_shared] )*;
                $( #[$after_shared] )*;
                $($item)*
            ))*
        }
    };
}
*/