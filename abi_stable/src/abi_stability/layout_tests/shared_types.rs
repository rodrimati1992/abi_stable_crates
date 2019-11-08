#![allow(dead_code)]

use std::marker::PhantomData;

#[allow(unused_imports)]
use core_extensions::{matches, prelude::*};

#[allow(unused_imports)]
use crate::{
    external_types::{
        RMutex,RRwLock,ROnce
    },
    std_types::*,
};


pub(super) mod basic_enum {
    #[repr(C)]
    #[derive(StableAbi)]
    pub enum Enum {
        Variant0,
        Variant1 { a: u32 },
    }
}

pub(super) mod gen_basic {
    use super::PhantomData;
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct Generics<T: 'static> {
        x: &'static T,
        y: &'static T,
        _marker: PhantomData<T>,
    }
}

pub(super) mod gen_more_lts {
    use super::PhantomData;
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(bound = "T:'a")]
    pub struct Generics<'a, T> {
        x: &'a T,
        y: &'a T,
        _marker: PhantomData<&'a T>,
    }
}

pub(super) mod enum_extra_fields_b {
    #[repr(C)]
    #[derive(StableAbi)]
    pub enum Enum {
        Variant0,
        Variant1 { a: u32,b:u32,c:u32 },
    }
}

pub(super) mod extra_variant {
    use crate::std_types::RString;
    #[repr(C)]
    #[derive(StableAbi)]
    pub enum Enum {
        Variant0,
        Variant1 { a: u32 },
        Variant3(RString),
    }
}


pub(super) mod swapped_fields_first {
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct Rectangle {
        y: u32,
        x: u32,
        w: u16,
        h: u32,
    }
}


pub(super) mod gen_more_lts_b {
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct Generics<'a> {
        x: &'a (),
        y: &'static (),
    }
}


pub(super) mod mod_5 {
    use super::RString;
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct Mod{
        pub function_0: extern "C" fn()->RString,
        pub function_1: extern "C" fn(&mut u32,u64,RString),
        pub function_2: extern "C" fn(&mut u32,u64,RString),
    }
}


pub(super) mod mod_7 {
    use super::RString;
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct Mod{
        pub function_0: extern "C" fn()->RString,
        pub function_1: extern "C" fn(&mut u32,u64,RString),
        pub function_2: extern "C" fn((),(),()),
    }
}