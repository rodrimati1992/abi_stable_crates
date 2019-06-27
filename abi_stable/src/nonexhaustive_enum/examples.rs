/*!
Example non-exhaustive enums,used in tests
*/


pub(crate) mod command_one{
    use std::fmt::{self,Display};

    #[repr(u8)]
    #[derive(StableAbi,Hash,Debug,PartialEq,Eq,Clone)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,PartialEq,Eq,Clone),
        assert_nonexhaustive="Foo",
    )))]
    pub(crate) enum Foo{
        A,
    }

    impl Display for Foo{
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            Ok(())
        }
    }
}

pub(crate) mod command_one_more_traits_1{
    use std::fmt::{self,Display};

    #[repr(u8)]
    #[derive(StableAbi,Debug,PartialEq,Eq,Clone,Hash)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,PartialEq,Eq,Clone,Hash),
        assert_nonexhaustive("Foo"),
    )))]
    pub(crate) enum Foo{
        A,
    }

    impl Display for Foo{
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            Ok(())
        }
    }
}

pub(crate) mod command_one_more_traits_2{
    use std::fmt::{self,Display};

    #[repr(u8)]
    #[derive(StableAbi,Debug,PartialEq,Eq,PartialOrd,Clone,Hash)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,PartialEq,Eq,PartialOrd,Clone,Hash)
    )))]
    pub(crate) enum Foo{
        A,
    }

    impl Display for Foo{
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            Ok(())
        }
    }
}

pub(crate) mod command_one_more_traits_3{
    use std::fmt::{self,Display};

    #[repr(u8)]
    #[derive(StableAbi,Debug,PartialEq,Eq,PartialOrd,Ord,Clone,Hash)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,PartialEq,Eq,PartialOrd,Ord,Clone,Hash)
    )))]
    pub(crate) enum Foo{
        A,
    }

    impl Display for Foo{
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            Ok(())
        }
    }
}

pub(crate) mod command_a{
    use std::fmt::{self,Display};

    #[repr(u8)]
    #[derive(StableAbi,Hash,Debug,PartialEq,Eq,Clone)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,PartialEq,Eq,Clone)
    )))]
    pub(crate) enum Foo{
        A,
        B(i8),
    }

    impl Display for Foo{
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            Ok(())
        }
    }
}

pub(crate) mod command_a_exhaustive{
    use std::fmt::{self,Display};

    #[repr(u8)]
    #[derive(StableAbi,Hash,Debug,PartialEq,Eq,Clone)]
    pub(crate) enum Foo{
        A,
        B(i8),
    }

    impl Display for Foo{
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            Ok(())
        }
    }
}

pub(crate) mod command_b{
    use std::fmt::{self,Display};

    #[repr(u8)]
    #[derive(StableAbi,Hash,Debug,PartialEq,Eq,Clone)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,PartialEq,Eq,Clone)
    )))]
    pub(crate) enum Foo{
        A,
        B(i8),
        C,
    }

    impl Display for Foo{
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            Ok(())
        }
    }
}

pub(crate) mod command_c{
    use std::fmt::{self,Display};
    
    use crate::std_types::RString;

    #[repr(u8)]
    #[derive(StableAbi,Hash,Debug,PartialEq,Eq,Clone)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,PartialEq,Eq,Clone)
    )))]
    pub(crate) enum Foo{
        A,
        B(i8),
        C,
        D{
            name:RString,
        },
    }

}

pub(crate) mod command_c_mismatched_field{
    use std::fmt::{self,Display};

    use crate::std_types::RVec;

    #[repr(u8)]
    #[derive(StableAbi,Hash,Debug,PartialEq,Eq,Clone)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,PartialEq,Eq,Clone)
    )))]
    pub(crate) enum Foo{
        A,
        B(i8),
        C,
        D{
            name:RVec<u8>,
        },
    }

}

pub(crate) mod command_serde{
    use std::fmt::{self,Display};

    use serde::{Serialize,Deserialize};

    use crate::{
        nonexhaustive_enum::{
            SerializeEnum,DeserializeOwned,DeserializeBorrowed,NonExhaustiveFor,NonExhaustive,
            GetEnumInfo,GetVTable,
        },
        std_types::RString,
        type_level::bools::{True},
    };

    #[repr(u8)]
    #[derive(StableAbi,Hash,Debug,PartialEq,Eq,Ord,PartialOrd,Clone,Deserialize,Serialize)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,Display,PartialEq,Eq,Clone,Deserialize,Serialize)
    )))]
    pub(crate) enum Foo{
        A,
        B(i8),
        C,
        D{
            name:RString,
        },
    }

    impl Display for Foo{
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            match self {
                Foo::A=>write!(f,"Variant A"),
                Foo::B(v)=>write!(f,"Variant B with value:{}",v),
                Foo::C=>write!(f,"Variant C"),
                Foo::D{name}=>write!(f,"Variant D named:{}",name),
            }
        }
    }


    delegate_interface_serde!{
        impl[T,] Traits<T> for Foo_Interface;
        lifetime='borr;
        delegate_to=super::codecs::Json;
    }
}

pub(crate) mod too_large{
    #[repr(u8)]
    #[derive(StableAbi,Hash,Debug,PartialEq,Eq,Clone)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,PartialEq,Eq,Clone)
    )))]
    pub(crate) enum Foo{
        Variant([u16;32])
    }
}


pub(crate) mod generic_a{
    use std::fmt::{self,Display};

    #[repr(u8)]
    #[derive(StableAbi,Hash,Debug,PartialOrd,Ord,PartialEq,Eq,Clone)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,PartialEq,Eq,Ord,PartialOrd,Clone,Hash)
    )))]
    pub(crate) enum Foo<T>{
        A,
        B,
        C(T),
    }


    impl<T> Display for Foo<T>
    where
        T:Display
    {
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            match self {
                Foo::A=>write!(f,"Variant A"),
                Foo::B=>write!(f,"Variant B"),
                Foo::C(v)=>write!(f,"Variant C:{}",v),
            }
        }
    }
}


pub(crate) mod many_ranges_a{
    use std::fmt::{self,Display};

    #[repr(u8)]
    #[derive(StableAbi,Debug,PartialEq,Eq,Clone)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,PartialEq,Eq,Clone)
    )))]
    pub(crate) enum Foo{
        A,
        B=40,
        C,
        D,
        E=60,
        F,
    }
}

pub(crate) mod many_ranges_b{
    use std::fmt::{self,Display};

    #[repr(u8)]
    #[derive(StableAbi,Debug,PartialEq,Eq,Clone)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,PartialEq,Eq,Clone)
    )))]
    pub(crate) enum Foo{
        A,
        B=40,
        C,
        E=60,
    }

}


pub(crate) mod command_h{
    use std::fmt::{self,Display};

    #[repr(u8)]
    #[derive(StableAbi,Hash,Debug,PartialEq,Eq,Clone)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,PartialEq,Eq,Clone)
    )))]
    pub(crate) enum Foo{
        A,
        B,
        C,
    }

    impl Display for Foo{
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            Ok(())
        }
    }
}


pub(crate) mod command_h_mismatched_discriminant{
    use std::fmt::{self,Display};

    #[repr(u8)]
    #[derive(StableAbi,Hash,Debug,PartialEq,Eq,Clone)]
    #[sabi(kind(WithNonExhaustive(
        size=64,
        traits(Debug,PartialEq,Eq,Clone)
    )))]
    pub(crate) enum Foo{
        A=40,
        B,
        C,
    }

    impl Display for Foo{
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            Ok(())
        }
    }
}





pub(crate) mod codecs{
    use serde::{Serialize,Deserialize};

    use crate::{
        nonexhaustive_enum::{
            SerializeEnum,DeserializeOwned,DeserializeBorrowed,NonExhaustiveFor,NonExhaustive,
            GetEnumInfo,GetVTable,
        },
        std_types::{RBoxError,RStr,RCow},
        type_level::bools::{True},
    };   

    pub struct Json;

    impl<E> SerializeEnum<E> for Json
    where
        E:Serialize
    {
        fn serialize_enum<'a>(this:&'a E) -> Result<RCow<'a, str>, RBoxError>{
            serde_json::to_string(this)
                .map(RCow::from)
                .map_err(RBoxError::new)
        }
    }

    impl<E,S,I> DeserializeOwned<E,S,I> for Json
    where
        E:GetVTable<S,I>+for<'de>Deserialize<'de>,
    {
        fn deserialize_enum(s: RStr<'_>) -> Result<NonExhaustive<E,S,I>, RBoxError>{
            serde_json::from_str(&*s)
                .map(NonExhaustive::with_storage_and_interface)
                .map_err(RBoxError::new)
        }
    }

    impl<'borr,E,S,I> DeserializeBorrowed<'borr,E,S,I> for Json
    where
        E:GetVTable<S,I>+Deserialize<'borr>+'borr,
    {
        fn deserialize_enum(s: RStr<'borr>) -> Result<NonExhaustive<E,S,I>, RBoxError> {
            serde_json::from_str(s.into())
                .map(NonExhaustive::with_storage_and_interface)
                .map_err(RBoxError::new)
        }
    }
}