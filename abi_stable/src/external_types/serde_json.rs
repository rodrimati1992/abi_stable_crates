use std::{
    convert::TryFrom,
    fmt::{self,Debug,Display},
    mem,
};

use serde::{Deserialize,Serialize,Deserializer,Serializer};
use serde_json::{
    error::Error as JsonError,
    value::RawValue,
};

use crate::std_types::{RStr,RString};


/// An ffi-safe equivalent of `&serde_json::value::RawValue`
#[repr(transparent)]
#[derive(StableAbi,Copy,Clone)]
pub struct RawValueRef<'a>{
    ref_:RStr<'a>,
}


impl<'a> RawValueRef<'a>{
/**
Converts a `&str` to a `RawValueRef<'a>` without checking whether it is valid JSON.

# Safety

`input` must be valid JSON and contain no leading or trailing whitespace.

*/
    pub unsafe fn from_str_unchecked(input:&'a str)->RawValueRef<'a>{
        Self{
            ref_:RStr::from(input),
        }
    }

/**
Converts a `RStr<'a>` to a `RawValueRef<'a>` without checking whether it is valid JSON.

# Safety

`input` must be valid JSON and contain no leading or trailing whitespace.

*/
    pub unsafe fn from_rstr_unchecked(input:RStr<'a>)->RawValueRef<'a>{
        Self{
            ref_:input,
        }
    }

    #[inline]
    pub fn get(&self)->&'a str{
        self.ref_.as_str()
    }

    #[inline]
    pub fn get_rstr(&self)->RStr<'a>{
        self.get().into()
    }
}


impl<'a> Debug for RawValueRef<'a>{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Debug::fmt(&self.ref_,f)
    }
}


impl<'a> Display for RawValueRef<'a>{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Display::fmt(&self.ref_,f)
    }
}


impl<'a> From<&'a RawValue> for RawValueRef<'a>{
    fn from(v:&'a RawValue)->Self{
        Self{
            ref_:v.get().into(),
        }
    }
}


impl<'a> TryFrom<&'a str> for RawValueRef<'a>{
    type Error=JsonError;
    fn try_from(v:&'a str)->Result<Self,JsonError>{
        serde_json::from_str::<&'a RawValue>(v)
            .map(Self::from)
    }
}


impl<'a> Serialize for RawValueRef<'a>{
    fn serialize<Z>(&self, serializer: Z) -> Result<Z::Ok, Z::Error>
    where
        Z: Serializer,
    {
        unsafe{
            into_ref_rawvalue(self.ref_.as_str()).serialize(serializer)
        }
    }
}


impl<'de: 'a, 'a> Deserialize<'de> for RawValueRef<'a>{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <&'a RawValue>::deserialize(deserializer)
            .map(Self::from)
    }
}

///////////////////////////////////////////////////////////////////////////////


/// An ffi-safe equivalent of `Box<serde_json::value::RawValue>`
#[repr(transparent)]
#[derive(StableAbi,Clone)]
pub struct RawValueBox{
    string:RString,
}


impl RawValueBox{
/**
Converts a `String` to an `RawValueBox` without checking whether it is valid JSON.

# Safety

`input` must be valid JSON and contain no leading or trailing whitespace.

*/
    pub unsafe fn from_string_unchecked(input:String)->RawValueBox{
        Self{
            string:input.into()
        }
    }

/**
Converts an `RString` to an `RawValueBox` without checking whether it is valid JSON.

# Safety

`input` must be valid JSON and contain no leading or trailing whitespace.

*/
    pub unsafe fn from_rstring_unchecked(input:RString)->RawValueBox{
        Self{
            string:input
        }
    }

    #[inline]
    pub fn get(&self)->&str{
        self.string.as_str()
    }
    
    #[inline]
    pub fn get_rstr(&self)->RStr<'_>{
        self.get().into()
    }
}


impl Debug for RawValueBox{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Debug::fmt(&self.string,f)
    }
}


impl Display for RawValueBox{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Display::fmt(&self.string,f)
    }
}


impl From<Box<RawValue>> for RawValueBox{
    fn from(v:Box<RawValue>)->Self{
        let string:String=from_boxed_rawvalue(v).into();
        Self{
            string:string.into()
        }
    }
}


impl TryFrom<String> for RawValueBox{
    type Error=JsonError;
    fn try_from(v:String)->Result<Self,JsonError>{
        RawValue::from_string(v)
            .map(Self::from)
    }
}


impl Serialize for RawValueBox{
    fn serialize<Z>(&self, serializer: Z) -> Result<Z::Ok, Z::Error>
    where
        Z: Serializer,
    {
        unsafe{
            into_ref_rawvalue(&self.string).serialize(serializer)
        }
    }
}


impl<'de> Deserialize<'de> for RawValueBox{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Box<RawValue>>::deserialize(deserializer)
            .map(Self::from)
    }
}


///////////////////////////////////////////////////////////////////////////////


fn from_boxed_rawvalue(x:Box<RawValue>)->Box<str>{
    // This would become Undefined Behavior in either of these cases:
    //
    // - serde_json somehow changes RawValue to be more than a newtype wrapper around `str`
    //
    // - transmuting from Box<ReprTransparentNewtype> to Box<str>
    //
    unsafe{ mem::transmute::<Box<RawValue>,Box<str>>(x) }
}


unsafe fn into_boxed_rawvalue(x:Box<str>)->Box<RawValue>{
    // This would become Undefined Behavior in either of these cases:
    //
    // - serde_json somehow changes RawValue to be more than a newtype wrapper around `str`
    //
    // - transmuting from Box<ReprTransparentNewtype> to Box<str>
    //
    mem::transmute::<Box<str>,Box<RawValue>>(x)
}


fn from_ref_rawvalue(x:&RawValue)->&str{
    x.get()
}


unsafe fn into_ref_rawvalue(x:&str)->&RawValue{
    // This would become Undefined Behavior in either of these cases:
    //
    // - serde_json somehow changes RawValue to be more than a newtype wrapper around `str`
    //
    // - transmuting from &ReprTransparentNewtype to &str
    //
    mem::transmute::<&str,&RawValue>(x)
}

