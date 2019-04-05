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
