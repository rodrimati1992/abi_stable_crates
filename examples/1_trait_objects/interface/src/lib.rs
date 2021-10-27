//! This is an example `interface crate`,
//! where all publically available modules(structs of function pointers) and types are declared,
//!
//! To load the library and the modules together,
//! call `<PluginMod_Ref as RootModule>::load_from_directory`,
//! which will load the dynamic library from a directory(folder),
//! and all the modules inside of the library.
//!
//!

use abi_stable::{
    declare_root_module_statics,
    external_types::crossbeam_channel::RSender,
    library::RootModule,
    package_version_strings, sabi_trait,
    sabi_types::{RMut, VersionStrings},
    std_types::{RBox, RCow, ROk, ROption, RResult, RSome, RStr, RString, RVec},
    StableAbi,
};

use serde::{Deserialize, Serialize};

mod commands;
mod error;
pub mod utils;
mod vec_from_map;
mod which_plugin;

pub use self::{
    commands::{
        AsyncCommand, BasicCommand, BasicRetVal, CommandDescription, CommandTrait, WhichVariant,
    },
    error::{Error, Unsupported},
    vec_from_map::VecFromMap,
    which_plugin::WhichPlugin,
};

///////////////////////////////////////////////////////////////////////////////

/// The identifier for a plugin.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi, Serialize, Deserialize)]
pub struct PluginId {
    pub named: RCow<'static, str>,
    /// The number of the instance of this Plugin.
    pub instance: u64,
}

/// Describes whether a boxed error is a command or a return value.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi, Serialize, Deserialize)]
pub enum WhichCommandRet {
    Command,
    Return,
}

/// The response from having called `ApplicationMut::send_command_to_plugin` ealier.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginResponse<'a> {
    /// The id of the plugin that is responding.
    pub plugin_id: PluginId,
    /// The response from the plugin
    pub response: RCow<'a, str>,
}

impl<'a> PluginResponse<'a> {
    pub fn owned_response(plugin_id: PluginId, response: RString) -> Self {
        Self {
            plugin_id,
            response: response.into(),
        }
    }
    pub fn borrowed_response(plugin_id: PluginId, response: RStr<'a>) -> Self {
        Self {
            plugin_id,
            response: response.into(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

pub type PluginType = Plugin_TO<'static, RBox<()>>;

/// A plugin which is loaded by the application,and provides some functionality.
#[sabi_trait]
//#[sabi(debug_print)]
pub trait Plugin {
    /// Handles a JSON encoded command.
    fn json_command(
        &mut self,
        command: RStr<'_>,
        app: ApplicationMut<'_>,
    ) -> RResult<RString, Error>;

    /// Handles a response from another Plugin,
    /// from having called `ApplicationMut::send_command_to_plugin` ealier.
    fn handle_response<'a>(
        &mut self,
        response: PluginResponse<'a>,
        _app: ApplicationMut<'_>,
    ) -> RResult<ROption<PluginResponse<'a>>, Error> {
        ROk(RSome(response))
    }

    /// Gets the PluginId that was passed to this plugin in its constructor.
    fn plugin_id(&self) -> &PluginId;

    /// Gets a description of all commands from this Plugin.
    fn list_commands(&self) -> RVec<CommandDescription>;

    /// Closes the plugin,
    ///
    /// This does not unload the dynamic library of this plugin,
    /// you can instantiate another instance of this plugin with
    /// `PluginMod_Ref::get_module().new()(application_handle)`.
    ///
    ///
    ///
    /// The `#[sabi(last_prefix_field)]` attribute here means that this is the last method
    /// that was defined in the first compatible version of the library
    /// (0.1.0, 0.2.0, 0.3.0, 1.0.0, 2.0.0 ,etc),
    /// requiring new methods to always be added below preexisting ones.
    ///
    /// The `#[sabi(last_prefix_field)]` attribute would stay on this method until the library
    /// bumps its "major" version,
    /// at which point it would be moved to the last method at the time.
    #[sabi(last_prefix_field)]
    fn close(self, app: ApplicationMut<'_>);
}

///////////////////////////////////////////////////////////////////////////////

/// The root module of a`plugin` dynamic library.
///
/// To load this module,
/// call <PluginMod as RootModule>::load_from_directory(some_directory_path)
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = "PluginMod_Ref")))]
#[sabi(missing_field(panic))]
pub struct PluginMod {
    /// Constructs the plugin.
    ///
    ///
    /// The `#[sabi(last_prefix_field)]` attribute here means that this is the last field in this struct
    /// that was defined in the first compatible version of the library
    /// (0.1.0, 0.2.0, 0.3.0, 1.0.0, 2.0.0 ,etc),
    /// requiring new fields to always be added below preexisting ones.
    ///
    /// The `#[sabi(last_prefix_field)]` attribute would stay on this field until the library
    /// bumps its "major" version,
    /// at which point it would be moved to the last field at the time.
    ///
    #[sabi(last_prefix_field)]
    pub new: extern "C" fn(RSender<AsyncCommand>, PluginId) -> RResult<PluginType, Error>,
}

impl RootModule for PluginMod_Ref {
    declare_root_module_statics! {PluginMod_Ref}
    const BASE_NAME: &'static str = "plugin";
    const NAME: &'static str = "plugin";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}

///////////////////////////////////////////////////////////////////////////////

/// A mutable reference to the application implementation.
pub type ApplicationMut<'a> = Application_TO<'a, RMut<'a, ()>>;

#[sabi_trait]
pub trait Application {
    /// Asynchronously Sends a command to the plugin(s) specified by `which_plugin`.
    ///
    /// # Errors
    ///
    /// Returns an `Error::InvalidPlugin` if `which_plugin` is invalid.
    fn send_command_to_plugin(
        &mut self,
        from: &PluginId,
        which_plugin: WhichPlugin,
        command: RString,
    ) -> RResult<(), Error>;

    /// Gets the `PluginId`s of the plugins specified by `which_plugin`.
    ///
    ///
    /// The `#[sabi(last_prefix_field)]` attribute here means that this is the last method
    /// that was defined in the first compatible version of the library
    /// (0.1.0, 0.2.0, 0.3.0, 1.0.0, 2.0.0 ,etc),
    /// requiring new methods to always be added below preexisting ones.
    ///
    /// The `#[sabi(last_prefix_field)]` attribute would stay on this method until the library
    /// bumps its "major" version,
    /// at which point it would be moved to the last method at the time.
    ///
    #[sabi(last_prefix_field)]
    fn get_plugin_id(&self, which_plugin: WhichPlugin) -> RResult<RVec<PluginId>, Error>;

    /// Gets the sender end of a channel to send commands to the application/other plugins.
    fn sender(&self) -> RSender<AsyncCommand>;

    /// Gets the PluginId of all loaded plugins
    fn loaded_plugins(&self) -> RVec<PluginId>;
}
