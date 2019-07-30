#![allow(non_snake_case)]

macro_rules! declare_tuple {
(
    struct_attrs[ $(#[$meta:meta])* ]
    
    into_tuple_attrs[ $(#[$into_tuple_attrs:meta])* ]

    $tconstr:ident[$( $tparam:ident ),* $(,)? ]
) => (
    $(#[$meta])*
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash,StableAbi)]
    #[repr(C)]
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

    impl< $($tparam,)* > $tconstr<$($tparam,)*>{
        $(#[$into_tuple_attrs])*
        #[inline]
        pub fn into_tuple(self)-> ($($tparam,)*) {
            self.into()
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
        /// ```
        /// use abi_stable::std_types::tuple::*;
        /// 
        /// assert_eq!( Tuple1(1).into_tuple(), (1,) );
        /// 
        /// ```
    ]
    
    Tuple1[
        A,
    ]
}

declare_tuple! {
    struct_attrs[
        /// An ffi safe 1 element tuple.
    ]

    into_tuple_attrs[
        /// Converts this Tuple2 to a pair.
        /// 
        /// # Example
        /// 
        /// ```
        /// use abi_stable::std_types::tuple::*;
        /// 
        /// assert_eq!( Tuple2(1,2).into_tuple(), (1,2) );
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
        /// An ffi safe 1 element tuple.
    ]

    into_tuple_attrs[
        /// Converts this Tuple3 to a 3-tuple.
        /// 
        /// # Example
        /// 
        /// ```
        /// use abi_stable::std_types::tuple::*;
        /// 
        /// assert_eq!( Tuple3(1,2,3).into_tuple(), (1,2,3) );
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
        /// An ffi safe 1 element tuple.
    ]

    into_tuple_attrs[
        /// Converts this Tuple4 to a 4-tuple.
        /// 
        /// # Example
        /// 
        /// ```
        /// use abi_stable::std_types::tuple::*;
        /// 
        /// assert_eq!( Tuple4(1,2,3,4).into_tuple(), (1,2,3,4) );
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
