macro_rules! declare_tuple {
($tconstr:ident[$( $tfield:ident : $tparam:ident),* $(,)? ]) => (
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash,StableAbi)]
    #[repr(C)]
    #[sabi(inside_abi_stable_crate)]
    pub struct $tconstr< $($tparam,)* > {
        $(pub $tfield: $tparam,)*
    }

    impl_into_rust_repr! {
        impl[ $($tparam,)* ] Into<( $($tparam,)* )> for $tconstr< $($tparam,)* > {
            fn(this){
                ($(this.$tfield,)*)
            }
        }
    }

    impl_from_rust_repr! {
        impl[ $($tparam,)* ] From<( $($tparam,)* )> for $tconstr< $($tparam,)* > {
            fn(this){
                let ($($tfield,)*)=this;
                $tconstr {
                    $($tfield,)*
                }
            }
        }
    }


)}

declare_tuple! {
    Tuple1[
        e0: A,
    ]
}

declare_tuple! {
    Tuple2[
        e0: A,
        e1: B,
    ]
}

declare_tuple! {
    Tuple3[
        e0: A,
        e1: B,
        e2: C,
    ]
}

declare_tuple! {
    Tuple4[
        e0: A,
        e1: B,
        e2: C,
        e3: D,
    ]
}
