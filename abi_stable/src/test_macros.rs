/// Passes the tokens through as well as defines a constant with the stringified tokens.
#[allow(unused_macros)]
macro_rules! and_stringify {
    (
        $vis:vis const $stringified_name:ident;
        $( $cosmos:tt )*
    ) => (
        $vis const $stringified_name:&'static str=stringify!($( $cosmos )*);

        $( $cosmos )*
    )
}

#[allow(unused_macros)]
macro_rules! assert_matches {
    ($value:expr, $pattern:pat $(if $cond:expr)?) => (
        match $value {
            x => {
                assert!(
                    ::std::matches!(x, $pattern $(if $cond)?),
                    "Expected ¨{}¨, found ¨{:?}¨",
                    stringify!($pattern $(if $cond)?),
                    x,
                );
            }
        }
    );
}
