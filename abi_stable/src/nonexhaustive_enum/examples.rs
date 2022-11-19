//! Example non-exhaustive enums,used in tests

#![allow(dead_code, missing_docs)]
#![allow(clippy::derive_partial_eq_without_eq)]

pub mod command_one {
    use std::fmt::{self, Display};

    #[repr(u8)]
    #[derive(StableAbi, Hash, Debug, PartialEq, Eq, Clone)]
    #[sabi(kind(WithNonExhaustive(
        size = 64,
        traits(Debug, PartialEq, Eq, Clone),
        assert_nonexhaustive = Foo,
    )))]
    pub enum Foo {
        A,
    }

    impl Display for Foo {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }
}

pub mod command_one_more_traits_1 {
    use std::fmt::{self, Display};

    #[repr(u8)]
    #[derive(StableAbi, Debug, PartialEq, Eq, Clone, Hash)]
    #[sabi(kind(WithNonExhaustive(
        size = 64,
        traits(Debug, PartialEq, Eq, Clone, Hash),
        assert_nonexhaustive(Foo),
    )))]
    pub enum Foo {
        A,
    }

    impl Display for Foo {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }
}

pub mod command_one_more_traits_2 {
    use std::fmt::{self, Display};

    #[repr(u8)]
    #[derive(StableAbi, Debug, PartialEq, Eq, PartialOrd, Clone, Hash)]
    #[sabi(kind(WithNonExhaustive(
        size = 64,
        traits(Debug, PartialEq, Eq, PartialOrd, Clone, Hash)
    )))]
    pub enum Foo {
        A,
    }

    impl Display for Foo {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }
}

pub mod command_one_more_traits_3 {
    use std::fmt::{self, Display};

    #[repr(u8)]
    #[derive(StableAbi, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
    #[sabi(kind(WithNonExhaustive(
        size = 64,
        traits(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)
    )))]
    pub enum Foo {
        A,
    }

    impl Display for Foo {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }
}

pub mod command_a {
    use std::fmt::{self, Display};

    #[repr(u8)]
    #[derive(StableAbi, Hash, Debug, PartialEq, Eq, Clone)]
    #[sabi(kind(WithNonExhaustive(size = 64, traits(Debug, PartialEq, Eq, Clone))))]
    pub enum Foo {
        A,
        B(i8),
    }

    impl Display for Foo {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }
}

pub mod command_a_exhaustive {
    use std::fmt::{self, Display};

    #[repr(u8)]
    #[derive(StableAbi, Hash, Debug, PartialEq, Eq, Clone)]
    pub enum Foo {
        A,
        B(i8),
    }

    impl Display for Foo {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }
}

pub mod command_b {
    use std::fmt::{self, Display};

    #[repr(u8)]
    #[derive(StableAbi, Hash, Debug, PartialEq, Eq, Clone)]
    #[sabi(kind(WithNonExhaustive(size = 64, traits(Debug, PartialEq, Eq, Clone))))]
    pub enum Foo {
        A,
        B(i8),
        C,
    }

    impl Display for Foo {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }
}

pub mod command_c {
    use crate::std_types::RString;

    #[repr(u8)]
    #[derive(StableAbi, Hash, Debug, PartialEq, Eq, Clone)]
    #[sabi(kind(WithNonExhaustive(size = 64, traits(Debug, PartialEq, Eq, Clone))))]
    pub enum Foo {
        A,
        B(i8),
        C,
        D { name: RString },
    }
}

pub mod command_c_mismatched_field {
    use crate::std_types::RVec;

    #[repr(u8)]
    #[derive(StableAbi, Hash, Debug, PartialEq, Eq, Clone)]
    #[sabi(kind(WithNonExhaustive(size = 64, traits(Debug, PartialEq, Eq, Clone))))]
    pub enum Foo {
        A,
        B(i8),
        C,
        D { name: RVec<u8> },
    }
}

pub mod command_serde {
    use std::fmt::{self, Display};

    use serde::{Deserialize, Serialize};

    use crate::std_types::RString;

    #[repr(u8)]
    #[derive(
        StableAbi, Hash, Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Deserialize, Serialize,
    )]
    #[sabi(kind(WithNonExhaustive(
        size = 64,
        traits(Debug, Display, PartialEq, Eq, Clone, Deserialize, Serialize)
    )))]
    // #[sabi(debug_print)]
    pub enum Foo {
        A,
        B(i8),
        C,
        D { name: RString },
    }

    impl Display for Foo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Foo::A => write!(f, "Variant A"),
                Foo::B(v) => write!(f, "Variant B with value:{}", v),
                Foo::C => write!(f, "Variant C"),
                Foo::D { name } => write!(f, "Variant D named:{}", name),
            }
        }
    }

    delegate_interface_serde! {
        impl[T,] Traits<T> for Foo_Interface;
        lifetime='borr;
        delegate_to=super::codecs::Json;
    }
}

pub mod too_large {
    #[repr(u8)]
    #[derive(StableAbi, Hash, Debug, PartialEq, Eq, Clone)]
    #[sabi(kind(WithNonExhaustive(size = 64, traits(Debug, PartialEq, Eq, Clone))))]
    pub enum Foo<T = i8> {
        A,
        B(T),
        C([u16; 32]),
    }
}

pub mod generic_a {
    use std::fmt::{self, Display};

    #[repr(u8)]
    #[derive(StableAbi, Hash, Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
    #[sabi(kind(WithNonExhaustive(
        size = 64,
        traits(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Hash)
    )))]
    //#[sabi(debug_print)]
    pub enum Foo<T> {
        A,
        B,
        C(T),
    }

    impl<T> Display for Foo<T>
    where
        T: Display,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Foo::A => write!(f, "Variant A"),
                Foo::B => write!(f, "Variant B"),
                Foo::C(v) => write!(f, "Variant C:{}", v),
            }
        }
    }
}

pub mod generic_b {
    #[repr(u8)]
    #[derive(StableAbi, Debug, PartialEq)]
    #[sabi(kind(WithNonExhaustive(size = 64, align = 8, traits(Debug, PartialEq))))]
    pub enum Foo<T> {
        A,
        B,
        C(T),
    }
}

pub mod many_ranges_a {
    #[repr(u8)]
    #[derive(StableAbi, Debug, PartialEq, Eq, Clone)]
    #[sabi(kind(WithNonExhaustive(size = 64, traits(Debug, PartialEq, Eq, Clone))))]
    pub enum Foo {
        A,
        B = 40,
        C,
        D,
        E = 60,
        F,
    }
}

pub mod many_ranges_b {
    #[repr(u8)]
    #[derive(StableAbi, Debug, PartialEq, Eq, Clone)]
    #[sabi(kind(WithNonExhaustive(size = 64, traits(Debug, PartialEq, Eq, Clone))))]
    pub enum Foo {
        A,
        B = 40,
        C,
        E = 60,
    }
}

pub mod command_h {
    use std::fmt::{self, Display};

    #[repr(u8)]
    #[derive(StableAbi, Hash, Debug, PartialEq, Eq, Clone)]
    #[sabi(kind(WithNonExhaustive(size = 64, traits(Debug, PartialEq, Eq, Clone))))]
    pub enum Foo {
        A,
        B,
        C,
    }

    impl Display for Foo {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }
}

pub mod command_h_mismatched_discriminant {
    use std::fmt::{self, Display};

    #[repr(u8)]
    #[derive(StableAbi, Hash, Debug, PartialEq, Eq, Clone)]
    #[sabi(kind(WithNonExhaustive(size = 64, traits(Debug, PartialEq, Eq, Clone))))]
    pub enum Foo {
        A = 40,
        B,
        C,
    }

    impl Display for Foo {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }
}

pub mod const_expr_size_align {
    use std::fmt::{self, Display};

    const fn size() -> usize {
        10
    }
    const fn align() -> usize {
        2
    }

    #[repr(u8)]
    #[derive(StableAbi, Debug, PartialEq)]
    #[sabi(kind(WithNonExhaustive(
        size = { size() },
        align = { align() },
        traits(Debug, PartialEq)
    )))]
    pub enum Foo<T> {
        A,
        B,
        C(T),
    }

    impl<T> Display for Foo<T> {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }
}

pub mod codecs {
    use serde::{Deserialize, Serialize};

    use crate::{
        nonexhaustive_enum::{
            DeserializeEnum, GetEnumInfo, GetVTable, NonExhaustive, SerializeEnum,
        },
        std_types::{RBoxError, RString},
    };

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(impl_InterfaceType())]
    pub struct Json;

    impl<E> SerializeEnum<E> for Json
    where
        E: Serialize + GetEnumInfo,
    {
        type Proxy = RString;

        fn serialize_enum(this: &E) -> Result<RString, RBoxError> {
            serde_json::to_string(this)
                .map(RString::from)
                .map_err(RBoxError::new)
        }
    }

    impl<'borr, E, S, I> DeserializeEnum<'borr, NonExhaustive<E, S, I>> for Json
    where
        E: GetVTable<S, I> + for<'de> Deserialize<'de>,
    {
        type Proxy = RString;

        fn deserialize_enum(s: RString) -> Result<NonExhaustive<E, S, I>, RBoxError> {
            serde_json::from_str(&s)
                .map(NonExhaustive::with_storage_and_interface)
                .map_err(RBoxError::new)
        }
    }
}
