use std::{
    collections::{HashMap, VecDeque},
    io, mem,
    path::{Path, PathBuf},
    sync::Arc,
};

use abi_stable::{
    external_types::crossbeam_channel::{self, RReceiver, RSender},
    library::{lib_header_from_path, LibraryError, LibrarySuffix, RawLibrary},
    sabi_trait::prelude::TD_Opaque,
    std_types::{RErr, ROk, RResult, RSome, RStr, RString, RVec},
};

#[allow(unused_imports)]
use core_extensions::{SelfOps, SliceExt, StringExt};

use example_1_interface::{
    Application, Application_TO, AsyncCommand, Error as AppError, PluginId, PluginMod_Ref,
    PluginResponse, PluginType, WhichPlugin,
};

use serde::Deserialize;

use serde_json::value::RawValue;

use smallvec::SmallVec;

mod app;
mod vec_from_map;

use crate::{
    app::{ApplicationState, TheApplication},
    vec_from_map::VecFromMap,
};

/// Returns the path the plugin will be loaded from.
fn compute_plugin_path(base_name: &str) -> io::Result<PathBuf> {
    let debug_dir = "../../../target/debug/"
        .as_ref_::<Path>()
        .into_::<PathBuf>();
    let release_dir = "../../../target/release/"
        .as_ref_::<Path>()
        .into_::<PathBuf>();

    let debug_path = RawLibrary::path_in_directory(&debug_dir, base_name, LibrarySuffix::NoSuffix);

    let release_path =
        RawLibrary::path_in_directory(&release_dir, base_name, LibrarySuffix::NoSuffix);

    match (debug_path.exists(), release_path.exists()) {
        (false, false) => debug_path,
        (true, false) => debug_path,
        (false, true) => release_path,
        (true, true) => {
            if debug_path.metadata()?.modified()? < release_path.metadata()?.modified()? {
                release_path
            } else {
                debug_path
            }
        }
    }
    .piped(Ok)
}

/// A description of what plugin to load.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PluginToLoad {
    Named(String),
    WithInstances {
        #[serde(alias = "name")]
        named: String,
        #[serde(default = "one_u64")]
        instances: u64,
        #[serde(alias = "renamed")]
        rename: Option<String>,
    },
}

fn one_u64() -> u64 {
    1
}

/// The type that the configuration file is deserialized into.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub plugins: RVec<PluginToLoad>,
    pub commands: VecFromMap<WhichPlugin, Box<RawValue>>,
}

/// A description of plugin instances that were instantiated and left to instantiate,
/// as well as the root module of the plugin's dynamic library to instantiate the plugins.
pub struct PluginModAndIndices {
    root_module: PluginMod_Ref,
    to_be_instantiated: u64,
    indices: Vec<usize>,
}

pub type PluginIndices = SmallVec<[usize; 1]>;

/// Commands sent to plugins after calling `Application::send_command_to_plugin`.
#[derive(Debug)]
pub struct DelayedCommand {
    /// Which plugin sent the command
    from: PluginId,
    /// The index in plugins to which the command is sent.
    plugin_index: usize,
    /// The command for the `plugin_index` plugin.
    command: Arc<RString>,
}

/// Used to handle the responses to the delayed commands sent to plugins after calling
/// `Application::send_command_to_plugin`.
#[derive(Debug)]
pub struct DelayedResponse {
    /// The plugin that sends the reponse.
    from: PluginId,
    /// The plugin that sent the command for which this is the reponse.
    to: usize,
    /// The response from the `from` plugin to the `to` plugin.
    response: RString,
}

fn main() -> io::Result<()> {
    let config_path = match std::env::args_os().nth(1) {
        Some(os) => PathBuf::from(os),
        None => {
            println!("Help:You can pass a configuration's path as a command-line argument.");
            PathBuf::from("./data/app_config.json")
        }
    };

    let file_contents = match std::fs::read_to_string(&*config_path) {
        Ok(x) => x,
        Err(e) => {
            eprintln!(
                "Could not load the configuration file at:\n\
                \t{}\n\
                Because of this error:\n{}\n",
                config_path.display(),
                e
            );
            std::process::exit(1);
        }
    };

    let config: Config = match serde_json::from_str(&file_contents) {
        Ok(x) => x,
        Err(e) => {
            eprintln!(
                "Could not parse the configuration file at:\n\
                 \t{}\n\
                 Because of this error:\n\
                 {}\n",
                config_path.display(),
                e
            );
            std::process::exit(1);
        }
    };

    let mut nonexistent_files = Vec::<(String, io::Error)>::new();

    let mut library_errs = Vec::<(String, LibraryError)>::new();

    let mut loaded_libraries = Vec::<String>::new();

    let mut plugins = Vec::new();
    let mut state = ApplicationState::new();

    for plug in &config.plugins {
        let (named, instances, rename) = match plug {
            PluginToLoad::Named(named) => ((*named).clone(), 1, None),
            PluginToLoad::WithInstances {
                named,
                instances,
                rename,
            } => ((*named).clone(), *instances, rename.clone()),
        };

        let name_key = rename.unwrap_or_else(|| named.clone());

        if let Some(mod_i) = state.id_map.get_mut(&*name_key) {
            mod_i.to_be_instantiated += instances;
            continue;
        }

        let library_path: PathBuf = match compute_plugin_path(named.as_ref()) {
            Ok(x) => x,
            Err(e) => {
                nonexistent_files.push((named.clone(), e));
                continue;
            }
        };

        let res = (|| {
            let header = lib_header_from_path(&library_path)?;
            header.init_root_module::<PluginMod_Ref>()
        })();

        let root_module = match res {
            Ok(x) => x,
            Err(e) => {
                library_errs.push((named.clone(), e));
                continue;
            }
        };

        loaded_libraries.push(name_key.clone());

        state.id_map.insert(
            name_key.into_::<RString>(),
            PluginModAndIndices {
                root_module,
                to_be_instantiated: instances,
                indices: Vec::with_capacity(instances as usize),
            },
        );
    }

    if !nonexistent_files.is_empty() {
        for (name, e) in nonexistent_files {
            eprintln!(
                "Could not load librarr:\n\
                 \t{}\n\
                 because of this error:\n\
                 {}\n\
                ",
                name, e
            )
        }
        std::process::exit(1);
    }

    if !library_errs.is_empty() {
        for (name, e) in library_errs {
            eprintln!(
                "Could not load librarr:\n\
                 \t{}\n\
                 because of this error:\n\
                 {}\n\
                ",
                name, e
            )
        }
        std::process::exit(1);
    }

    let mut plugin_new_errs = Vec::<(String, AppError)>::new();

    for name in loaded_libraries {
        let mod_i = state.id_map.get_mut(&*name).unwrap();
        for _ in 0..mem::replace(&mut mod_i.to_be_instantiated, 0) {
            let plugin_constructor = mod_i.root_module.new();
            let new_id = PluginId {
                named: name.clone().into(),
                instance: mod_i.indices.len() as u64,
            };
            let plugin = match plugin_constructor(state.sender.clone(), new_id.clone()) {
                ROk(x) => x,
                RErr(e) => {
                    plugin_new_errs.push((name.clone(), e));
                    continue;
                }
            };

            let new_index = plugins.len();

            plugins.push(plugin);

            mod_i.indices.push(new_index);
            state.plugin_ids.push(new_id);
        }
    }

    if !plugin_new_errs.is_empty() {
        for (name, e) in plugin_new_errs {
            eprintln!(
                "Could not instantiate plugin:\n\
                 \t{}\n\
                 because of this error:\n\
                 {}\n\
                ",
                name, e
            )
        }
        std::process::exit(1);
    }

    let mut config_commands = config.commands.vec.into_iter();

    let mut app = TheApplication { plugins, state };

    while !app.is_finished() {
        if let Some((which_plugin, command)) = config_commands.next() {
            let command = command.get();
            if let Err(e) = app.run_command(which_plugin.clone(), command.into()) {
                eprintln!(
                    "Error while running command on:\n{:?}\nError:{}\nCommand:\n{:?}\n",
                    which_plugin, e, command
                );
            }
        }

        if let Err(e) = app.tick() {
            eprintln!("Error in application loop:\n{}\n", e);
        }
    }

    if app.is_finished() {
        println!("timeout waiting for events");
    }

    Ok(())
}
