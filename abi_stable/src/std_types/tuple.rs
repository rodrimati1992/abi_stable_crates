#![allow(non_snake_case)]

macro_rules! declare_tuple {
($tconstr:ident[$( $tparam:ident ),* $(,)? ]) => (
    /// Ffi-safe equivalent of tuples.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash,StableAbi)]
    #[repr(C)]
    #[sabi(inside_abi_stable_crate)]
    pub struct $tconstr< $($tparam,)* > (
        $(pub $tparam,)*
    );

    impl_into_rust_repr! {
        impl[ $($tparam,)* ] Into<( $($tparam,)* )> for $tconstr< $($tparam,)* > {
            fn(this){
                let $tconstr($($tparam,)*)=this;
                ($($tparam,)*)
            }
        }
    }

    impl_from_rust_repr! {
        impl[ $($tparam,)* ] From<( $($tparam,)* )> for $tconstr< $($tparam,)* > {
            fn(this){
                let ($($tparam,)*)=this;
                $tconstr ( $($tparam),* )
            }
        }
    }


)}

declare_tuple! {
    Tuple1[
        A,
    ]
}

declare_tuple! {
    Tuple2[
        A,
        B,
    ]
}

declare_tuple! {
    Tuple3[
        A,
        B,
        C,
    ]
}

declare_tuple! {
    Tuple4[
        A,
        B,
        C,
        D,
    ]
}
