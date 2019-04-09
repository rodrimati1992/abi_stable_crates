/// Equivalent to `?` for `RResult<_,_>`.
#[macro_export]
macro_rules! rtry {
    ($expr:expr) => {{
        use $crate::result::{RErr, ROk};
        match $expr.into() {
            ROk(x) => x,
            RErr(x) => return RErr(From::from(x)),
        }
    }};
}

/// Equivalent to `?` for `ROption<_>`.
#[macro_export]
macro_rules! rtry_opt {
    ($expr:expr) => {{
        use $crate::option::{RNone, RSome};
        match $expr.into() {
            RSome(x) => x,
            RNone => return RNone,
        }
    }};
}

/**
Use this to make sure that you handle panics inside `extern fn` correctly.

This macro causes an abort if a panic reaches this point.

It does not prevent functions inside from using `::std::panic::catch_unwind` to catch the panic.

# Example 

```
use std::fmt;

use abi_stable::{
    extern_fn_panic_handling,
    std_types::RString,
};


pub extern "C" fn print_debug<T>(this: &T,buf: &mut RString)
where
    T: fmt::Debug,
{
    extern_fn_panic_handling! {
        use std::fmt::Write;

        println!("{:?}",this);
    }
}


```


*/
#[macro_export]
macro_rules! extern_fn_panic_handling {
    ( $($fn_contents:tt)* ) => ({
        use std::panic::{self,AssertUnwindSafe};

        let result=panic::catch_unwind(AssertUnwindSafe(move||{
            $($fn_contents)*
        }));

        match result {
            Ok(x)=>x,
            Err(_)=>$crate::utils::ffi_panic_message(file!(),line!()),
        }
    })
}
