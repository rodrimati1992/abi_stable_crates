/// Passes the tokens through as well as defines a constant with the stringified tokens.
macro_rules! and_stringify {
    (
        const $stringified_name:ident;
        $( $cosmos:tt )*
    ) => (
        pub const $stringified_name:&'static str=stringify!($( $cosmos )*);

        $( $cosmos )*
    )
}
