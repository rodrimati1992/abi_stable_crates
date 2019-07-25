/*!
This is an example `interface crate`,
where all publically available modules(structs of function pointers) and types are declared,

To load the library and the modules together,
call `<PluginMod as RootModule>::load_from_directory`,
which will load the dynamic library from a directory(folder),
and all the modules inside of the library.


*/

use abi_stable::{
    StableAbi,
    sabi_trait,
    package_version_strings,
    declare_root_module_statics,
    library::RootModule,
    sabi_types::VersionStrings,
    external_types::{
        crossbeam_channel::RSender,
    },
    std_types::{RBox, RCow, RVec, RStr, RString,RResult, ROption, ROk,RSome},
};

use serde::{Serialize,Deserialize};

mod commands;
mod error;
mod which_plugin;
mod vec_from_map;
pub mod utils;


pub use self::{
    commands::{
        BasicCommand,BasicRetVal,CommandDescription,CommandTrait,WhichVariant,AsyncCommand,
    },
    error::{Error,Unsupported},
    which_plugin::WhichPlugin,
    vec_from_map::VecFromMap,
};


///////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Debug,Clone,PartialEq,Eq,StableAbi,Serialize,Deserialize)]
pub struct PluginId{
    pub named:RCow<'static,str>,
    /// The number of the instance of this Plugin.
    pub instance:u64,
}


#[repr(u8)]
#[derive(Debug,Clone,PartialEq,Eq,StableAbi,Serialize,Deserialize)]
pub enum WhichCommandRet{
    Command,
    Return,
}


#[repr(C)]
#[derive(Debug,Clone,PartialEq,Eq,StableAbi)]
pub struct PluginResponse<'a>{
    pub plugin_id:PluginId,
    pub response:RCow<'a,str>,
}


impl<'a> PluginResponse<'a>{
    pub fn owned_response(plugin_id:PluginId,response:RString)->Self{
        Self{plugin_id,response:response.into()}
    }
    pub fn borrowed_response(plugin_id:PluginId,response:RStr<'a>)->Self{
        Self{plugin_id,response:response.into()}
    }
}



///////////////////////////////////////////////////////////////////////////////


pub type PluginType=Plugin_TO<'static,RBox<()>>;


/**
A plugin which is loaded by the application,and provides some functionality.
*/
#[sabi_trait]
//#[sabi(debug_print)]
pub trait Plugin {

    /// Handles a JSON encoded command.
    fn json_command(
        &mut self,
        command: RStr<'_>,
        app:ApplicationMut<'_>,
    )->RResult<RString,Error>;

    /// Handles a response from another Plugin,
    /// from having called `ApplicationMut::send_command_to_plugin` ealier.
    fn handle_response<'a>(
        &mut self,
        response:PluginResponse<'a>,
        _app:ApplicationMut<'_>,
    )->RResult<ROption<PluginResponse<'a>>,Error>{
        ROk(RSome(response))
    }

    /// Gets the PluginId that was passed to this plugin in its constructor.
    fn plugin_id(&self)->&PluginId;

    /// Gets a description of all commands from this Plugin.
    fn list_commands(&self)->RVec<CommandDescription>;

    /// Closes the plugin,
    ///
    /// This does not unload the dynamic library of this plugin,
    /// you can instantiate another instance of this plugin with 
    /// `PluginMod::get_module().new()(application_handle)`.
    #[sabi(last_prefix_field)]
    fn close(self,app:ApplicationMut<'_>);
}


///////////////////////////////////////////////////////////////////////////////

/// The root module of a`plugin` dynamic library.
///
/// To load this module,
/// call <PluginMod as RootModule>::load_from_directory(some_directory_path)
#[repr(C)]
#[derive(StableAbi)] 
#[sabi(kind(Prefix(prefix_struct="PluginMod")))]
#[sabi(missing_field(panic))]
pub struct PluginModVal {
    #[sabi(last_prefix_field)]
    /// Constructs the plugin.
    pub new: extern "C" fn(RSender<AsyncCommand>,PluginId) -> RResult<PluginType,Error>,
}


impl RootModule for PluginMod {
    declare_root_module_statics!{PluginMod}
    const BASE_NAME: &'static str = "plugin";
    const NAME: &'static str = "plugin";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}



///////////////////////////////////////////////////////////////////////////////


/// A mutable reference to the application implementation.
pub type ApplicationMut<'a>=Application_TO<'a,&'a mut ()>;


#[sabi_trait]
pub trait Application{

    /// Asynchronously Sends a command to the plugin(s) specified by `which_plugin`.
    ///
    /// # Errors
    ///
    /// Returns an `Error::InvalidPlugin` if `which_plugin` is invalid.
    fn send_command_to_plugin(
        &mut self,
        from:&PluginId,
        which_plugin:WhichPlugin,
        command:RString,
    )->RResult<(),Error>;

    /// Gets the `PluginId`s of the plugins specified by `which_plugin`.
    fn get_plugin_id(&self,which_plugin:WhichPlugin)->RResult<RVec<PluginId>,Error>;

    /// Gets the sender end of a channel to send commands to the application/other plugins.
    fn sender(&self)->RSender<AsyncCommand>;

    /// Gets the PluginId of all loaded plugins
    fn loaded_plugins(&self)->RVec<PluginId>;
}