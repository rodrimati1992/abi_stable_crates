use crate::{PluginId,WhichPlugin};

use std::{
    fmt,
};

use serde::{
    Serialize,Deserialize,
    de::{self,Deserializer, DeserializeOwned, IgnoredAny, Visitor, MapAccess, Error as _},
};

use abi_stable::{
    StableAbi,
    std_types::*,
};

/// The commands that map to methods in the Plugin trait.
// This is intentionally not `#[derive(StableAbi)]`,
// since it can be extended in minor versions of the interface.
// I has to be serialized to pass it through ffi.
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub enum BasicCommand{
    GetCommands,
}


/// These is the (serialized) return value of calling `PluginExt::send_basic_command`.
// This is intentionally not `#[derive(StableAbi)]`,
// since it can be extended in minor versions of the interface.
// I has to be serialized to pass it through ffi.
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub enum BasicRetVal{
    GetCommands(RVec<CommandDescription>),
}


// This is intentionally not `#[derive(StableAbi)]`,
// since it can be extended in minor versions of the interface.
// I has to be serialized to pass it through ffi.
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(untagged)]
pub enum CommandUnion<T>{
    ForPlugin(T),
    Basic(BasicCommand),
}


#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
#[serde(untagged)]
pub enum ReturnValUnion<T>{
    ForPlugin(T),
    Basic(BasicRetVal),
}



////////////////////////////////////////////////////////////////////////////////


/// A partially deserialize command,that only deserialized its variant.
#[derive(Debug,Clone)]
pub struct WhichVariant{
    pub variant:RString,
}


struct WhichVariantVisitor;

impl<'de> Visitor<'de> for WhichVariantVisitor{
    type Value = WhichVariant;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map with a single entry,or a string")
    }

    fn visit_str<E>(self, value: &str) -> Result<WhichVariant, E>
    where
        E: de::Error,
    {
        Ok(WhichVariant{variant:value.to_string().into()})
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let (variant,_)=access.next_entry::<RString,IgnoredAny>()?
            .ok_or_else(||M::Error::custom("Expected a map with a single entry"))?;
        if let Some((second,_))=access.next_entry::<RString,IgnoredAny>()? {
            let s=format!(
                "Expected a map with a single field,\n\
                 instead found both {{ \"{}\":... , \"{}\": ... }}",
                 variant,
                 second,
            );
            return Err(M::Error::custom(s));
        }
        Ok(WhichVariant{variant})
    }
}

impl<'de> Deserialize<'de> for WhichVariant{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(WhichVariantVisitor)
    }
}




////////////////////////////////////////////////////////////////////////////////


/// Denotes this as a command type.
pub trait CommandTrait:Serialize{
    type Returns:DeserializeOwned;
}

impl CommandTrait for BasicCommand{
    type Returns=BasicRetVal;
}


/// Describes a command.
#[repr(C)]
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize,StableAbi)]
pub struct CommandDescription{
    /// A description of what this command does.
    pub name:RCow<'static,str>,
    /// A description of what this command does,
    /// optionally with a description of the command format.
    pub description:RCow<'static,str>,
}


impl CommandDescription{
    pub fn from_literals(
        name:&'static str,
        description:&'static str,
    )->Self{
        CommandDescription{
            name:name.into(),
            description:description.into(),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(Debug,Clone,PartialEq,Eq,StableAbi)]
pub struct AsyncCommand{
    pub from:PluginId,
    pub which_plugin:WhichPlugin,
    pub command:RString,
}
