/*!
This is an `implementation crate`,
It exports all the modules(structs of function pointers) required by the 
`example_0_interface`(the `interface crate`) with these functions:

- get_library: Exports the root module 
- get_hello_world_mod :Exports an example module.

*/

use std::{
    collections::HashSet,
    marker::PhantomData,
};

use example_0_interface::{
    RemoveWords, 
    TextOpsMod,HelloWorldMod,TextOpsMod_Prefix,
    DeserializerMod,
    TOState, TOStateBox,TOCommand,TOReturnValue,TOCommandBox,TOReturnValueArc,
    PrefixTypeMod0,
    ForTests,
};

use abi_stable::{
    export_sabi_module,
    extern_fn_panic_handling, impl_get_type_info,
    library::WithLayout,
    erased_types::{ImplType,SerializeImplType,TypeInfo},
    prefix_type::{PrefixTypeTrait,WithMetadata},
    traits::{IntoReprC},
    std_types::{RCow, RStr,RBox,RVec,RArc, RString,RResult,ROk,RErr,RBoxError}, 
    StableAbi, DynTrait,
};
use core_extensions::{SelfOps,StringExt};

use serde::{Serialize,Deserialize};
use serde_json;


///////////////////////////////////////////////////////////////////////////////////


/// Exports the root module of this library.
///
/// WithLayout is used to check that the layout of `TextOpsMod` in this dynamic library
/// is compatible with the layout of it in the binary that loads this library.
#[export_sabi_module]
pub extern "C" fn get_library() -> WithLayout<TextOpsMod_Prefix> {
    extern_fn_panic_handling!{
        instantiate_root_module()
            .piped(WithLayout::from_prefix)
    }
}


fn instantiate_root_module()->&'static TextOpsMod_Prefix{
    TextOpsMod {
        new,
        hello_world:HelloWorldMod{
            greeter,
            for_tests,
        }.leak_into_prefix(),
        deserializers:{
            // Another way to instantiate a module.
            const MOD_:DeserializerMod=DeserializerMod{
                deserialize_state,
                deserialize_command,
                deserialize_command_borrowing,
                deserialize_return_value,
            };
            static WITH_META:WithMetadata<DeserializerMod>=
                WithMetadata::new(PrefixTypeTrait::METADATA,MOD_);
            WITH_META.as_prefix()
        },
        reverse_lines,
        remove_words_cow,
        remove_words_str,
        remove_words_string: remove_words,
        get_processed_bytes,
        run_command,
        prefix_types_tests:PrefixTypeMod0{
            field_a:123,
        }.leak_into_prefix(),
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
    fn serialize_impl<'a>(&'a self) -> Result<RCow<'a, RStr<'a>>, RBoxError> {
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
    fn serialize_impl<'a>(&'a self) -> Result<RCow<'a, RStr<'a>>, RBoxError> {
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
    fn serialize_impl<'a>(&'a self) -> Result<RCow<'a, RStr<'a>>, RBoxError> {
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

fn serialize_json<'a, T>(value: &'a T) -> Result<RCow<'a, RStr<'a>>, RBoxError>
where
    T: serde::Serialize,
{
    match serde_json::to_string::<T>(&value) {
        Ok(v)=>Ok(v.into_c().piped(RCow::Owned)),
        Err(e)=>Err(RBoxError::new(e)),
    }
}


//////////////////////////////////////////////////////////////////////////////////////

/// Defines how a TOStateBox is deserialized from json.
pub extern "C" fn deserialize_state(s:RStr<'_>) -> RResult<TOStateBox, RBoxError>{
    extern_fn_panic_handling! {
        deserialize_json::<TextOperationState>(s)
            .map(DynTrait::from_value)
    }
}

/// Defines how a TOCommandBox is deserialized from json.
pub extern "C" fn deserialize_command(
    s:RStr<'_>
) -> RResult<TOCommandBox<'static>, RBoxError>{
    extern_fn_panic_handling! {
        deserialize_json::<Command>(s)
            .map(RBox::new)
            .map(DynTrait::from_ptr)
    }
}

/// Defines how a TOCommandBox is deserialized from json.
pub extern "C" fn deserialize_command_borrowing<'borr>(
    s:RStr<'borr>
) -> RResult<TOCommandBox<'borr>, RBoxError>{
    extern_fn_panic_handling! {
        deserialize_json::<Command>(s)
            .map(RBox::new)
            .map(|x|DynTrait::from_borrowing_ptr(x,TOCommand))
    }
}

/// Defines how a TOReturnValueArc is deserialized from json.
pub extern "C" fn deserialize_return_value(s:RStr<'_>) -> RResult<TOReturnValueArc, RBoxError>{
    extern_fn_panic_handling! {
        deserialize_json::<ReturnValue>(s)
            .map(RArc::new)
            .map(DynTrait::from_ptr)
    }
}

//////////////////////////////////////////////////////////////////////////////////////


/// Constructs a TextOperationState and erases it by wrapping it into a 
/// `DynTrait<Box<()>,TOState>`.
pub extern "C" fn new() -> TOStateBox {
    extern_fn_panic_handling! {
        let this=TextOperationState{
            processed_bytes:0,
        };
        DynTrait::from_value(this)
    }
}



/// Reverses order of the lines in `text`.
pub extern "C" fn reverse_lines<'a>(this: &mut TOStateBox, text: RStr<'a>)-> RString {
    extern_fn_panic_handling! {
        let this = this.as_unerased_mut::<TextOperationState>().unwrap();

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
}

/// Removes the words in `param.words` from `param.string`,
/// as well as the whitespace that comes after it.
// This is a separate function because ìnitializing `remove_words_str` with `remove_words`
// does not work,due to some bound lifetime stuff,once it does this function will be obsolete.
pub extern "C" fn remove_words_str(
    this: &mut TOStateBox,
    param: RemoveWords<'_, RStr<'_>>,
) -> RString {
    remove_words(this, param)
}

/// Removes the words in `param.words` from `param.string`,
/// as well as the whitespace that comes after it.
// This is a separate function because ìnitializing `remove_words_cow` with `remove_words`
// does not work,due to some bound lifetime stuff,once it does this function will be obsolete.
pub extern "C" fn remove_words_cow<'a>(
    this: &mut TOStateBox,
    param: RemoveWords<'a, RCow<'a,RStr<'a>>>,
) -> RString {
    remove_words(this, param)
}

/// Removes the words in `param.words` from `param.string`,
/// as well as the whitespace that comes after it.
pub extern "C" fn remove_words<S>(this: &mut TOStateBox, param: RemoveWords<S>) -> RString
where
    S: AsRef<str> + Clone + StableAbi,
{
    extern_fn_panic_handling! {
        let this = this.as_unerased_mut::<TextOperationState>().unwrap();

        this.processed_bytes+=param.string.len() as u64;

        let set=param.words.iter().map(|s| s.as_ref_::<str>() ).collect::<HashSet<&str>>();
        let mut buffer=String::new();

        let haystack=&*param.string;
        let mut prev_was_deleted=false;
        for kv in haystack.split_while(|c|c.is_alphabetic()) {
            let s=kv.str;
            let is_a_word=kv.key;
            let is_deleted= (!is_a_word&&prev_was_deleted) || (is_a_word && set.contains(&s));
            if !is_deleted {
                buffer.push_str(s);
            }
            prev_was_deleted=is_deleted;
        }

        buffer.into()
    }
}

/// Returns the ammount of text (in bytes) 
/// that was processed in functions taking `&mut TOStateBox`.
pub extern "C" fn get_processed_bytes(this: &TOStateBox) -> u64 {
    extern_fn_panic_handling! {
        let this = this.as_unerased::<TextOperationState>().unwrap();
        this.processed_bytes
    }
}



fn run_command_inner(this:&mut TOStateBox,command:Command)->ReturnValue{
    match command {
        Command::ReverseLines(s)=>{
            reverse_lines(this,s.as_rstr())
                .piped(ReturnValue::ReverseLines)
        }
        Command::RemoveWords{string,words,_marker:_}=>{
            remove_words(this,RemoveWords{
                string:string.as_rstr(),
                words:words.as_rslice(),
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
pub extern "C" fn run_command(
    this:&mut TOStateBox,
    command:TOCommandBox<'static>
)->TOReturnValueArc{
    extern_fn_panic_handling! {
        let command = command.into_unerased::<Command<'static>>().unwrap().piped(RBox::into_inner);
        run_command_inner(this,command)
            .piped(RArc::new)
            .piped(DynTrait::from_ptr)
    }
}


/////////////////////////////////////////////////////////////////////////////


pub extern "C" fn greeter(name:RStr<'_>){
    extern_fn_panic_handling!{
        println!("Hello, {}!", name);
    }
}

pub extern "C" fn for_tests()->ForTests{
    extern_fn_panic_handling!{
        let arc=RArc::new(RString::from("hello"));
        let box_=RBox::new(10);
        let vec_=RVec::from(vec!["world".into_c()]);
        let string=RString::from("what the foo.");
        ForTests{
            arc_address:(&*arc) as *const _ as usize,
            arc,

            box_address:(&*box_) as *const _ as usize,
            box_,
            
            vec_address:vec_.as_ptr() as usize,
            vec_,
            
            string_address:string.as_ptr() as usize,
            string,
        }
    }
}


/////////////////////////////////////////////////////////////////////////////


#[cfg(test)]
mod tests{
    use super::*;

    use example_0_interface::{MODULES,Modules};

    fn setup(){
        MODULES.init(instantiate_root_module);
    }

    #[test]
    fn test_reverse_lines() {
        let mut state = new();
        assert_eq!(
            &*reverse_lines(&mut state, "hello\nbig\nworld".into(),()),
            "world\nbig\nhello\n"
        );
    }

    #[test]
    fn test_remove_words() {
        let mut state = new();
        {
            let words = ["burrito".into_c(), "like".into(),"a".into()];
            let param = RemoveWords {
                string: "Monads are like a burrito wrapper.".into(),
                words: words[..].into_c(),
            };
            assert_eq!(&*remove_words(&mut state, param), "Monads are wrapper.");
        }
        {
            let words = ["largest".into_c(),"is".into()];
            let param = RemoveWords {
                string: "The   largest planet  is    jupiter.".into(),
                words: words[..].into_c(),
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

        let json_string=serde_json::to_string(json).unwrap();

        let value0=TOStateBox::deserialize_from_str(json).unwrap();

        let value1=serde_json::from_str::<TOStateBox>(&json_string).unwrap();

        assert_eq!(value0,value1);

    }


    #[test]
    fn serializing(){
        setup();

        let this=TextOperationState {
            processed_bytes: 1337,
        }.piped(DynTrait::from_value);

        let serialized_0= this.serialized().unwrap().split_whitespace().collect::<String>();

        let expected_0=r#"{"processed_bytes":1337}"#;

        assert_eq!(serialized_0,expected_0);

        assert_eq!(
            serde_json::to_string(&this).unwrap(), 
            serde_json::to_string(&expected_0).unwrap(),
        );
    }

}


