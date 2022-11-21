macro_rules! declare_constructors {
    ($foo:ident) => {
        pub fn new_a() -> NonExhaustiveFor<$foo> {
            // these transmutes are for testing compatibility of enums across versions
            unsafe { std::mem::transmute(super::example_3::Foo::A.piped(NonExhaustive::new)) }
        }
        pub fn new_b(n: i8) -> NonExhaustiveFor<$foo> {
            unsafe { std::mem::transmute(super::example_3::Foo::B(n).piped(NonExhaustive::new)) }
        }

        pub fn new_c() -> NonExhaustiveFor<$foo> {
            unsafe { std::mem::transmute(super::example_3::Foo::C.piped(NonExhaustive::new)) }
        }
    };
}

pub mod example_1 {
    use crate::nonexhaustive_enum::{NonExhaustive, NonExhaustiveFor};
    use core_extensions::SelfOps;

    #[repr(u8)]
    #[derive(StableAbi, Debug, Clone, PartialEq, Eq)]
    #[sabi(kind(WithNonExhaustive(size = [usize;4], traits(Debug, Clone, PartialEq))))]
    pub enum Foo {
        A,
    }

    declare_constructors! {Foo}
}

pub mod example_2 {
    use crate::nonexhaustive_enum::{NonExhaustive, NonExhaustiveFor};
    use core_extensions::SelfOps;

    #[repr(u8)]
    #[derive(StableAbi, Debug, Clone, PartialEq, Eq)]
    #[sabi(kind(WithNonExhaustive(size = [usize;4], traits(Debug, Clone, PartialEq))))]
    pub enum Foo {
        A,
        B(i8),
    }

    declare_constructors! {Foo}
}

pub mod example_3 {
    use crate::nonexhaustive_enum::{NonExhaustive, NonExhaustiveFor};
    use core_extensions::SelfOps;

    #[repr(u8)]
    #[derive(StableAbi, Debug, Clone, PartialEq, Eq)]
    #[sabi(kind(WithNonExhaustive(size = [usize;4], traits(Debug, Clone, PartialEq))))]
    pub enum Foo {
        A,
        B(i8),
        C,
    }

    declare_constructors! {Foo}
}
