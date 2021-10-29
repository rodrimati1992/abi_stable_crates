//! This is an `implementation crate`,
//! It exports the root module(a struct of function pointers) required by the
//! `example_0_interface`(the `interface crate`).

use abi_stable::{
    export_root_module,
    external_types::crossbeam_channel::RSender,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    sabi_trait::prelude::TD_Opaque,
    std_types::{ROk, RResult, RStr, RString, RVec},
};

use example_1_interface::{
    utils::process_command, ApplicationMut, AsyncCommand, CommandDescription, Error as AppError,
    Plugin, PluginId, PluginMod, PluginMod_Ref, PluginType, Plugin_TO, WhichPlugin,
};

use core_extensions::SelfOps;

use serde::{Deserialize, Serialize};

use serde_json::value::Value;

///////////////////////////////////////////////////////////////////////////////////

/// Exports the root module of this library.
///
/// This code isn't run until the layout of the type it returns is checked.
#[export_root_module]
fn instantiate_root_module() -> PluginMod_Ref {
    PluginMod { new }.leak_into_prefix()
}

//////////////////////////////////////////////////////////////////////////////////////

#[sabi_extern_fn]
pub fn new(_sender: RSender<AsyncCommand>, plugin_id: PluginId) -> RResult<PluginType, AppError> {
    let this = CommandUtils { plugin_id };
    ROk(Plugin_TO::from_value(this, TD_Opaque))
}

//////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub enum UtilCmd {
    Repeat {
        how_much: usize,
        plugin: WhichPlugin,
        command: Value,
    },
    Batch {
        plugin: WhichPlugin,
        commands: Vec<Value>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReturnValue {
    Repeat,
    Batch,
}

//////////////////////////////////////////////////////////////////////////////////////

fn run_command_inner(
    this: &mut CommandUtils,
    command: UtilCmd,
    mut app: ApplicationMut<'_>,
) -> Result<ReturnValue, AppError> {
    match command {
        UtilCmd::Repeat {
            how_much,
            plugin,
            command,
        } => {
            let s = serde_json::to_string(&command).unwrap();
            for _ in 0..how_much {
                app.send_command_to_plugin(this.plugin_id(), plugin.clone(), s.clone().into())
                    .into_result()?;
            }
            ReturnValue::Repeat
        }
        UtilCmd::Batch { plugin, commands } => {
            for command in commands {
                let s = serde_json::to_string(&command).unwrap();
                app.send_command_to_plugin(this.plugin_id(), plugin.clone(), s.into())
                    .into_result()?;
            }
            ReturnValue::Batch
        }
    }
    .piped(Ok)
}

//////////////////////////////////////////////////////////////////////////////////////

struct CommandUtils {
    plugin_id: PluginId,
}

impl Plugin for CommandUtils {
    fn json_command(
        &mut self,
        command: RStr<'_>,
        app: ApplicationMut<'_>,
    ) -> RResult<RString, AppError> {
        process_command(self, command, |this, command| {
            run_command_inner(this, command, app)
        })
    }

    fn plugin_id(&self) -> &PluginId {
        &self.plugin_id
    }

    fn list_commands(&self) -> RVec<CommandDescription> {
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
",
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
        ]
        .into()
    }

    fn close(self, _app: ApplicationMut<'_>) {}
}
