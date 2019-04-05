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


#[macro_export]
macro_rules! rstr_helper {
    (coerce_lit;$s:literal) => {{
        mod string_length_module {
            $crate::_priv_get_string_length! {$s}
        }
        use $crate::RStr;
        RStr::<'static>::_private_from_raw_parts($s.as_ptr(), string_length_module::LENGTH)
    }};
}

#[macro_export]
macro_rules! rslice {
    ( $( $elem:expr ),* $(,)? ) => ({
        let slic_:&'static [_]=&[$( $elem ),*];
        $crate::RSlice::_private_from_raw_parts(
            slic_.as_ptr(),
            rslice!(count_elements;;; $( $elem ),* ),
        )
    });
    ( $ty:ty; $repeated:expr )=>({
        let arr:&[$ty]=&[$ty;$repeated];
        $crate::RSlice::_private_from_raw_parts(
            slic_,
            $repeated,
        )
    });
    (count_elements;;;)=>{0};
    (count_elements;;;$e0:expr)=>{1};
    (count_elements;;;$e0:expr,$e1:expr)=>{2};
    (count_elements;;;$e0:expr,$e1:expr,$e2:expr)=>{3};
    (count_elements;;;$e0:expr,$e1:expr,$e2:expr,$e3:expr)=>{4};
    (count_elements;;;$e0:expr,$e1:expr,$e2:expr,$e3:expr,$e4:expr)=>{5};
    (count_elements;;;
        $e0:expr,$e1:expr,$e2:expr,$e3:expr,$e4:expr,$e5:expr
    )=>{
        6
    };
    (count_elements;;;
        $e0:expr,$e1:expr,$e2:expr,$e3:expr,$e4:expr,$e5:expr,$e6:expr
    )=>{
        7
    };
    (count_elements;;;
        $e0:expr,$e1:expr,$e2:expr,$e3:expr,$e4:expr,$e5:expr,$e6:expr,$e7:expr
    )=>{
        8
    };
    (count_elements;;;
        $e0:expr,$e1:expr,$e2:expr,$e3:expr,$e4:expr,$e5:expr,$e6:expr,$e7:expr,
        $($rest:expr),*
    )=>{
        8+rslice!(count_elements;;;$($rest),*)
    };
}

