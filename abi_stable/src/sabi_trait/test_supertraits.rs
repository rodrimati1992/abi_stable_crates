// This pub module only tests that the code inside compiles
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(dead_code)]

use std::{
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    error::Error as ErrorTrait,
    fmt::{self, Debug, Display, Write as FmtWriteTrait},
    io::{
        self, BufRead as IoBufReadTrait, Read as IoReadTrait, Seek as IoSeekTrait,
        Write as IoWriteTrait,
    },
};

use crate::{
    sabi_trait,
    std_types::RBox,
    test_utils::{GetImpls, GetImplsHelper},
    type_level::downcasting::{TD_CanDowncast, TD_Opaque},
};

pub mod no_supertraits {
    use super::*;

    #[sabi_trait]
    pub trait Trait {
        fn method(&self) {}
    }

    pub struct Struct;

    impl Trait for Struct {}

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_ERROR);
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
    }
}

pub mod static_supertrait {
    use super::*;

    #[sabi_trait]
    pub trait Trait: 'static {
        fn method(&self) {}
    }

    pub struct Struct<'a>(&'a str);

    impl Trait for Struct<'static> {}

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_ERROR);
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct(""), TD_CanDowncast);
        object.method();
    }

    // Testing that Trait has 'static as a supertrait,
    trait Dummy {
        fn dummy<T>()
        where
            T: Trait;
    }
    impl Dummy for () {
        fn dummy<T>()
        where
            T: Trait + 'static,
        {
        }
    }
}

pub mod nonstatic_supertrait {
    use super::*;

    #[sabi_trait]
    pub trait Trait<'a>: 'a {
        fn method(&self) {}
    }

    pub struct Struct<'a>(&'a str);

    impl<'a, 'b: 'a> Trait<'a> for Struct<'b> {}
    impl<'a, T: 'a> Trait<'a> for Option<T> {}

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, 'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_ERROR);
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct(""), TD_CanDowncast);
        object.method();
    }

    // Testing that Trait has 'a as a supertrait,
    trait Dummy {
        fn dummy<'a, T>()
        where
            T: Trait<'a>;
    }
    impl Dummy for () {
        fn dummy<'a, T>()
        where
            T: Trait<'a> + 'a,
        {
        }
    }

    struct MakeBorrowedCTO<'a, 'b, T>(&'b (), &'a (), T);

    impl<'a, 'b, T> MakeBorrowedCTO<'a, 'b, T>
    where
        T: 'a,
        'a: 'b,
    {
        pub const NONE: Option<&'a T> = None;

        pub const CONST: Trait_CTO<'a, 'a, 'b> = Trait_CTO::from_const(&Self::NONE, TD_Opaque);
        pub fn get_const(_: &'a T) -> Trait_CTO<'a, 'a, 'b> {
            Self::CONST
        }
    }

    // Testing that Trait does not have 'a as a supertrait.
    fn assert_trait_inner<'a, T>(_: T)
    where
        T: Trait<'a>,
    {
    }
    fn assert_trait() {
        let mut a = String::new();
        a.push('w');
        assert_trait_inner(Struct(&a));

        {
            let a = 0usize;
            MakeBorrowedCTO::get_const(&a).method();
        }
        {
            let a = String::new();
            MakeBorrowedCTO::get_const(&a).method();
        }
    }
}

pub mod only_clone {
    use super::*;

    #[sabi_trait]
    pub trait Trait: Clone {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_ERROR);
    }

    #[derive(Clone)]
    pub struct Struct;

    impl Trait for Struct {}

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        let _ = object.clone();
    }
}

pub mod only_display {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait: Display {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_ERROR);
    }

    pub struct Struct;

    impl Display for Struct {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            Display::fmt("What!?", f)
        }
    }

    impl Trait for Struct {}

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        format!("{}", object);
    }
}

pub mod only_debug {
    use super::*;

    #[sabi_trait]
    pub trait Trait: Debug {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_ERROR);
    }

    #[derive(Debug)]
    pub struct Struct;

    impl Trait for Struct {}

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        format!("{:?}", object);
    }
}

// pub mod only_serialize{
//     use super::*;

//     #[sabi_trait]
//     #[sabi(use_dyntrait)]
//     pub trait Trait:Serialize{
//         fn method(&self){}
//     }

//     #[derive(::serde::Serialize)]
//     pub struct Struct;

//     impl Trait for Struct{}

//     impl SerializeType for Struct {
//         fn serialize_impl<'a>(&'a self) -> Result<RCow<'a, str>, RBoxError>{
//             Ok(RCow::from("Struct"))
//         }
//     }

//     fn assert_bound<T>(_:&T)
//     where
//         T:serde::Serialize
//     {}

//     fn test_constructible(){
//         let object=Trait_TO::from_value(Struct,TD_CanDowncast);
//         object.method();
//         assert_bound(&object);
//     }
// }

// pub mod only_deserialize_a{
//     use super::*;
//     use serde::Deserialize;

//     #[sabi_trait]
//     #[sabi(use_dyntrait)]
//     //#[sabi(debug_print_trait)]
//     pub trait Trait: for<'a> Deserialize<'a> {
//         fn method(&self){}
//     }

//     impl<'a> DeserializeOwnedInterface<'a> for Trait_Interface{
//         type Deserialized=Trait_Backend<'a,RBox<()>>;

//         fn deserialize_impl(s: RStr<'_>) -> Result<Self::Deserialized, RBoxError>{
//             Ok(DynTrait::from_value(Struct,Trait_Interface::NEW))
//         }
//     }

//     #[derive(Deserialize)]
//     pub struct Struct;

//     impl Trait for Struct{}

//     fn assert_bound<T>(_:&T)
//     where
//         T:for<'a>Deserialize<'a>
//     {}

//     fn test_constructible(){
//         let object=Trait_TO::from_value(Struct,TD_CanDowncast);
//         object.method();
//         assert_bound(&object);
//     }
// }

pub mod only_partial_eq {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    // #[sabi(debug_print_trait)]
    pub trait Trait: PartialEq {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_SERIALIZE);
        assert!(!GI::IMPLS_EQ);
        assert!(GI::IMPLS_PARTIAL_EQ);
        assert!(!GI::IMPLS_ORD);
        assert!(!GI::IMPLS_PARTIAL_ORD);
        assert!(!GI::IMPLS_HASH);
        assert!(!GI::IMPLS_DESERIALIZE);
        assert!(!GI::IMPLS_ITERATOR);
        assert!(!GI::IMPLS_DOUBLE_ENDED_ITERATOR);
        assert!(!GI::IMPLS_FMT_WRITE);
        assert!(!GI::IMPLS_IO_WRITE);
        assert!(!GI::IMPLS_IO_SEEK);
        assert!(!GI::IMPLS_IO_READ);
        assert!(!GI::IMPLS_IO_BUF_READ);
        assert!(!GI::IMPLS_ERROR);
    }

    #[derive(PartialEq)]
    pub struct Struct;

    impl Trait for Struct {}

    fn assert_bound<T>(_: &T)
    where
        T: Trait + PartialEq,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_eq {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait: Eq {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_SERIALIZE);
        assert!(GI::IMPLS_EQ);
        assert!(GI::IMPLS_PARTIAL_EQ);
        assert!(!GI::IMPLS_ORD);
        assert!(!GI::IMPLS_PARTIAL_ORD);
        assert!(!GI::IMPLS_HASH);
        assert!(!GI::IMPLS_DESERIALIZE);
        assert!(!GI::IMPLS_ITERATOR);
        assert!(!GI::IMPLS_DOUBLE_ENDED_ITERATOR);
        assert!(!GI::IMPLS_FMT_WRITE);
        assert!(!GI::IMPLS_IO_WRITE);
        assert!(!GI::IMPLS_IO_SEEK);
        assert!(!GI::IMPLS_IO_READ);
        assert!(!GI::IMPLS_IO_BUF_READ);
        assert!(!GI::IMPLS_ERROR);
    }

    #[derive(Eq, PartialEq)]
    pub struct Struct;

    impl Trait for Struct {}

    fn assert_bound<T>(_: &T)
    where
        T: Trait + Eq,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_partial_ord {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait: PartialOrd {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_SERIALIZE);
        assert!(!GI::IMPLS_EQ);
        assert!(GI::IMPLS_PARTIAL_EQ);
        assert!(!GI::IMPLS_ORD);
        assert!(GI::IMPLS_PARTIAL_ORD);
        assert!(!GI::IMPLS_HASH);
        assert!(!GI::IMPLS_DESERIALIZE);
        assert!(!GI::IMPLS_ITERATOR);
        assert!(!GI::IMPLS_DOUBLE_ENDED_ITERATOR);
        assert!(!GI::IMPLS_FMT_WRITE);
        assert!(!GI::IMPLS_IO_WRITE);
        assert!(!GI::IMPLS_IO_SEEK);
        assert!(!GI::IMPLS_IO_READ);
        assert!(!GI::IMPLS_IO_BUF_READ);
        assert!(!GI::IMPLS_ERROR);
    }
    #[derive(PartialEq, PartialOrd)]
    pub struct Struct;

    impl Trait for Struct {}

    fn assert_bound<T>(_: &T)
    where
        T: Trait + PartialOrd,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_ord {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait: Ord {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_SERIALIZE);
        assert!(GI::IMPLS_EQ);
        assert!(GI::IMPLS_PARTIAL_EQ);
        assert!(GI::IMPLS_ORD);
        assert!(GI::IMPLS_PARTIAL_ORD);
        assert!(!GI::IMPLS_HASH);
        assert!(!GI::IMPLS_DESERIALIZE);
        assert!(!GI::IMPLS_ITERATOR);
        assert!(!GI::IMPLS_DOUBLE_ENDED_ITERATOR);
        assert!(!GI::IMPLS_FMT_WRITE);
        assert!(!GI::IMPLS_IO_WRITE);
        assert!(!GI::IMPLS_IO_SEEK);
        assert!(!GI::IMPLS_IO_READ);
        assert!(!GI::IMPLS_IO_BUF_READ);
        assert!(!GI::IMPLS_ERROR);
    }

    #[derive(PartialEq, PartialOrd, Eq, Ord)]
    pub struct Struct;

    impl Trait for Struct {}

    fn assert_bound<T>(_: &T)
    where
        T: Trait + Ord,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_hash {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait: Hash {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_SERIALIZE);
        assert!(!GI::IMPLS_EQ);
        assert!(!GI::IMPLS_PARTIAL_EQ);
        assert!(!GI::IMPLS_ORD);
        assert!(!GI::IMPLS_PARTIAL_ORD);
        assert!(GI::IMPLS_HASH);
        assert!(!GI::IMPLS_DESERIALIZE);
        assert!(!GI::IMPLS_ITERATOR);
        assert!(!GI::IMPLS_DOUBLE_ENDED_ITERATOR);
        assert!(!GI::IMPLS_FMT_WRITE);
        assert!(!GI::IMPLS_IO_WRITE);
        assert!(!GI::IMPLS_IO_SEEK);
        assert!(!GI::IMPLS_IO_READ);
        assert!(!GI::IMPLS_IO_BUF_READ);
        assert!(!GI::IMPLS_ERROR);
    }

    #[derive(Hash)]
    pub struct Struct;

    impl Trait for Struct {}

    fn assert_bound<T>(_: &T)
    where
        T: Trait + std::hash::Hash,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_iterator_a {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait<'a, T: 'a>: Iterator<Item = &'a T> {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, 'static, RBox<()>, ()>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_SERIALIZE);
        assert!(!GI::IMPLS_EQ);
        assert!(!GI::IMPLS_PARTIAL_EQ);
        assert!(!GI::IMPLS_ORD);
        assert!(!GI::IMPLS_PARTIAL_ORD);
        assert!(!GI::IMPLS_HASH);
        assert!(!GI::IMPLS_DESERIALIZE);
        assert!(GI::IMPLS_ITERATOR);
        assert!(!GI::IMPLS_DOUBLE_ENDED_ITERATOR);
        assert!(!GI::IMPLS_FMT_WRITE);
        assert!(!GI::IMPLS_IO_WRITE);
        assert!(!GI::IMPLS_IO_SEEK);
        assert!(!GI::IMPLS_IO_READ);
        assert!(!GI::IMPLS_IO_BUF_READ);
        assert!(!GI::IMPLS_ERROR);
    }

    pub struct Struct<'a, T>(&'a T);

    impl<'a, T> Trait<'a, T> for Struct<'a, T> {}

    impl<'a, T> Iterator for Struct<'a, T> {
        type Item = &'a T;
        fn next(&mut self) -> Option<&'a T> {
            None
        }
    }

    fn assert_bound<'a, T: 'a>(_: &T)
    where
        T: Trait<'a, i32> + Iterator<Item = &'a i32>,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct(&0), TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_iterator_b {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait<T: 'static>: Iterator<Item = &'static T> {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>, ()>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_SERIALIZE);
        assert!(!GI::IMPLS_EQ);
        assert!(!GI::IMPLS_PARTIAL_EQ);
        assert!(!GI::IMPLS_ORD);
        assert!(!GI::IMPLS_PARTIAL_ORD);
        assert!(!GI::IMPLS_HASH);
        assert!(!GI::IMPLS_DESERIALIZE);
        assert!(GI::IMPLS_ITERATOR);
        assert!(!GI::IMPLS_DOUBLE_ENDED_ITERATOR);
        assert!(!GI::IMPLS_FMT_WRITE);
        assert!(!GI::IMPLS_IO_WRITE);
        assert!(!GI::IMPLS_IO_SEEK);
        assert!(!GI::IMPLS_IO_READ);
        assert!(!GI::IMPLS_IO_BUF_READ);
        assert!(!GI::IMPLS_ERROR);
    }

    pub struct Struct;

    impl Trait<i32> for Struct {}

    impl Iterator for Struct {
        type Item = &'static i32;
        fn next(&mut self) -> Option<&'static i32> {
            None
        }
    }

    fn assert_bound<T>(_: &T)
    where
        T: Trait<i32> + Iterator<Item = &'static i32>,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_de_iterator {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait<T: 'static>: DoubleEndedIterator<Item = &'static T> {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>, ()>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_SERIALIZE);
        assert!(!GI::IMPLS_EQ);
        assert!(!GI::IMPLS_PARTIAL_EQ);
        assert!(!GI::IMPLS_ORD);
        assert!(!GI::IMPLS_PARTIAL_ORD);
        assert!(!GI::IMPLS_HASH);
        assert!(!GI::IMPLS_DESERIALIZE);
        assert!(GI::IMPLS_ITERATOR);
        assert!(GI::IMPLS_DOUBLE_ENDED_ITERATOR);
        assert!(!GI::IMPLS_FMT_WRITE);
        assert!(!GI::IMPLS_IO_WRITE);
        assert!(!GI::IMPLS_IO_SEEK);
        assert!(!GI::IMPLS_IO_READ);
        assert!(!GI::IMPLS_IO_BUF_READ);
        assert!(!GI::IMPLS_ERROR);
    }

    pub struct Struct;

    impl Trait<i32> for Struct {}

    impl Iterator for Struct {
        type Item = &'static i32;
        fn next(&mut self) -> Option<&'static i32> {
            None
        }
    }

    impl DoubleEndedIterator for Struct {
        fn next_back(&mut self) -> Option<&'static i32> {
            None
        }
    }

    fn assert_bound<T>(_: &T)
    where
        T: Trait<i32> + DoubleEndedIterator<Item = &'static i32>,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_error {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait: Error {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(GI::IMPLS_DISPLAY);
        assert!(GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_SERIALIZE);
        assert!(!GI::IMPLS_EQ);
        assert!(!GI::IMPLS_PARTIAL_EQ);
        assert!(!GI::IMPLS_ORD);
        assert!(!GI::IMPLS_PARTIAL_ORD);
        assert!(!GI::IMPLS_HASH);
        assert!(!GI::IMPLS_DESERIALIZE);
        assert!(!GI::IMPLS_ITERATOR);
        assert!(!GI::IMPLS_DOUBLE_ENDED_ITERATOR);
        assert!(!GI::IMPLS_FMT_WRITE);
        assert!(!GI::IMPLS_IO_WRITE);
        assert!(!GI::IMPLS_IO_SEEK);
        assert!(!GI::IMPLS_IO_READ);
        assert!(!GI::IMPLS_IO_BUF_READ);
        assert!(GI::IMPLS_ERROR);
    }

    #[derive(Debug)]
    pub struct Struct;

    impl Display for Struct {
        fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
            Ok(())
        }
    }

    impl ErrorTrait for Struct {}

    impl Trait for Struct {}

    fn assert_bound<T>(_: &T)
    where
        T: Trait + ErrorTrait,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_fmt_write {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait: FmtWrite {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_SERIALIZE);
        assert!(!GI::IMPLS_EQ);
        assert!(!GI::IMPLS_PARTIAL_EQ);
        assert!(!GI::IMPLS_ORD);
        assert!(!GI::IMPLS_PARTIAL_ORD);
        assert!(!GI::IMPLS_HASH);
        assert!(!GI::IMPLS_DESERIALIZE);
        assert!(!GI::IMPLS_ITERATOR);
        assert!(!GI::IMPLS_DOUBLE_ENDED_ITERATOR);
        assert!(GI::IMPLS_FMT_WRITE);
        assert!(!GI::IMPLS_IO_WRITE);
        assert!(!GI::IMPLS_IO_SEEK);
        assert!(!GI::IMPLS_IO_READ);
        assert!(!GI::IMPLS_IO_BUF_READ);
        assert!(!GI::IMPLS_ERROR);
    }

    pub struct Struct;

    impl FmtWriteTrait for Struct {
        fn write_str(&mut self, _: &str) -> Result<(), fmt::Error> {
            Ok(())
        }
    }

    impl Trait for Struct {}

    fn assert_bound<T>(_: &T)
    where
        T: Trait + FmtWriteTrait,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_io_write {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait: IoWrite {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_SERIALIZE);
        assert!(!GI::IMPLS_EQ);
        assert!(!GI::IMPLS_PARTIAL_EQ);
        assert!(!GI::IMPLS_ORD);
        assert!(!GI::IMPLS_PARTIAL_ORD);
        assert!(!GI::IMPLS_HASH);
        assert!(!GI::IMPLS_DESERIALIZE);
        assert!(!GI::IMPLS_ITERATOR);
        assert!(!GI::IMPLS_DOUBLE_ENDED_ITERATOR);
        assert!(!GI::IMPLS_FMT_WRITE);
        assert!(GI::IMPLS_IO_WRITE);
        assert!(!GI::IMPLS_IO_SEEK);
        assert!(!GI::IMPLS_IO_READ);
        assert!(!GI::IMPLS_IO_BUF_READ);
        assert!(!GI::IMPLS_ERROR);
    }

    pub struct Struct;

    impl IoWriteTrait for Struct {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Ok(0)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Trait for Struct {}

    fn assert_bound<T>(_: &T)
    where
        T: Trait + IoWriteTrait,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_io_read {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait: IoRead {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_SERIALIZE);
        assert!(!GI::IMPLS_EQ);
        assert!(!GI::IMPLS_PARTIAL_EQ);
        assert!(!GI::IMPLS_ORD);
        assert!(!GI::IMPLS_PARTIAL_ORD);
        assert!(!GI::IMPLS_HASH);
        assert!(!GI::IMPLS_DESERIALIZE);
        assert!(!GI::IMPLS_ITERATOR);
        assert!(!GI::IMPLS_DOUBLE_ENDED_ITERATOR);
        assert!(!GI::IMPLS_FMT_WRITE);
        assert!(!GI::IMPLS_IO_WRITE);
        assert!(!GI::IMPLS_IO_SEEK);
        assert!(GI::IMPLS_IO_READ);
        assert!(!GI::IMPLS_IO_BUF_READ);
        assert!(!GI::IMPLS_ERROR);
    }

    pub struct Struct;

    impl IoReadTrait for Struct {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Ok(0)
        }
    }

    impl Trait for Struct {}

    fn assert_bound<T>(_: &T)
    where
        T: Trait + IoReadTrait,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_io_bufread {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait: IoBufRead {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_SERIALIZE);
        assert!(!GI::IMPLS_EQ);
        assert!(!GI::IMPLS_PARTIAL_EQ);
        assert!(!GI::IMPLS_ORD);
        assert!(!GI::IMPLS_PARTIAL_ORD);
        assert!(!GI::IMPLS_HASH);
        assert!(!GI::IMPLS_DESERIALIZE);
        assert!(!GI::IMPLS_ITERATOR);
        assert!(!GI::IMPLS_DOUBLE_ENDED_ITERATOR);
        assert!(!GI::IMPLS_FMT_WRITE);
        assert!(!GI::IMPLS_IO_WRITE);
        assert!(!GI::IMPLS_IO_SEEK);
        assert!(GI::IMPLS_IO_READ);
        assert!(GI::IMPLS_IO_BUF_READ);
        assert!(!GI::IMPLS_ERROR);
    }

    pub struct Struct;

    impl IoReadTrait for Struct {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Ok(0)
        }
    }

    impl IoBufReadTrait for Struct {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Ok(&[])
        }
        fn consume(&mut self, _amt: usize) {}
    }

    impl Trait for Struct {}

    fn assert_bound<T>(_: &T)
    where
        T: Trait + IoBufReadTrait,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_io_seek {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait: IoSeek {
        fn method(&self) {}
    }

    #[test]
    fn test_impls() {
        type GI = GetImpls<Trait_TO<'static, RBox<()>>>;
        assert!(!GI::IMPLS_SEND);
        assert!(!GI::IMPLS_SYNC);
        assert!(!GI::IMPLS_UNPIN);
        assert!(!GI::IMPLS_CLONE);
        assert!(!GI::IMPLS_DISPLAY);
        assert!(!GI::IMPLS_DEBUG);
        assert!(!GI::IMPLS_SERIALIZE);
        assert!(!GI::IMPLS_EQ);
        assert!(!GI::IMPLS_PARTIAL_EQ);
        assert!(!GI::IMPLS_ORD);
        assert!(!GI::IMPLS_PARTIAL_ORD);
        assert!(!GI::IMPLS_HASH);
        assert!(!GI::IMPLS_DESERIALIZE);
        assert!(!GI::IMPLS_ITERATOR);
        assert!(!GI::IMPLS_DOUBLE_ENDED_ITERATOR);
        assert!(!GI::IMPLS_FMT_WRITE);
        assert!(!GI::IMPLS_IO_WRITE);
        assert!(GI::IMPLS_IO_SEEK);
        assert!(!GI::IMPLS_IO_READ);
        assert!(!GI::IMPLS_IO_BUF_READ);
        assert!(!GI::IMPLS_ERROR);
    }

    pub struct Struct;

    impl IoSeekTrait for Struct {
        fn seek(&mut self, _: io::SeekFrom) -> io::Result<u64> {
            Ok(0)
        }
    }

    impl Trait for Struct {}

    fn assert_bound<T>(_: &T)
    where
        T: Trait + IoSeekTrait,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

pub mod every_trait {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:
        Clone
        + Iterator
        + DoubleEndedIterator<Item = &'static ()>
        + Display
        + Debug
        + Error
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + FmtWrite
        + IoWrite
        + IoRead
        + IoBufRead
        + IoSeek
        + Send
        + Sync
        + Unpin
    {
        fn method(&self) {}
    }
    #[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
    pub struct Struct;

    impl Display for Struct {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            Display::fmt("What!?", f)
        }
    }

    impl Iterator for Struct {
        type Item = &'static ();
        fn next(&mut self) -> Option<&'static ()> {
            None
        }
    }

    impl DoubleEndedIterator for Struct {
        fn next_back(&mut self) -> Option<&'static ()> {
            None
        }
    }

    impl ErrorTrait for Struct {}

    impl FmtWriteTrait for Struct {
        fn write_str(&mut self, _: &str) -> Result<(), fmt::Error> {
            Ok(())
        }
    }

    impl IoWriteTrait for Struct {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Ok(0)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl IoReadTrait for Struct {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Ok(0)
        }
    }

    impl IoBufReadTrait for Struct {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Ok(&[])
        }
        fn consume(&mut self, _amt: usize) {}
    }

    impl IoSeekTrait for Struct {
        fn seek(&mut self, _: io::SeekFrom) -> io::Result<u64> {
            Ok(0)
        }
    }

    impl Trait for Struct {}

    fn assert_bound<T>(_: &T)
    where
        T: Trait
            + Clone
            + Iterator
            + DoubleEndedIterator<Item = &'static ()>
            + Display
            + Debug
            + ErrorTrait
            + PartialEq
            + Eq
            + PartialOrd
            + Ord
            + std::hash::Hash
            + FmtWriteTrait
            + IoWriteTrait
            + IoReadTrait
            + IoBufReadTrait
            + IoSeekTrait
            + Send
            + Sync
            + Unpin,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

pub mod every_trait_static {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:
        'static
        + Clone
        + Iterator
        + DoubleEndedIterator<Item = &'static ()>
        + Display
        + Debug
        + Error
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
        + FmtWrite
        + IoWrite
        + IoRead
        + IoBufRead
        + IoSeek
    {
        fn method(&self) {}
    }
    #[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
    pub struct Struct;

    impl Display for Struct {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            Display::fmt("What!?", f)
        }
    }

    impl Iterator for Struct {
        type Item = &'static ();
        fn next(&mut self) -> Option<&'static ()> {
            None
        }
    }

    impl DoubleEndedIterator for Struct {
        fn next_back(&mut self) -> Option<&'static ()> {
            None
        }
    }

    impl ErrorTrait for Struct {}

    impl FmtWriteTrait for Struct {
        fn write_str(&mut self, _: &str) -> Result<(), fmt::Error> {
            Ok(())
        }
    }

    impl IoWriteTrait for Struct {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Ok(0)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl IoReadTrait for Struct {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Ok(0)
        }
    }

    impl IoBufReadTrait for Struct {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Ok(&[])
        }
        fn consume(&mut self, _amt: usize) {}
    }

    impl IoSeekTrait for Struct {
        fn seek(&mut self, _: io::SeekFrom) -> io::Result<u64> {
            Ok(0)
        }
    }

    impl Trait for Struct {}

    fn assert_bound<T>(_: &T)
    where
        T: Trait
            + Clone
            + Iterator
            + DoubleEndedIterator<Item = &'static ()>
            + Display
            + Debug
            + ErrorTrait
            + PartialEq
            + Eq
            + PartialOrd
            + Ord
            + std::hash::Hash
            + FmtWriteTrait
            + IoWriteTrait
            + IoReadTrait
            + IoBufReadTrait
            + IoSeekTrait,
    {
    }

    fn test_constructible() {
        let object = Trait_TO::from_value(Struct, TD_CanDowncast);
        object.method();
        assert_bound(&object);
    }
}

////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

pub mod every_trait_nonstatic {
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait<'a>:
        'a+
        Clone+
        Iterator+DoubleEndedIterator<Item=&'static ()>+
        Display+Debug+Error+
        //PartialEq+Eq+PartialOrd+Ord+
        Hash+
        FmtWrite+IoWrite+IoRead+IoBufRead+IoSeek
    {
        fn method(&self){}
    }
    #[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
    pub struct Struct<'a>(&'a str);

    impl Display for Struct<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            Display::fmt("What!?", f)
        }
    }

    impl Iterator for Struct<'_> {
        type Item = &'static ();
        fn next(&mut self) -> Option<&'static ()> {
            None
        }
    }

    impl DoubleEndedIterator for Struct<'_> {
        fn next_back(&mut self) -> Option<&'static ()> {
            None
        }
    }

    impl ErrorTrait for Struct<'_> {}

    impl FmtWriteTrait for Struct<'_> {
        fn write_str(&mut self, _: &str) -> Result<(), fmt::Error> {
            Ok(())
        }
    }

    impl IoWriteTrait for Struct<'_> {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Ok(0)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl IoReadTrait for Struct<'_> {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Ok(0)
        }
    }

    impl IoBufReadTrait for Struct<'_> {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Ok(&[])
        }
        fn consume(&mut self, _amt: usize) {}
    }

    impl IoSeekTrait for Struct<'_> {
        fn seek(&mut self, _: io::SeekFrom) -> io::Result<u64> {
            Ok(0)
        }
    }

    impl<'a> Trait<'a> for Struct<'a> {}

    fn assert_bound<'a, T>(_: &T)
    where
        T: Trait<'a>,
    {
    }

    fn test_constructible() {
        let string = String::new();
        let value = Struct(&string);
        let object = Trait_TO::from_ptr(RBox::new(value), TD_Opaque);
        object.method();
        assert_bound(&object);

        {
            let value = Struct(&string);
            constructs_const_a(&value);

            let value = Struct("");
            constructs_const_a(&value);
        }
    }

    const CONST_A: Trait_CTO<'static, 'static, 'static> =
        Trait_CTO::from_const(&Struct(""), TD_Opaque);

    fn constructs_const_a<'a, 'b, 'borr, T>(ref_: &'b T) -> Trait_CTO<'a, 'borr, 'b>
    where
        T: 'borr + 'a + Trait<'a>,
    {
        Trait_CTO::from_const(ref_, TD_Opaque)
    }
}
