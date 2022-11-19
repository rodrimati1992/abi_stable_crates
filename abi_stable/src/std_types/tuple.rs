//! Contains ffi-safe equivalents of tuples up to 4 elements.

#![allow(non_snake_case)]

macro_rules! declare_tuple {
(
    struct_attrs[ $(#[$meta: meta])* ]

    into_tuple_attrs[ $(#[$into_tuple_attrs: meta])* ]

    from_tuple_attrs[ $(#[$from_tuple_attrs: meta])* ]

    $tconstr: ident[$( $tparam: ident ),* $(,)? ]
) => (
    $(#[$meta])*
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, StableAbi)]
    #[repr(C)]
    pub struct $tconstr< $($tparam,)* > (
        $(pub $tparam,)*
    );

    impl_into_rust_repr! {
        impl[ $($tparam,)* ] Into<( $($tparam,)* )> for $tconstr< $($tparam,)* > {
            fn(this){
                let $tconstr($($tparam,)*) = this;
                ($($tparam,)*)
            }
        }
    }

    impl_from_rust_repr! {
        impl[ $($tparam,)* ] From<( $($tparam,)* )> for $tconstr< $($tparam,)* > {
            fn(this){
                let ($($tparam,)*) = this;
                $tconstr ( $($tparam),* )
            }
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    impl< $($tparam,)* > $tconstr<$($tparam,)*>{
        $(#[$into_tuple_attrs])*
        #[inline]
        pub fn into_tuple(self)-> ($($tparam,)*) {
            let $tconstr($($tparam,)*) = self;
            ($($tparam,)*)
        }
        $(#[$from_tuple_attrs])*
        #[inline]
        pub fn from_tuple(from: ($($tparam,)*))-> Self {
            let ($($tparam,)*) = from;
            Self($($tparam,)*)
        }
    }

)}

declare_tuple! {
    struct_attrs[
        /// An ffi safe 1 element tuple.
    ]

    into_tuple_attrs[
        /// Converts this Tuple1 to a unary tuple.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::std_types::*;
        ///
        /// assert_eq!(Tuple1(1).into_tuple(), (1,));
        ///
        /// ```
    ]

    from_tuple_attrs[
        /// Converts a unary tuple to a `Tuple1`.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::std_types::*;
        ///
        /// assert_eq!(Tuple1::from_tuple((1,)), Tuple1(1));
        ///
        /// ```
    ]

    Tuple1[
        A,
    ]
}

declare_tuple! {
    struct_attrs[
        /// An ffi safe 2 element tuple.
    ]

    into_tuple_attrs[
        /// Converts this Tuple2 to a pair.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::std_types::*;
        ///
        /// assert_eq!(Tuple2(1, 2).into_tuple(), (1, 2));
        ///
        /// ```
    ]

    from_tuple_attrs[
        /// Converts a pair to a `Tuple2`.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::std_types::*;
        ///
        /// assert_eq!(Tuple2::from_tuple((1, 2)), Tuple2(1, 2));
        ///
        /// ```
    ]

    Tuple2[
        A,
        B,
    ]
}

declare_tuple! {
    struct_attrs[
        /// An ffi safe 3 element tuple.
    ]

    into_tuple_attrs[
        /// Converts this Tuple3 to a 3-tuple.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::std_types::*;
        ///
        /// assert_eq!(Tuple3(1, 2, 3).into_tuple(), (1, 2, 3));
        ///
        /// ```
    ]

    from_tuple_attrs[
        /// Converts a 3-tuple to a `Tuple3`.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::std_types::*;
        ///
        /// assert_eq!(Tuple3::from_tuple((1, 2, 3)), Tuple3(1, 2, 3));
        ///
        /// ```
    ]

    Tuple3[
        A,
        B,
        C,
    ]
}

declare_tuple! {
    struct_attrs[
        /// An ffi safe 4 element tuple.
    ]

    into_tuple_attrs[
        /// Converts this Tuple4 to a 4-tuple.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::std_types::*;
        ///
        /// assert_eq!(Tuple4(1, 2, 3, 4).into_tuple(), (1, 2, 3, 4));
        ///
        /// ```
    ]

    from_tuple_attrs[
        /// Converts a 4-tuple to a `Tuple4`.
        ///
        /// # Example
        ///
        /// ```rust
        /// use abi_stable::std_types::*;
        ///
        /// assert_eq!(Tuple4::from_tuple((1, 2, 3, 4)), Tuple4(1, 2, 3, 4));
        ///
        /// ```
    ]

    Tuple4[
        A,
        B,
        C,
        D,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::unit_cmp)]
    fn value_macro() {
        assert_eq!(rtuple!(), ());
        assert_eq!(rtuple!(3), Tuple1(3));
        assert_eq!(rtuple!(3, 5), Tuple2(3, 5));
        assert_eq!(rtuple!(3, 5, 8), Tuple3(3, 5, 8));
        assert_eq!(rtuple!(3, 5, 8, 9), Tuple4(3, 5, 8, 9));
    }

    #[test]
    #[allow(clippy::let_unit_value)]
    fn type_macro() {
        let _: RTuple!() = ();
        let _: RTuple!(i32) = Tuple1(3);
        let _: RTuple!(i32, i32,) = Tuple2(3, 5);
        let _: RTuple!(i32, i32, u32,) = Tuple3(3, 5, 8);
        let _: RTuple!(i32, i32, u32, u32) = Tuple4(3, 5, 8, 9);
    }
}
