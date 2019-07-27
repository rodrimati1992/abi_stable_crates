use crate::{
    commands::{
        BasicCommand,
        CommandUnion,
        CommandUnion as CU,
        CommandTrait,
        ReturnValUnion,
        ReturnValUnion as RVU,
        WhichVariant,
        BasicRetVal,
    },
    error::Unsupported,
    ApplicationMut,WhichCommandRet,Error,Plugin,PluginType,
};

use abi_stable::{
    std_types::{RStr, RString, RBoxError, RResult},
};

use serde::{Serialize,Deserialize};


/**
Sends a json encoded command to a plugin,and returns the response by encoding it to json.

# Errors

These are all error that this function returns
(this does not include error returned as part of the command):

- Error::Serialize:
    If the command/return value could not be serialized to JSON.

- Error::Deserialize
    If the command/return value could not be deserialized from JSON(this comes from the plugin).

- Error::UnsupportedCommand
    If the command is not supported by the plugin.

*/
pub fn process_command<'de,P,C,R,F>(this:&mut P,command:RStr<'de>,f:F)->RResult<RString,Error>
where 
    P:Plugin,
    F:FnOnce(&mut P,C)->Result<R,Error>,
    C:Deserialize<'de>,
    R:Serialize,
{
    (||->Result<RString,Error>{
        let command=command.as_str();

        let which_variant=serde_json::from_str::<WhichVariant>(&command)
            .map_err(|e| Error::Deserialize(RBoxError::new(e),WhichCommandRet::Command) )?;

        let command=serde_json::from_str::<CommandUnion<C>>(command)
            .map_err(|e|{
                Error::unsupported_command(Unsupported{
                    plugin_name:this.plugin_id().named.clone().into_owned(),
                    command_name:which_variant.variant,
                    error:RBoxError::new(e),
                    supported_commands:this.list_commands(),
                })
            })?;

        let ret:ReturnValUnion<R>=match command {
            CU::Basic(BasicCommand::GetCommands)=>{
                let commands=this.list_commands();
                RVU::Basic(BasicRetVal::GetCommands(commands))
            }
            CU::ForPlugin(cmd)=>{
                RVU::ForPlugin(f(this,cmd)?)
            }
        };

        match serde_json::to_string(&ret) {
            Ok(v)=>Ok(v.into()),
            Err(e)=>Err(Error::Serialize(RBoxError::new(e),WhichCommandRet::Return)),
        }
    })().into()
}




/**
Sends a typed command to a plugin.

# Errors

These are all error that this function returns
(this does not include error returned as part of the command):

- Error::Serialize:
    If the command/return value could not be serialized to JSON.

- Error::Deserialize
    If the command/return value could not be deserialized from JSON(this comes from the plugin).

- Error::UnsupportedReturnValue:
    If the return value could not be deserialized from JSON
    (after checking that it has the `{"name":"...",description: ... }` format),
    containing the name of the command this is a return value for .

- Error::UnsupportedCommand
    If the command is not supported by the plugin.

*/
pub fn send_command<C>(
    this:&mut PluginType,
    command:&C,
    app:ApplicationMut<'_>
)->Result<C::Returns,Error>
where 
    C:CommandTrait,
{
    let cmd=serde_json::to_string(&command)
         .map_err(|e| Error::Serialize(RBoxError::new(e),WhichCommandRet::Command) )?;

    let ret=this.json_command(RStr::from(&*cmd),app).into_result()?;

    let which_variant=serde_json::from_str::<WhichVariant>(&*ret)
        .map_err(|e| Error::Deserialize(RBoxError::new(e),WhichCommandRet::Return) )?;

    serde_json::from_str::<C::Returns>(&ret)
        .map_err(|e|{
            Error::unsupported_return_value(Unsupported{
                plugin_name:this.plugin_id().named.clone().into_owned(),
                command_name:which_variant.variant,
                error:RBoxError::new(e),
                supported_commands:this.list_commands(),
            })
        })

}
