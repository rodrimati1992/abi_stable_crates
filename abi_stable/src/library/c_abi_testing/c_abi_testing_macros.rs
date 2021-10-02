macro_rules! check_roundtrip {
    (
        $funcs:ident,
        $initial_int:expr,
        ($($ret_val:ident=$composite:expr),* $(,)*),

        $ret_fn:ident,
        $take_fn:ident
        $(,)*
    ) => {
        #[allow(unused_parens)]
        {
        let res=($funcs.$ret_fn)($initial_int);
        let composite=($($composite),*);

        if res!=composite {
            return Err(make_invalid_cabi_err(composite.clone(),res.clone()));
        }
        let ($($ret_val),*)=res.clone();
        let int=($funcs.$take_fn)($($ret_val),*);
        if int!=$initial_int {
            return Err(make_invalid_cabi_err(
                (composite.clone(),$initial_int),
                (res.clone(),int),
            ));
        }
    }}
}

macro_rules! anon_struct {
    (
        $($fname:ident : $fval:expr),*
        $(,)*
    ) => {{
        #[allow(non_camel_case_types)]
        struct Anonymous<$($fname),*>{
            $($fname:$fname,)*
        }
        Anonymous{
            $($fname:$fval,)*
        }
    }};
}
