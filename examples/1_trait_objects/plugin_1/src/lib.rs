/*!
This is an `implementation crate`,
It exports the root module(a struct of function pointers) required by the 
`example_0_interface`(the `interface crate`) in the 
version of `get_library` with a mangled function name.

*/

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
    Application,
    ApplicationMut,
    CommandDescription,
    Error as AppError,
    Plugin,PluginType,PluginId,PluginMod,PluginModVal,
    Plugin_from_value,
    utils::process_command,
    WhichPlugin,
};


use core_extensions::SelfOps;

use serde::{Serialize,Deserialize};

use serde_json::value::Value;

///////////////////////////////////////////////////////////////////////////////////


/// Exports the root module of this library.
///
/// This code isn't run until the layout of the type it returns is checked.
#[export_root_module]
fn instantiate_root_module()->&'static PluginMod{
    PluginModVal {
        new,
    }.leak_into_prefix()
}


//////////////////////////////////////////////////////////////////////////////////////


#[sabi_extern_fn]
pub fn new(_sender:RSender<AsyncCommand>,plugin_id:PluginId) -> RResult<PluginType,AppError> {
    let this=TextMunging{
        plugin_id,
    };
    ROk(Plugin_from_value::<_,TU_Opaque>(this))
}


//////////////////////////////////////////////////////////////////////////////////////



#[derive(Debug,Serialize,Deserialize)]
pub enum UtilCmd{
    Repeat{
        how_much:usize,
        plugin:WhichPlugin,
        command:Value,
    },
    Batch{
        plugin:WhichPlugin,
        commands:Vec<Value>,
    },
}

#[derive(Debug,Serialize,Deserialize)]
pub enum ReturnValue{
    Repeat,
    Batch,
}





//////////////////////////////////////////////////////////////////////////////////////


fn run_command_inner(
    this:&mut TextMunging,
    command:UtilCmd,
    mut app:ApplicationMut<'_>,
)->Result<ReturnValue,AppError>{
    match command {
        UtilCmd::Repeat{how_much,plugin,command}=>{
            let s=serde_json::to_string(&command).unwrap();
            for _ in 0..how_much {
                app.send_command_to_plugin(
                    this.plugin_id(),
                    plugin.clone(),
                    s.clone().into(),
                ).into_result()?;
            }
            ReturnValue::Repeat
        }
        UtilCmd::Batch{plugin,commands}=>{
            for command in commands {
                let s=serde_json::to_string(&command).unwrap();
                app.send_command_to_plugin(
                    this.plugin_id(),
                    plugin.clone(),
                    s.into(),
                ).into_result()?;
            }
            ReturnValue::Batch
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
        process_command(self,command,|this,command|{
            run_command_inner(this,command,app)
        })
    }

    fn plugin_id(&self)->&PluginId{
        &self.plugin_id
    }

    fn list_commands(&self)->RVec<CommandDescription>{
        vec![
            CommandDescription::from_literals(
                "Repeat",
"\
Sends a command to a plugin N times.

Command params:
{
    \"how_much\":10,
    \"plugin\":\"plugin:last\",
    \"command\":{ ... some command ... }
}
"
            ),
            CommandDescription::from_literals(
                "Batch",
"\
Sends a sequence of commands to a plugin.

Command params:
{
    \"plugin\":\"plugin_name\",
    \"commands\":[
        { ... some command ... },
        { ... some command ... },
        { ... some command ... }
    ]
}",
            ),
        ].into()
    }

    fn close(self,_app:ApplicationMut<'_>){}
}

