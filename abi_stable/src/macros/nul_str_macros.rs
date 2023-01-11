/// Constructs a [`NulStr`] from a string literal,
/// truncating the string on internal nul bytes.
///
/// # Correctness
///
/// This truncates the passed in string if it contains nul bytes,
/// which means that silent truncation can happen with arbitrary inputs,
/// rather than compile-time errors.
///
/// # Example
///
/// ```rust
/// use abi_stable::nulstr_trunc;
///
/// assert_eq!(nulstr_trunc!("Huh?").to_str_with_nul(), "Huh?\0");
///
/// assert_eq!(nulstr_trunc!("Hello!").to_str_with_nul(), "Hello!\0");
///
/// assert_eq!(
///     nulstr_trunc!("Hello\0, world!").to_str_with_nul(),
///     "Hello\0"
/// );
///
/// ```
///
/// [`NulStr`]: ./sabi_types/struct.NulStr.html
#[macro_export]
macro_rules! nulstr_trunc {
    ($str:expr $(,)*) => {{
        const __STR_NHPMWYD3NJA: $crate::sabi_types::NulStr<'_> =
            $crate::sabi_types::NulStr::from_str($crate::pmr::concat!($str, "\0"));
        __STR_NHPMWYD3NJA
    }};
}

/// Constructs a [`NulStr`] from a string literal.
///
/// # Error
///
/// This causes a compile-time error if the input string contains nul byte(s).
///
/// # Example
///
/// ```rust
/// use abi_stable::nulstr;
///
/// assert_eq!( nulstr!("Huh?").to_str_with_nul(), "Huh?\0" );
/// assert_eq!( nulstr!("Hello!").to_str_with_nul(), "Hello!\0" );
/// ```
///
/// Nul bytes in the middle of the string cause compilation errors:
/// ```compile_fail
/// use abi_stable::nulstr;
///
/// assert_eq!(nulstr!("Hello\0, world!").to_str_with_nul(), "Hello\0");
/// ```
///
/// [`NulStr`]: ./sabi_types/struct.NulStr.html
#[macro_export]
macro_rules! nulstr {
    ($str:expr $(,)*) => {{
        const __STR_NHPMWYD3NJA: $crate::sabi_types::NulStr<'_> =
            $crate::sabi_types::NulStr::__try_from_str_unwrapping($crate::pmr::concat!($str, "\0"));

        __STR_NHPMWYD3NJA
    }};
}
