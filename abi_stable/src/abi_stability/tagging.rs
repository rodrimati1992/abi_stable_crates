use std::{
    collections::BTreeMap,
    fmt::{self,Debug,Display},
    mem,
};


use core_extensions::{
    matches,
    strings::LeftPadder,
    prelude::*,
};

use crate::{
    StableAbi,
    std_types::{
        StaticStr,
        StaticSlice,
        RVec,
        ROption,
        RSome,
        RNone,
    },
    traits::IntoReprC,
    utils::FmtPadding,
};



/**
A dynamically types data structure used to encode extra properties 
about a type in its layout constant.

Some usecases for this:

- Encoding the traits that the interface in VirtualWrapper requires.

- Check the marker traits implemented by any type,using specialization.

# Comparison semantics

Tags don't use strict equality when doing layout checking ,
here is an exhaustive list on what is considered compatible 
for each variant **as an interface**:

- Null:
    A Tag which is compatible with any other one.
    Note that Nulls are stripped from arrays,set,and maps.

- Integers/bools/strings:
    They must be strictly equal.

- Arrays:
    They must have the same length,with each element defining its compatibility .

- Sets/Maps:
    The set/map in the interface must be a subset of the implementation,
    with each element defining its compatibility .

# Example

Declaring each variant.

```rust
use abi_stable::{
    tag,
    abi_stability::Tag,
};

const NULL:Tag=Tag::null();


const BOOL_MACRO:Tag=tag!( false );
const BOOL_FN   :Tag=Tag::bool_(false);


const INT_MACRO_0:Tag=tag!(  100 );
const INT_FN_0   :Tag=Tag::int(100);

const INT_MACRO_1:Tag=tag!( -100 );
const INT_FN_1   :Tag=Tag::int(-100);


// This can only be declared using the function for now.
const UINT:Tag=Tag::uint(100);


const STR_0_MACRO:Tag=tag!("Hello,World!");
const STR_0_FN:Tag=Tag::str("Hello,World!");

const ARR_0_MACRO:Tag=tag![[ 0,1,2,3 ]];
const ARR_0_FN:Tag=Tag::arr(&[
    Tag::int(0),
    Tag::int(1),
    Tag::int(2),
    Tag::int(3),
]);


const SET_0_MACRO:Tag=tag!{{ 0,1,2,3 }};
const SET_0_FN:Tag=Tag::set(&[
    Tag::int(0),
    Tag::int(1),
    Tag::int(2),
    Tag::int(3),
]);


const MAP_0_MACRO:Tag=tag!{{
    0=>"a",
    1=>"b",
    2=>false,
    3=>100,
}};
const MAP_0_FN:Tag=Tag::map(&[
    Tag::kv( Tag::int(0), Tag::str("a")),
    Tag::kv( Tag::int(1), Tag::str("b")),
    Tag::kv( Tag::int(2), Tag::bool_(false)),
    Tag::kv( Tag::int(3), Tag::int(100)),
]);


```

# Creating a complex data structure.


```rust
use abi_stable::{
    tag,
    abi_stability::Tag,
};

const TAG:Tag=tag!{{
    // This must match exactly,
    // adding required traits on the interface or the implementation
    // would be a breaking change.
    "required"=>tag![[
        "Copy",
    ]],
    
    "requires at least"=>tag!{{
        "Debug",
        "Display",
    }},
}};


```
*/
#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub enum Tag{
    Primitive(Primitive),
    Array(StaticSlice<Tag>),
    Set(StaticSlice<Tag>),
    Map(StaticSlice<KeyValue<Tag>>),
}

#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub enum Primitive{
    Null,
    Bool(bool),
    Int(i64),
    UInt(u64),
    String_(StaticStr),
}

#[repr(C)]
#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub enum CheckableTag{
    Primitive(Primitive),
    Array(Vec<CheckableTag>),
    Set(BTreeMap<CheckableTag,CheckableTag>),
    Map(BTreeMap<CheckableTag,CheckableTag>),
}


#[repr(C)]
#[derive(Debug,Copy,Clone,PartialEq,Eq,PartialOrd,Ord,Hash,StableAbi)]
#[sabi(inside_abi_stable_crate)]
pub struct KeyValue<T>{
    key:T,
    value:T,
}


impl Tag{
    pub const fn null()->Self{
        Tag::Primitive(Primitive::Null)
    }
    pub const fn bool_(b:bool)->Self{
        Tag::Primitive(Primitive::Bool(b))
    }

    pub const fn int(n:i64)->Self{
        Tag::Primitive(Primitive::Int(n))
    }

    pub const fn uint(n:u64)->Self{
        Tag::Primitive(Primitive::UInt(n))
    }

    pub const fn str(s:&'static str)->Self{
        Tag::Primitive(Primitive::String_(StaticStr::new(s)))
    }

    pub const fn arr(s:&'static [Tag])->Self{
        Tag::Array(StaticSlice::new(s))
    }

    pub const fn set(s:&'static [Tag])->Self{
        Tag::Set(StaticSlice::new(s))
    }

    pub const fn kv(key:Tag,value:Tag)->KeyValue<Tag>{
        KeyValue{key,value}
    }

    pub const fn map(s:&'static [KeyValue<Tag>])->Self{
        Tag::Map(StaticSlice::new(s))
    }
}

impl Tag{
    pub fn to_checkable(self)->CheckableTag{
        match self {
            Tag::Primitive(prim)=>CheckableTag::Primitive(prim),
            Tag::Array(arr)=>{
                arr.iter().cloned()
                    .filter(|x| *x!=Tag::null() )
                    .map(Self::to_checkable)
                    .collect::<Vec<CheckableTag>>()
                    .piped(CheckableTag::Array)
            }
            Tag::Set(arr)=>{
                arr.iter().cloned()
                    .filter(|x| *x!=Tag::null() )
                    .map(|x| (x.to_checkable(),Tag::null().to_checkable()) )
                    .collect::<BTreeMap<CheckableTag,CheckableTag>>()
                    .piped(CheckableTag::Set)
            }
            Tag::Map(arr)=>{
                arr.iter().cloned()
                    .filter(|kv| kv.key!=Tag::null() )
                    .map(|kv| (kv.key.to_checkable(),kv.value.to_checkable())  )
                    .collect::<BTreeMap<CheckableTag,CheckableTag>>()
                    .piped(CheckableTag::Map)
            }
        }
    }
}

impl CheckableTag{
    pub fn check_compatible(&self,other:&Self)->Result<(),TagErrors>{
        use self::CheckableTag as CT;

        let err_with_variant=|vari:TagErrorVariant|{
            TagErrors{
                expected:self.clone(),
                found:other.clone(),
                backtrace:vec![].into(),
                errors:vec![vari].into(),
            }
        };

        let mismatched_val_err=|cond:bool|{
            if cond {
                Ok(())
            }else{
                Err(err_with_variant(TagErrorVariant::MismatchedValue))
            }
        };

        let same_variant=match (self,other) {
            (CT::Primitive(Primitive::Null),_)=>
                return Ok(()),
            (CT::Primitive(l),CT::Primitive(r))=>
                mem::discriminant(l)==mem::discriminant(r),
            (l,r)=>
                mem::discriminant(l)==mem::discriminant(r),
        };

        if !same_variant {
            return Err(err_with_variant(TagErrorVariant::MismatchedDiscriminant))
        }

        let is_map=matches!(CT::Map{..}=self);
        
        match (self,other) {
            (CT::Primitive(l),CT::Primitive(r))=>{
                match (l,r) {
                    (Primitive::Null,Primitive::Null)=>(),
                    (Primitive::Null,_)=>(),

                    (Primitive::Bool(l_cond),Primitive::Bool(r_cond))=>
                        mismatched_val_err(l_cond==r_cond)?,
                    (Primitive::Bool(cond),_)=>{},
                    
                    (Primitive::Int(l_num),Primitive::Int(r_num))=>
                        mismatched_val_err(l_num==r_num)?,
                    (Primitive::Int(num),_)=>{},

                    (Primitive::UInt(l_num),Primitive::UInt(r_num))=>
                        mismatched_val_err(l_num==r_num)?,
                    (Primitive::UInt(num),_)=>{},
                    
                    (Primitive::String_(l_str),Primitive::String_(r_str))=>
                        mismatched_val_err(l_str.as_str()==r_str.as_str())?,
                    (Primitive::String_(s),_)=>{},
                }    
            },
            (CT::Primitive(_),_)=>{}
            
            (CT::Array(l_arr),CT::Array(r_arr))=>{
                let l_arr=l_arr.as_slice();
                let r_arr=r_arr.as_slice();

                if l_arr.len()!=r_arr.len() {
                    let e=TagErrorVariant::MismatchedArrayLength{
                        expected:l_arr.len(),
                        found:r_arr.len(),
                    };
                    return Err(err_with_variant(e));
                }

                for (l_elem,r_elem) in l_arr.iter().zip(r_arr.iter()) {
                    l_elem.check_compatible(r_elem)
                        .map_err(|errs| errs.context(l_elem.clone()) )?;
                }
            }
            (CT::Array(arr),_)=>{},
            
             (CT::Set(l_map),CT::Set(r_map))
            |(CT::Map(l_map),CT::Map(r_map))=>{
                if l_map.len() > r_map.len() {
                    let e=TagErrorVariant::MismatchedAssocLength{
                        expected:l_map.len(),
                        found:r_map.len(),
                    };
                    return Err(err_with_variant(e));
                }

                let mut r_iter=r_map.iter();

                'outer:for (l_key,l_elem) in l_map {
                    let mut first_err=None::<KeyValue<&CheckableTag>>;
                    
                    'inner:loop {
                        let (r_key,r_elem)=match r_iter.next() {
                            Some(x)=>x,
                            None=>break 'inner,
                        };

                        match l_key.check_compatible(r_key)
                            .and_then(|_| l_elem.check_compatible(r_elem) )
                        {
                            Ok(_)=>continue 'outer,
                            Err(_)=>{
                                first_err.get_or_insert( KeyValue::new(r_key,r_elem) );
                            },
                        }
                    }

                    let e=if is_map {
                        TagErrorVariant::MismatchedMapEntry{
                            expected:KeyValue::new(l_key.clone(),l_elem.clone()),
                            found:first_err.map(|x|x.map(Clone::clone)).into_c(),
                        }
                    }else{
                        TagErrorVariant::MissingSetValue{
                            expected:l_key.clone(),
                            found:first_err.map(|x|x.key).cloned().into_c(),
                        }
                    };
                    return Err(err_with_variant(e));
                }
            }
            (CT::Set(set),_)=>{},
            (CT::Map(set),_)=>{},
        }
        Ok(())
    }

}


/////////////////////////////////////////////////////////////////

impl<T> KeyValue<T>{
    pub const fn new(key:T,value:T)->Self{
        Self{ key,value }
    }
    pub fn map<F,U>(self,mut f:F)->KeyValue<U>
    where F:FnMut(T)->U
    {
        KeyValue{
            key:f(self.key),
            value:f(self.value),
        }
    }
}

impl<T> Display for KeyValue<T> 
where T:Display
{
    fn fmt(&self,f:&mut fmt::Formatter)->fmt::Result{
        write!(f,"{}=>{}",self.key,self.value)
    }
}


/////////////////////////////////////////////////////////////////


pub struct FromLiteral<T>(pub T);

impl FromLiteral<bool>{
    pub const fn to_tag(self)->Tag{
        Tag::bool_(self.0)
    }
}

impl FromLiteral<&'static str>{
    pub const fn to_tag(self)->Tag{
        Tag::str(self.0)
    }
}

impl FromLiteral<i64>{
    pub const fn to_tag(self)->Tag{
        Tag::int(self.0)
    }
}

impl FromLiteral<Tag>{
    pub const fn to_tag(self)->Tag{
        self.0
    }
}


/////////////////////////////////////////////////////////////////


fn display_iter<I>(iter:I,f:&mut fmt::Formatter<'_>,indent:usize)->fmt::Result 
where
    I:IntoIterator,
    I::Item:Display,
{
    let mut buffer=String::new();
    for elem in iter {
        Display::fmt(&buffer.display_pad(indent,&elem)?,f)?;
        writeln!(f,",")?;
    }
    Ok(())
}


 

impl Display for Primitive {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
        match *self {
            Primitive::Null=>{
                write!(f,"null")?;
            },
            Primitive::Bool(cond)=>{
                write!(f,"{}",cond)?;
            },
            Primitive::Int(num)=>{
                write!(f,"{}",num)?;
            },
            Primitive::UInt(num)=>{
                write!(f,"{}",num)?;
            },
            Primitive::String_(s)=>{
                write!(f,"'{}'",s)?;
            },
        }
        Ok(())
    }
}


macro_rules! impl_display {
    ($ty:ident) => (
        impl Display for $ty {
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
                match self {
                    $ty::Primitive(prim)=>{
                        Display::fmt(prim,f)?;
                    },
                    $ty::Array(arr)=>{
                        writeln!(f,"[")?;
                        display_iter(&*arr,f,4)?;
                        write!(f,"]")?;
                    },
                    $ty::Set(map)|$ty::Map(map)=>{
                        writeln!(f,"{{")?;
                        let iter=map.iter().map(|(k,v)| KeyValue::new(k,v) );
                        display_iter(iter,f,4)?;
                        write!(f,"}}")?;
                    },
                }
                Ok(())
            }
        }
    )
}

impl_display!{CheckableTag}


/////////////////////////////////////////////////////////////////



/////////////////////////////////////////////////////////////////


#[derive(Debug,Clone,PartialEq)]
pub struct TagErrors{
    expected:CheckableTag,
    found:CheckableTag,
    backtrace:RVec<CheckableTag>,
    errors:RVec<TagErrorVariant>,
}


impl TagErrors{
    pub fn context(mut self,current:CheckableTag)->Self{
        self.backtrace.push(current);
        self
    }
}


impl Display for TagErrors {
    fn fmt(&self,f:&mut fmt::Formatter)->fmt::Result{
        let mut buffer=String::new();

        writeln!(f,"Stacktrace:")?;
        if self.backtrace.is_empty() {
            writeln!(f,"    Empty.")?;
        }else{
            for stack in self.backtrace.iter().rev() {
                writeln!(f,"    Inside:\n{},",buffer.display_pad(8,stack)? )?;
            }
        }
        writeln!(f,"Expected:\n{}",buffer.display_pad(4,&self.expected)? )?;
        writeln!(f,"Found:\n{}",buffer.display_pad(4,&self.found)? )?;
        writeln!(f,"Errors:\n")?;
        for stack in self.backtrace.iter().rev() {
            writeln!(f,"    Error:\n{},",buffer.display_pad(8,stack)? )?;
        }
        Ok(())
    }
}


/////////////////////////////////////////////////////////////////


#[derive(Debug,Clone,PartialEq)]
pub enum TagErrorVariant{
    MismatchedDiscriminant,
    MismatchedValue,
    MismatchedArrayLength{
        expected:usize,
        found:usize,
    },
    MismatchedAssocLength{
        expected:usize,
        found:usize,
    },
    MissingSetValue{
        expected:CheckableTag,
        found:ROption<CheckableTag>,
    },
    MismatchedMapEntry{
        expected:KeyValue<CheckableTag>,
        found:ROption<KeyValue<CheckableTag>>,
    },
}


impl Display for TagErrorVariant {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
        match self {
            TagErrorVariant::MismatchedDiscriminant=>{
                writeln!(f,"Mismatched Tag variant.")?;
            }
            TagErrorVariant::MismatchedValue=>{
                writeln!(f,"Mitmatched Value.")?;
            }
            TagErrorVariant::MismatchedArrayLength{expected,found}=>{
                writeln!(f,"Mismatched length  expected:{}  found:{}",expected,found)?;
            }
            TagErrorVariant::MismatchedAssocLength{expected,found}=>{
                writeln!(
                    f,
                    "Mismatched length  expected at least:{}  found:{}",
                    expected,
                    found,
                )?;
            }
            TagErrorVariant::MissingSetValue{expected,found}=>{
                let mut buffer=String::new();
                writeln!(
                    f,
                    "Mismatched value in set\nExpected:\n{}",
                    buffer.display_pad(4,&expected)?
                )?;
                match found {
                    RSome(found) => writeln!(f,"Found:\n{}",buffer.display_pad(4,&found)?),
                    RNone => writeln!(f,"Found:\n    Nothing",),
                }?;
            },
            TagErrorVariant::MismatchedMapEntry{expected,found}=>{
                let mut buffer=String::new();
                writeln!(
                    f,
                    "Mismatched entry in map\nExpected:\n{}",
                    buffer.display_pad(4,&expected)?
                )?;
                match found {
                    RSome(found) => writeln!(f,"Found:\n{}",buffer.display_pad(4,&found)?),
                    RNone => writeln!(f,"Found:\n    Nothing",),
                }?;
            },
        }
        Ok(())
    }
}


////////////////////////////////////////////////////////////////////////////////



#[cfg(test)]
mod test;
