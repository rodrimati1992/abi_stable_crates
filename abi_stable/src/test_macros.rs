/// Passes the tokens through as well as defines a constant with the stringified tokens.
macro_rules! and_stringify {
    (
        $vis:vis const $stringified_name:ident;
        $( $cosmos:tt )*
    ) => (
        $vis const $stringified_name:&'static str=stringify!($( $cosmos )*);

        $( $cosmos )*
    )
}
