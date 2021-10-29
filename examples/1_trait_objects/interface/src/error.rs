use crate::{commands::CommandDescription, WhichCommandRet, WhichPlugin};

use abi_stable::{
    std_types::{RBox, RBoxError, RString, RVec},
    StableAbi,
};

use std::{
    error::Error as ErrorTrait,
    fmt::{self, Display},
};

use core_extensions::strings::StringExt;

#[repr(u8)]
#[derive(Debug, StableAbi)]
pub enum Error {
    /// An error produced by `serde_json::to_string`.
    Serialize(RBoxError, WhichCommandRet),
    /// An error produced by `serde_json::from_string`.
    Deserialize(RBoxError, WhichCommandRet),
    /// A deserialization error produced when trying to deserialize json
    /// as a particular command type.
    UnsupportedCommand(RBox<Unsupported>),
    /// A deserialization error produced when trying to deserialize json
    /// as a particular return value type.
    UnsupportedReturnValue(RBox<Unsupported>),
    /// An invalid plugin.
    InvalidPlugin(WhichPlugin),
    /// A custom error.
    Custom(RBoxError),
    /// A list of errors.
    Many(RVec<Error>),
}

/// Represents a command or return value that wasn't supported.
#[repr(C)]
#[derive(Debug, StableAbi)]
pub struct Unsupported {
    /// The name of the plugin for which the command/return value wasn't supported.
    pub plugin_name: RString,
    /// The command/return value that wasn't supported.
    pub command_name: RString,
    /// A custom error.
    pub error: RBoxError,
    /// A list of the commands that the plugin supports
    pub supported_commands: RVec<CommandDescription>,
}

impl Error {
    pub fn unsupported_command(what: Unsupported) -> Self {
        Error::UnsupportedCommand(RBox::new(what))
    }
    pub fn unsupported_return_value(what: Unsupported) -> Self {
        Error::UnsupportedReturnValue(RBox::new(what))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Serialize(e, which) => {
                let which = match which {
                    WhichCommandRet::Command => "command",
                    WhichCommandRet::Return => "return value",
                };
                writeln!(
                    f,
                    "Error happened while serializing the {}:\n{}\n",
                    which, e
                )
            }
            Error::Deserialize(e, which) => {
                let which = match which {
                    WhichCommandRet::Command => "command",
                    WhichCommandRet::Return => "return value",
                };
                writeln!(f, "Error happened while deserializing {}:\n{}\n", which, e)
            }
            Error::UnsupportedCommand(v) => {
                writeln!(
                    f,
                    "Plugin '{}' ooes not support this command:\n\
                     \t'{}'\n\
                     Because of this error:\n{}\n\
                     Supported commands:\
                    ",
                    v.plugin_name, v.command_name, v.error,
                )?;

                for supported in &v.supported_commands {
                    write!(
                        f,
                        "{}",
                        format!(
                            "\nName:\n{}\nDescription:\n{}\n\n",
                            supported.name.left_padder(4),
                            supported.description.left_padder(4),
                        )
                        .left_padder(4)
                    )?;
                }

                Ok(())
            }
            Error::UnsupportedReturnValue(v) => writeln!(
                f,
                "Unrecognized return value from '{}',named:\n\
                     \t'{}'\n\
                     Because of this error:\n{}\n\
                    ",
                v.plugin_name, v.command_name, v.error,
            ),
            Error::InvalidPlugin(wc) => writeln!(
                f,
                "Attempted to access a nonexistent plugin with the WhichPlugin:\n\t{:?}\n",
                wc
            ),
            Error::Custom(e) => Display::fmt(e, f),
            Error::Many(list) => {
                for e in list {
                    writeln!(f, "{}", e)?;
                }
                Ok(())
            }
        }
    }
}

impl ErrorTrait for Error {}
