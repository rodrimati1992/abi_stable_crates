use std::collections::HashSet;

use abi_stable_example_interface::{
    RemoveWords, TOLib,HelloWorldSubMod, TOState, TOStateBox,ThirdParam
};

use abi_stable::{
    mangle_library_getter,
    extern_fn_panic_handling, impl_get_type_info,
    library::WithLayout,
    traits::{ImplType, IntoReprC,SerializeImplType},
    std_types::{RCow, RStr, RString,RResult,ROk,RErr,RBoxError}, 
    StableAbi, VirtualWrapper,
};
use core_extensions::{SelfOps,StringExt};

use serde::{Serialize,Deserialize};
use serde_json;

#[derive(Debug,Serialize,Deserialize)]
struct TextOperationState {
    processed_bytes: u64,
}

impl_get_type_info! {
    impl GetTypeInfo for TextOperationState
    version=0,1,0;
}
impl ImplType for TextOperationState {
    type Interface = TOState;
}

impl SerializeImplType for TextOperationState {
    fn serialize_impl<'a>(&'a self) -> Result<RCow<'a, str>, RBoxError> {
        match serde_json::to_string(&self) {
            Ok(v)=>Ok(v.into_c().piped(RCow::Owned)),
            Err(e)=>Err(RBoxError::new(e)),
        }
    }
}

pub extern "C" fn new() -> TOStateBox {
    extern_fn_panic_handling! {
        let this=TextOperationState{
            processed_bytes:0,
        };
        VirtualWrapper::from_value(this)
    }
}

pub extern "C" fn deserialize_state(s:RStr<'_>) -> RResult<TOStateBox, RBoxError>{
    extern_fn_panic_handling! {
        match serde_json::from_str::<TextOperationState>(s.into()) {
            Ok(x) => ROk(VirtualWrapper::from_value(x)),
            Err(e) => RErr(RBoxError::new(e)),
        }
    }
}

pub extern "C" fn reverse_lines<'a>(this: &mut TOStateBox, text: RStr<'a>,_:ThirdParam)-> RString {
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

// This is a separate function because ìnitializing `remove_words_str` with `remove_words`
// does not work,due to some bound lifetime stuff,once it does this function will be obsolete.
pub extern "C" fn remove_words_str(
    this: &mut TOStateBox,
    param: RemoveWords<'_, RStr<'_>>,
) -> RString {
    remove_words(this, param)
}

// This is a separate function because ìnitializing `remove_words_cow` with `remove_words`
// does not work,due to some bound lifetime stuff,once it does this function will be obsolete.
pub extern "C" fn remove_words_cow(
    this: &mut TOStateBox,
    param: RemoveWords<'_, RCow<'_, str>>,
) -> RString {
    remove_words(this, param)
}

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
pub extern "C" fn get_processed_bytes(this: &TOStateBox) -> u64 {
    extern_fn_panic_handling! {
        let this = this.as_unerased::<TextOperationState>().unwrap();
        this.processed_bytes
    }
}

#[mangle_library_getter]
pub extern "C" fn get_library() -> WithLayout<TOLib> {
    extern_fn_panic_handling!{
        // println!("inside get_library_text_operations");
        TOLib {
            new,
            deserialize_state,
            reverse_lines,
            remove_words_cow,
            remove_words_str,
            remove_words_string: remove_words,
            get_processed_bytes,
        }.piped(WithLayout::new)
    }
}



/////////////////////////////////////////////////////////////////////////////


pub extern "C" fn greeter(name:RStr<'_>){
    extern_fn_panic_handling!{
        println!("Hello, {}!", name);
    }
}



#[mangle_library_getter]
pub extern "C" fn get_hello_world_mod() -> WithLayout<HelloWorldSubMod> {
    extern_fn_panic_handling!{
        HelloWorldSubMod{
            greeter,
        }.piped(WithLayout::new)
    }
}


/////////////////////////////////////////////////////////////////////////////


#[cfg(test)]
mod tests{
    use super::*;

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
        let json=r#"
            {
                "processed_bytes":101,
            }
        "#;

        let json_string=serde_json::to_string(json).unwrap();

        let value0=TOStateBox::deserialize_from_str(json).unwrap();

        let value1=serde_json::from_str::<TOStateBox>(&json_string).unwrap();

        assert_eq!(value0,value1 );

    }


    #[test]
    fn serializing(){
        let this=TextOperationState {
            processed_bytes: 1337,
        }.piped(VirtualWrapper::from_value);

        let serialized_0= this.serialized().split_whitespace().collect::<String>();

        let expected_0=r#"{"processed_bytes":1337}"#;

        assert_eq!(serialized_0,expected_0);

        assert_eq!(
            serde_json::to_string(&this), 
            serde_json::to_string(&expected_0),
        );
    }

}


