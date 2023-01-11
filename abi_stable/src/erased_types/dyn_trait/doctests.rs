macro_rules! test_case {
    ($test_kind:ident, $interface:expr, $ctor:ident, $e:expr) => {
        concat!(
            "```",
            stringify!($test_kind),
            "\n",
            stringify!(
                use abi_stable::{
                    erased_types::interfaces::*,
                    std_types::*,
                    DynTrait,
                };
                let _ = DynTrait::$ctor($e).interface($interface);
            ),
            "\n```",
        )
    };
}

macro_rules! fail_unpin {
    ($($args:tt)*) => {
        test_case!(compile_fail, UnpinInterface, $($args)*)
    };
}

#[doc = test_case!(rust, UnpinInterface, from_value, ())]
#[doc = test_case!(rust, UnpinInterface, from_ptr, RBox::new(()))]
#[doc = test_case!(rust, UnpinInterface, from_ptr, RArc::new(()))]
#[doc = fail_unpin!(from_value, std::marker::PhantomPinned)]
#[doc = fail_unpin!(from_ptr, RBox::new(std::marker::PhantomPinned))]
#[doc = fail_unpin!(from_ptr, RArc::new(std::marker::PhantomPinned))]
pub struct UnpinConstructible;
