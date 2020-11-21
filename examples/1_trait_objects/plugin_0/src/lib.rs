/*!
This is an `implementation crate`,
It exports the root module(a struct of function pointers) required by the 
`example_0_interface`(the `interface crate`).

*/

use std::{
    collections::HashSet,
};

use abi_stable::{
    export_root_module,
    sabi_extern_fn,
    external_types::crossbeam_channel::RSender,
    prefix_type::PrefixTypeTrait,
    sabi_trait::prelude::TU_Opaque,
    std_types::{RStr,RVec, RString,RResult,ROk},
};

use example_1_interface::{
    AsyncCommand,
    ApplicationMut,
    CommandDescription,
    Error as AppError,
    Plugin,PluginType,PluginId,PluginMod,PluginMod_Ref,
    Plugin_TO,
    utils::process_command,
};

use core_extensions::{SelfOps,StringExt};

use serde::{Serialize,Deserialize};

///////////////////////////////////////////////////////////////////////////////////


/// Exports the root module of this library.
///
/// This code isn't run until the layout of the type it returns is checked.
#[export_root_module]
fn instantiate_root_module()->PluginMod_Ref{
    PluginMod {
        new,
    }.leak_into_prefix()
}


//////////////////////////////////////////////////////////////////////////////////////


/// Instantiates the plugin.
#[sabi_extern_fn]
pub fn new(_sender:RSender<AsyncCommand>,plugin_id:PluginId) -> RResult<PluginType,AppError> {
    let this=TextMunging{
        plugin_id,
    };
    ROk(Plugin_TO::from_value(this,TU_Opaque))
}


//////////////////////////////////////////////////////////////////////////////////////



#[derive(Debug,Serialize,Deserialize)]
pub enum ProcessTextCmd{
    Rot13(String),
    CapitalizeWords{
        text:String,
        words:HashSet<String>
    },
}

#[derive(Debug,Serialize,Deserialize)]
pub enum ReturnValue{
    Rot13(String),
    CapitalizeWords(String),
}





//////////////////////////////////////////////////////////////////////////////////////


fn run_command_inner(
    _this:&mut TextMunging,
    command:ProcessTextCmd,
    _app:ApplicationMut<'_>,
)->Result<ReturnValue,AppError>{
    match command {
        ProcessTextCmd::Rot13(text)=>{
            pub fn rot13(this:char)-> char{
                match this{
                    v@'a'..='z'=>
                        ((((v as u8 - b'a')+13)%26)+b'a')as char,
                    v@'A'..='Z'=>
                        ((((v as u8 - b'A')+13)%26)+b'A')as char,
                    v=>v
                }
            }
            text.chars()
                .map(rot13)
                .collect::<String>()
                .piped(ReturnValue::Rot13)
        }
        ProcessTextCmd::CapitalizeWords{text,words}=>{
            let mut buffer=String::with_capacity(10);

            for kv in text.split_while(|c|c.is_alphabetic()) {
                let str_=kv.str;
                let is_a_word=kv.key;
                if is_a_word && words.contains(str_) {
                    buffer.extend(str_.chars().flat_map(char::to_uppercase));
                }else{
                    buffer.push_str(str_);
                }
            }
            ReturnValue::CapitalizeWords(buffer)
        }
    }.piped(Ok)
}

//////////////////////////////////////////////////////////////////////////////////////


struct TextMunging{
    plugin_id:PluginId,
}


impl Plugin for TextMunging {
    fn json_command(
        &mut self,
        command: RStr<'_>,
        app:ApplicationMut<'_>,
    )->RResult<RString,AppError>{
        process_command(self,command,|this,command:ProcessTextCmd|{
            run_command_inner(this,command,app)
        })
    }

    fn plugin_id(&self)->&PluginId{
        &self.plugin_id
    }

    fn list_commands(&self)->RVec<CommandDescription>{
        vec![
            CommandDescription::from_literals(
                "Rot13",
                "Uses the rot13 algorithm to hide spoilers in plain text."
            ),
            CommandDescription::from_literals(
                "CapitalizeWords",
                "Capitalizes the words specified in words.\n\
                 \n\
                 Command parans:\n\
                 {\"text\":\"text here\",words:[\"word0\",\"word1\"]}",
            ),
        ].into()
    }

    fn close(self,_app:ApplicationMut<'_>){}
}

