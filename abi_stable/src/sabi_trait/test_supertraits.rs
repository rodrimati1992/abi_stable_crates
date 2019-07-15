// This pub module only tests that the code inside compiles
#![allow(dead_code)] 

use std::{
    cmp::{Eq,PartialEq,Ord,PartialOrd},
    error::Error as ErrorTrait,
    fmt::{self,Display,Debug,Write as FmtWriteTrait},
    io::{
        self,
        Write as IoWriteTrait,
        Read as IoReadTrait,
        BufRead as IoBufReadTrait,
        Seek as IoSeekTrait,
    },
};



use crate::{
    sabi_trait,
    erased_types::{DeserializeOwnedInterface,SerializeImplType},
    type_level::unerasability::{TU_Unerasable,TU_Opaque},
    std_types::{RCow,RBox,RBoxError,RStr},
    DynTrait,
};


pub mod no_supertraits{
    use super::*;

    #[sabi_trait]
    pub trait Trait{
        fn method(&self){}
    }

    pub struct Struct;

    impl Trait for Struct{}

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
    }
}

pub mod only_clone{
    use super::*;

    #[sabi_trait]
    pub trait Trait:Clone{
        fn method(&self){}
    }

    #[derive(Clone)]
    pub struct Struct;

    impl Trait for Struct{}

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        let _=object.clone();
    }
}

pub mod only_display{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:Display{
        fn method(&self){}
    }

    pub struct Struct;

    impl Display for Struct{
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            Display::fmt("What!?",f)
        }
    }

    impl Trait for Struct{}

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        format!("{}",object);
    }
}

pub mod only_debug{
    use super::*;

    #[sabi_trait]
    pub trait Trait:Debug{
        fn method(&self){}
    }

    #[derive(Debug)]
    pub struct Struct;

    impl Trait for Struct{}

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        format!("{:?}",object);
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

//     impl SerializeImplType for Struct {
//         fn serialize_impl<'a>(&'a self) -> Result<RCow<'a, str>, RBoxError>{
//             Ok(RCow::from("Struct"))
//         }
//     }

//     fn assert_bound<T>(_:&T)
//     where
//         T:serde::Serialize
//     {}

//     fn test_constructible(){
//         let object=Trait_TO::from_value(Struct,TU_Unerasable);
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
//             Ok(DynTrait::from_any_value(Struct,Trait_Interface::NEW))
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
//         let object=Trait_TO::from_value(Struct,TU_Unerasable);
//         object.method();
//         assert_bound(&object);
//     }
// }

pub mod only_partial_eq{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:PartialEq{
        fn method(&self){}
    }

    #[derive(PartialEq)]
    pub struct Struct;

    impl Trait for Struct{}

    fn assert_bound<T>(_:&T)
    where
        T:Trait+PartialEq
    {}    

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_eq{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:Eq{
        fn method(&self){}
    }

    #[derive(Eq,PartialEq)]
    pub struct Struct;

    impl Trait for Struct{}

    fn assert_bound<T>(_:&T)
    where
        T:Trait+Eq
    {}    

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_partial_ord{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:PartialOrd{
        fn method(&self){}
    }

    #[derive(PartialEq,PartialOrd)]
    pub struct Struct;

    impl Trait for Struct{}

    fn assert_bound<T>(_:&T)
    where
        T:Trait+PartialOrd
    {}    

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_ord{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:Ord{
        fn method(&self){}
    }

    #[derive(PartialEq,PartialOrd,Eq,Ord)]
    pub struct Struct;

    impl Trait for Struct{}

    fn assert_bound<T>(_:&T)
    where
        T:Trait+Ord
    {}    

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_hash{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:Hash{
        fn method(&self){}
    }

    #[derive(Hash)]
    pub struct Struct;

    impl Trait for Struct{}

    fn assert_bound<T>(_:&T)
    where
        T:Trait+std::hash::Hash
    {}    

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_iterator_a{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait<'a,T:'a>:Iterator<Item=&'a T>{
        fn method(&self){}
    }

    pub struct Struct<'a,T>(&'a T);

    impl<'a,T> Trait<'a,T> for Struct<'a,T>{}

    impl<'a,T> Iterator for Struct<'a,T>{
        type Item=&'a T;
        fn next(&mut self)->Option<&'a T>{
            None
        }
    }

    fn assert_bound<'a,T:'a>(_:&T)
    where
        T:Trait<'a,i32>+Iterator<Item=&'a i32>
    {}    

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct(&0),TU_Unerasable);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_iterator_b{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait<T:'static>:Iterator<Item=&'static T>{
        fn method(&self){}
    }

    pub struct Struct;

    impl Trait<i32> for Struct{}

    impl Iterator for Struct{
        type Item=&'static i32;
        fn next(&mut self)->Option<&'static i32>{
            None
        }
    }

    fn assert_bound<T>(_:&T)
    where
        T:Trait<i32>+Iterator<Item=&'static i32>
    {}    

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_de_iterator{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait<T:'static>:DoubleEndedIterator<Item=&'static T>{
        fn method(&self){}
    }

    pub struct Struct;

    impl Trait<i32> for Struct{}

    impl Iterator for Struct{
        type Item=&'static i32;
        fn next(&mut self)->Option<&'static i32>{
            None
        }
    }

    impl DoubleEndedIterator for Struct{
        fn next_back(&mut self)->Option<&'static i32>{
            None
        }
    }

    fn assert_bound<T>(_:&T)
    where
        T:Trait<i32>+DoubleEndedIterator<Item=&'static i32>
    {}    

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        assert_bound(&object);
    }
}


pub mod only_error{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:Error{
        fn method(&self){}
    }

    #[derive(Debug)]
    pub struct Struct;

    impl Display for Struct{
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            Ok(())
        }
    }

    impl ErrorTrait for Struct{}

    impl Trait for Struct{}

    fn assert_bound<T>(_:&T)
    where
        T:Trait+ErrorTrait
    {}    

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_fmt_write{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:FmtWrite{
        fn method(&self){}
    }

    pub struct Struct;

    impl FmtWriteTrait for Struct{
        fn write_str(&mut self, s: &str) -> Result<(), fmt::Error>{
            Ok(())
        }
    }

    impl Trait for Struct{}

    fn assert_bound<T>(_:&T)
    where
        T:Trait+FmtWriteTrait
    {}    

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_io_write{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:IoWrite{
        fn method(&self){}
    }

    pub struct Struct;

    impl IoWriteTrait for Struct{
        fn write(&mut self, buf: &[u8]) -> io::Result<usize>{
            Ok(0)
        }
        fn flush(&mut self) -> io::Result<()>{
            Ok(())
        }
    }

    impl Trait for Struct{}

    fn assert_bound<T>(_:&T)
    where
        T:Trait+IoWriteTrait
    {}    

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_io_read{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:IoRead{
        fn method(&self){}
    }

    pub struct Struct;

    impl IoReadTrait for Struct{
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>{
            Ok(0)
        }
    }

    impl Trait for Struct{}

    fn assert_bound<T>(_:&T)
    where
        T:Trait+IoReadTrait
    {}    

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_io_bufread{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:IoBufRead{
        fn method(&self){}
    }

    pub struct Struct;

    impl IoReadTrait for Struct{
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>{
            Ok(0)
        }
    }

    impl IoBufReadTrait for Struct{    
        fn fill_buf(&mut self) -> io::Result<&[u8]>{
            Ok(&[])
        }
        fn consume(&mut self, amt: usize){}

    }

    impl Trait for Struct{}

    fn assert_bound<T>(_:&T)
    where
        T:Trait+IoBufReadTrait
    {}    

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        assert_bound(&object);
    }
}

pub mod only_io_seek{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:IoSeek{
        fn method(&self){}
    }

    pub struct Struct;

    impl IoSeekTrait for Struct{    
        fn seek(&mut self, _: io::SeekFrom) -> io::Result<u64>{
            Ok(0)
        }
    }

    impl Trait for Struct{}

    fn assert_bound<T>(_:&T)
    where
        T:Trait+IoSeekTrait
    {}    

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        assert_bound(&object);
    }
}

pub mod every_trait{
    use super::*;

    #[sabi_trait]
    #[sabi(use_dyntrait)]
    pub trait Trait:
        Clone+
        Iterator+DoubleEndedIterator<Item=&'static ()>+
        Display+Debug+Error+
        PartialEq+Eq+PartialOrd+Ord+
        Hash+
        FmtWrite+IoWrite+IoRead+IoBufRead+IoSeek
    {
        fn method(&self){}
    }
    #[derive(Debug,Clone,PartialEq,PartialOrd,Eq,Ord,Hash)]
    pub struct Struct;

    impl Display for Struct{
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
            Display::fmt("What!?",f)
        }
    }

    impl Iterator for Struct{
        type Item=&'static ();
        fn next(&mut self)->Option<&'static ()>{
            None
        }
    }

    impl DoubleEndedIterator for Struct{
        fn next_back(&mut self)->Option<&'static ()>{
            None
        }
    }

    impl ErrorTrait for Struct{}

    impl FmtWriteTrait for Struct{
        fn write_str(&mut self, s: &str) -> Result<(), fmt::Error>{
            Ok(())
        }
    }

    impl IoWriteTrait for Struct{
        fn write(&mut self, buf: &[u8]) -> io::Result<usize>{
            Ok(0)
        }
        fn flush(&mut self) -> io::Result<()>{
            Ok(())
        }
    }

    impl IoReadTrait for Struct{
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>{
            Ok(0)
        }
    }

    impl IoBufReadTrait for Struct{    
        fn fill_buf(&mut self) -> io::Result<&[u8]>{
            Ok(&[])
        }
        fn consume(&mut self, amt: usize){}
    }

    impl IoSeekTrait for Struct{    
        fn seek(&mut self, _: io::SeekFrom) -> io::Result<u64>{
            Ok(0)
        }
    }

    impl Trait for Struct{}

    fn assert_bound<T>(_:&T)
    where
        T:Trait+
            Clone+
            Iterator+DoubleEndedIterator<Item=&'static ()>+
            Display+Debug+ErrorTrait+
            PartialEq+Eq+PartialOrd+Ord+
            std::hash::Hash+
            FmtWriteTrait+IoWriteTrait+IoReadTrait+IoBufReadTrait+IoSeekTrait
    {}

    fn test_constructible(){
        let object=Trait_TO::from_value(Struct,TU_Unerasable);
        object.method();
        assert_bound(&object);
    }

}