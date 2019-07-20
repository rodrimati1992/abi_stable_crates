macro_rules! declare_constructors {
    ($foo:ident) => (
        pub fn new_a()->NonExhaustiveFor<$foo>{
            unsafe{
                std::mem::transmute(
                    super::example_3::Foo::A
                        .piped(NonExhaustive::new)
                )
            }
        }
        pub fn new_b(n:i8)->NonExhaustiveFor<$foo>{
            unsafe{
                std::mem::transmute(
                    super::example_3::Foo::B(n)
                        .piped(NonExhaustive::new)
                )
            }
        }

        pub fn new_c()->NonExhaustiveFor<$foo>{
            unsafe{
                std::mem::transmute(
                    super::example_3::Foo::C
                        .piped(NonExhaustive::new)
                )
            }
        }
    )
}

pub mod example_1{
    use core_extensions::SelfOps;
    use crate::nonexhaustive_enum::{NonExhaustive,NonExhaustiveFor};

    #[repr(u8)]
    #[derive(StableAbi,Debug,Clone,PartialEq)]
    #[sabi(kind(WithNonExhaustive(
        size="[usize;4]",
        traits(Debug,Clone,PartialEq)
    )))]
    //#[sabi(debug_print)]
    pub enum Foo{
        A,
    }

    declare_constructors!{Foo}
}


pub mod example_2{
    use core_extensions::SelfOps;
    use crate::nonexhaustive_enum::{NonExhaustive,NonExhaustiveFor};

    #[repr(u8)]
    #[derive(StableAbi,Debug,Clone,PartialEq)]
    #[sabi(kind(WithNonExhaustive(
        size="[usize;4]",
        traits(Debug,Clone,PartialEq)
    )))]
    pub enum Foo{
        A,
        B(i8),
    }

    declare_constructors!{Foo}
}


pub mod example_3{
    use core_extensions::SelfOps;
    use crate::nonexhaustive_enum::{NonExhaustive,NonExhaustiveFor};

    #[repr(u8)]
    #[derive(StableAbi,Debug,Clone,PartialEq)]
    #[sabi(kind(WithNonExhaustive(
        size="[usize;4]",
        traits(Debug,Clone,PartialEq)
    )))]
    pub enum Foo{
        A,
        B(i8),
        C,
    }

    declare_constructors!{Foo}
}



