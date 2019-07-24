/*!
This is an `implementation crate`,
It exports the root module(a struct of function pointers) required by the 
`example_0_interface`(the `interface crate`) in the 
version of `get_library` with a mangled function name.

*/

use std::{
    borrow::Cow,
    collections::HashSet,
    marker::PhantomData,
};

use example_0_interface::{
    RemoveWords, CowStrIter,
    TextOpsMod,TextOpsModVal,
    DeserializerModVal,
    TOState, TOStateBox,TOCommand,TOReturnValue,TOCommandBox,TOReturnValueArc,
};

use abi_stable::{
    external_types::RawValueBox,
    export_root_module,
    sabi_extern_fn,
    impl_get_type_info,
    erased_types::{ImplType,SerializeImplType,TypeInfo},
    prefix_type::{PrefixTypeTrait,WithMetadata},
    traits::{IntoReprC},
    std_types::{RCow, RStr,RBox,RVec,RArc, RString,RResult,ROk,RErr,RBoxError}, 
    DynTrait,
};
use core_extensions::{SelfOps,StringExt};

use serde::{Serialize,Deserialize};
use serde_json;


///////////////////////////////////////////////////////////////////////////////////

/// Exports the root module of this library.
///
/// This code isn't run until the layout of the type it returns is checked.
#[export_root_module]
// #[unsafe_no_layout_constant]
fn instantiate_root_module()->&'static TextOpsMod{
    TextOpsModVal {
        new,
        deserializers:{
            // Another way to instantiate a module.
            const MOD_:DeserializerModVal=DeserializerModVal{
                something:PhantomData,
                deserialize_state,
                deserialize_command,
                deserialize_command_borrowing,
                deserialize_return_value,
            };
            static WITH_META:WithMetadata<DeserializerModVal>=
                WithMetadata::new(PrefixTypeTrait::METADATA,MOD_);
            WITH_META.as_prefix()
        },
        reverse_lines,
        remove_words,
        get_processed_bytes,
        run_command,
    }.leak_into_prefix()
}


///////////////////////////////////////////////////////////////////////////////////


#[derive(Debug,Serialize,Deserialize,PartialEq)]
struct TextOperationState {
    processed_bytes: u64,
}

/// Declares TOState as the `ìnterface type` of `TextOperationState`.
///
/// Also declares the INFO constant,with information about the type,
/// used when erasing/unerasing the type with `DynTrait<_>`.
///
/// TOState defines which traits are required when constructing DynTrait<_>,
/// and which ones it provides after constructing it.
impl ImplType for TextOperationState {
    type Interface = TOState;

    const INFO: &'static TypeInfo=impl_get_type_info! { TextOperationState };
}

/// Defines how the type is serialized in DynTrait<_>.
impl SerializeImplType for TextOperationState {
    type Interface = TOState;
    
    fn serialize_impl<'a>(&'a self) -> Result<RawValueBox, RBoxError> {
        serialize_json(self)
    }
}


//////////////////////////////////////////////////////////////////////////////////////


#[derive(Debug,Serialize,Deserialize,PartialEq)]
pub enum Command<'a> {
    ReverseLines(RString),
    RemoveWords{
        string:RString,
        words:RVec<RString>,
        #[serde(skip)]
        _marker:PhantomData<&'a mut RString>,
    },
    GetProcessedBytes,
    Batch(RVec<Command<'a>>),
}



impl<'a> Iterator for Command<'a>{
    type Item=&'a mut RString;

    fn next(&mut self)->Option<Self::Item>{
        None
    }
}


/// Declares TOState as the `ìnterface type` of `TOCommand`.
///
/// Also declares the INFO constant,with information about the type,
/// used when erasing/unerasing the type with `DynTrait<_>`.
///
/// TOCommand defines which traits are required when constructing DynTrait<_>,
/// and which ones it provides after constructing it.
impl ImplType for Command<'static> {
    type Interface = TOCommand;

    const INFO: &'static TypeInfo=impl_get_type_info! { Command };
}

/// Defines how the type is serialized in DynTrait<_>.
impl<'borr> SerializeImplType for Command<'borr> {
    type Interface = TOCommand;
    fn serialize_impl<'a>(&'a self) -> Result<RawValueBox, RBoxError> {
        serialize_json(self)
    }
}



//////////////////////////////////////////////////////////////////////////////////////


#[derive(Debug,Serialize,Deserialize,PartialEq)]
pub enum ReturnValue {
    ReverseLines(RString),
    RemoveWords(RString),
    GetProcessedBytes(u64),
    Batch(RVec<ReturnValue>),
}

/// Declares TOState as the `ìnterface type` of `TOReturnValue`.
///
/// Also declares the INFO constant,with information about the type,
/// used when erasing/unerasing the type with `DynTrait<_>`.
///
/// TOReturnValue defines which traits are required when constructing DynTrait<_>,
/// and which ones it provides after constructing it.
impl ImplType for ReturnValue {
    type Interface = TOReturnValue;

    const INFO: &'static TypeInfo=impl_get_type_info! { ReturnValue };
}

/// Defines how the type is serialized in DynTrait<_>.
impl SerializeImplType for ReturnValue {
    type Interface = TOReturnValue;
    fn serialize_impl<'a>(&'a self) -> Result<RawValueBox, RBoxError> {
        serialize_json(self)
    }
}



//////////////////////////////////////////////////////////////////////////////////////


fn deserialize_json<'a, T>(s: RStr<'a>) -> RResult<T, RBoxError>
where
    T: serde::Deserialize<'a>,
{
    match serde_json::from_str::<T>(s.into()) {
        Ok(x) => ROk(x),
        Err(e) => RErr(RBoxError::new(e)),
    }
}

fn serialize_json<'a, T>(value: &'a T) -> Result<RawValueBox, RBoxError>
where
    T: serde::Serialize,
{
    match serde_json::to_string::<T>(&value) {
        Ok(v)=>unsafe{ Ok(RawValueBox::from_rstring_unchecked(v.into_c())) },
        Err(e)=>Err(RBoxError::new(e)),
    }
}


//////////////////////////////////////////////////////////////////////////////////////

/// Defines how a TOStateBox is deserialized from json.
#[sabi_extern_fn]
pub fn deserialize_state(s:RStr<'_>) -> RResult<TOStateBox, RBoxError>{
    deserialize_json::<TextOperationState>(s)
        .map(DynTrait::from_value)
}

/// Defines how a TOCommandBox is deserialized from json.
#[sabi_extern_fn]
pub fn deserialize_command(
    s:RStr<'_>
) -> RResult<TOCommandBox<'static>, RBoxError>{
    deserialize_json::<Command>(s)
        .map(RBox::new)
        .map(DynTrait::from_ptr)
}

/// Defines how a TOCommandBox is deserialized from json.
#[sabi_extern_fn]
pub fn deserialize_command_borrowing<'borr>(
    s:RStr<'borr>
) -> RResult<TOCommandBox<'borr>, RBoxError>{
    deserialize_json::<Command>(s)
        .map(RBox::new)
        .map(|x|DynTrait::from_borrowing_ptr(x,TOCommand))
}

/// Defines how a TOReturnValueArc is deserialized from json.
#[sabi_extern_fn]
pub fn deserialize_return_value(s:RStr<'_>) -> RResult<TOReturnValueArc, RBoxError>{
    deserialize_json::<ReturnValue>(s)
        .map(RArc::new)
        .map(DynTrait::from_ptr)
}

//////////////////////////////////////////////////////////////////////////////////////


/// Constructs a TextOperationState and erases it by wrapping it into a 
/// `DynTrait<Box<()>,TOState>`.
#[sabi_extern_fn]
pub fn new() -> TOStateBox {
    let this=TextOperationState{
        processed_bytes:0,
    };
    DynTrait::from_value(this)
}



/// Reverses order of the lines in `text`.
#[sabi_extern_fn]
pub fn reverse_lines<'a>(this: &mut TOStateBox, text: RStr<'a>)-> RString {
    let this = this.sabi_as_unerased_mut::<TextOperationState>().unwrap();

    this.processed_bytes+=text.len() as u64;

    let mut lines=text.lines().collect::<Vec<&str>>();
    lines.reverse();
    let mut buffer=RString::with_capacity(text.len());
    for line in lines {
        buffer.push_str(line);
        buffer.push('\n');
    }
    buffer
}


/// Removes the words in `param.words` from `param.string`,
/// as well as the whitespace that comes after it.
#[sabi_extern_fn]
pub fn remove_words<'w>(this: &mut TOStateBox, param: RemoveWords<'w,'_>) -> RString{
    let this = this.sabi_as_unerased_mut::<TextOperationState>().unwrap();

    this.processed_bytes+=param.string.len() as u64;

    let set=param.words.map(RCow::into).collect::<HashSet<Cow<'_,str>>>();
    let mut buffer=String::with_capacity(10);

    let haystack=&*param.string;
    let mut prev_was_deleted=false;
    for kv in haystack.split_while(|c|c.is_alphabetic()) {
        let s=kv.str;
        let cs=Cow::from(s);
        let is_a_word=kv.key;
        let is_deleted= (!is_a_word&&prev_was_deleted) || (is_a_word && set.contains(&cs));
        if !is_deleted {
            buffer.push_str(s);
        }
        prev_was_deleted=is_deleted;
    }

    buffer.into()
}

/// Returns the ammount of text (in bytes) 
/// that was processed in functions taking `&mut TOStateBox`.
#[sabi_extern_fn]
pub fn get_processed_bytes(this: &TOStateBox) -> u64 {
    let this = this.sabi_as_unerased::<TextOperationState>().unwrap();
    this.processed_bytes
}



fn run_command_inner(this:&mut TOStateBox,command:Command)->ReturnValue{
    match command {
        Command::ReverseLines(s)=>{
            reverse_lines(this,s.as_rstr())
                .piped(ReturnValue::ReverseLines)
        }
        Command::RemoveWords{string,words,_marker:_}=>{
            let iter=&mut words.iter().map(|s| RCow::Borrowed(s.as_rstr()) );

            remove_words(this,RemoveWords{
                string:string.as_rstr(),
                words:DynTrait::from_borrowing_ptr(iter,CowStrIter),
            })
            .piped(ReturnValue::RemoveWords)
        }
        Command::GetProcessedBytes=>{
            get_processed_bytes(this)
                .piped(ReturnValue::GetProcessedBytes)
        }
        Command::Batch(list)=>{
            list.into_iter()
                .map(|cmd| run_command_inner(this,cmd) )
                .collect::<RVec<ReturnValue>>()
                .piped(ReturnValue::Batch)
        }
    }
}


/// An interpreter for text operation commands
#[sabi_extern_fn]
pub fn run_command(
    this:&mut TOStateBox,
    command:TOCommandBox<'static>
)->TOReturnValueArc{
    let command = command.sabi_into_unerased::<Command<'static>>().unwrap()
        .piped(RBox::into_inner);
        
    run_command_inner(this,command)
        .piped(RArc::new)
        .piped(DynTrait::from_ptr)
}


/////////////////////////////////////////////////////////////////////////////


#[cfg(test)]
mod tests{
    use super::*;

    use abi_stable::library::RootModule;

    use serde_json::value::RawValue;

    fn setup(){
        let _=TextOpsMod::load_module_with(|| Ok::<_,()>(instantiate_root_module()) );
    }

    #[test]
    fn test_reverse_lines() {
        let mut state = new();
        assert_eq!(
            &*reverse_lines(&mut state, "hello\nbig\nworld".into()),
            "world\nbig\nhello\n"
        );
    }

    #[test]
    fn test_remove_words() {
        let mut state = new();
        {
            let words = ["burrito", "like","a"];
            let mut iter=words.iter().cloned().map(RCow::from);
            let param = RemoveWords {
                string: "Monads are like a burrito wrapper.".into(),
                words: DynTrait::from_borrowing_ptr(&mut iter,CowStrIter),
            };
            assert_eq!(&*remove_words(&mut state, param), "Monads are wrapper.");
        }
        {
            let words = ["largest","is"];
            let mut iter=words.iter().cloned().map(RCow::from);
            let param = RemoveWords {
                string: "The   largest planet  is    jupiter.".into(),
                words: DynTrait::from_borrowing_ptr(&mut iter,CowStrIter),
            };
            assert_eq!(&*remove_words(&mut state, param), "The   planet  jupiter.");
        }
    }

    #[test]
    fn deserializing(){
        setup();
        
        let json=r#"
            {
                "processed_bytes":101
            }
        "#;

        let rvref=serde_json::from_str::<&RawValue>(json).unwrap();
        let value0=TOStateBox::deserialize_from_proxy(rvref.into()).unwrap();

        let value1=serde_json::from_str::<TOStateBox>(&json).unwrap();

        assert_eq!(value0,value1);

    }


    #[test]
    fn serializing(){
        setup();

        let this=TextOperationState {
            processed_bytes: 1337,
        }.piped(DynTrait::from_value);

        let serialized_0= this.serialize_into_proxy()
            .unwrap()
            .get()
            .split_whitespace()
            .collect::<String>();

        let expected_0=r#"{"processed_bytes":1337}"#;

        assert_eq!(serialized_0,expected_0);

        assert_eq!(
            serde_json::to_string(&this).unwrap(), 
            expected_0,
        );
    }

}


