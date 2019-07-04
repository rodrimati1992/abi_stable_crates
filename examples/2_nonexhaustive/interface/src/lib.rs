use abi_stable::{
    nonexhaustive_enum::{NonExhaustiveFor,DeserializeOwned,GetEnumInfo,InterfaceBound},
    library::RootModule,
    sabi_types::{VersionStrings},
    std_types::{RBox,RString,RResult,RStr,RBoxError,RVec},
    type_level::{
        bools::True,
        impl_enum::{Implemented,Unimplemented,IsImplemented},
        trait_marker,
    },
    sabi_trait,
    StableAbi,InterfaceType,
    package_version_strings,
    declare_root_module_statics,
    impl_InterfaceType,
};

#[cfg(feature="v1_1")]
use serde::{Deserialize,Deserializer};

use serde_json::value::RawValue;


#[repr(transparent)]
#[cfg_attr(feature="v1_1",derive(Deserialize))]
#[derive(StableAbi,Debug,Clone,Copy,PartialEq)]
pub struct Cents{
    pub cents:u64,
}

#[repr(transparent)]
#[cfg_attr(feature="v1_1",derive(Deserialize))]
#[derive(StableAbi,Debug,Clone,Copy,PartialEq)]
pub struct ItemId{
    #[doc(hidden)]
    pub id:usize,
}


///////////////////////////////////////////////////////////////////////////////


#[repr(transparent)]
#[cfg(feature="v1_1")]
#[derive(StableAbi,Debug,Clone,PartialEq)]
pub struct SerdeWrapper<T>{
    pub inner:T,
}

#[cfg(feature="v1_1")]
impl<T> SerdeWrapper<T>{
    pub fn new(inner:T)->Self{
        Self{inner}
    }
}


#[cfg(feature="v1_1")]
impl<'de,T> Deserialize<'de> for SerdeWrapper<NonExhaustiveFor<T>>
where
    T: GetEnumInfo+'de,
    T::DefaultInterface: DeserializeOwned<T,T::DefaultStorage,T::DefaultInterface>,
    T::DefaultInterface: InterfaceBound<Deserialize=Implemented<trait_marker::Deserialize>>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <&RawValue>::deserialize(deserializer)?;
        NonExhaustiveFor::<T>::deserialize_owned_from_str(s.get())
            .map(|x| SerdeWrapper{inner:x} )
            .map_err(serde::de::Error::custom)
    }
}


///////////////////////////////////////////////////////////////////////////////


#[repr(u8)]
#[cfg_attr(feature="v1_1",derive(Deserialize))]
#[derive(StableAbi,Debug,Clone,PartialEq)]
#[sabi(kind(WithNonExhaustive(
    size="[usize;8]",
    traits(Debug,Clone,PartialEq,Deserialize),
)))]
pub enum Command{
    #[doc(hidden)]
    __NonExhaustive,
    CreateItem{
        name:RString,
        initial_count:u32,
        price:Cents,
    },
    DeleteItem{
        id:ItemId,
    },
    AddItem{
        id:ItemId,
        count:u32,
    },
    RemoveItem{
        id:ItemId,
        count:u32,
    },
    #[cfg(feature="v1_1")]
    RenameItem{
        id:ItemId,
        new_name:RString,
    },
    #[cfg(feature="v1_1")]
    Many{
        list:RVec<SerdeWrapper<NonExhaustiveFor<Command>>>
    },
}


impl DeserializeOwned<Command,Command_Storage,Command_Interface> for Command_Interface{
    fn deserialize_enum(s: RStr<'_>) -> Result<NonExhaustiveFor<Command>, RBoxError>{
        ShopMod::get_module().unwrap().deserialize_command()(s).into_result()
    }
}


///////////////////////////////////////////////////////////////////////////////


#[repr(u8)]
#[cfg_attr(feature="v1_1",derive(Deserialize))]
#[derive(StableAbi,Debug,Clone,PartialEq)]
#[sabi(kind(WithNonExhaustive(
    size=64,
    interface="Command_Interface",
)))]
pub enum ReturnVal{
    #[doc(hidden)]
    __NonExhaustive,
    CreateItem{
        count:u32,
        id:ItemId,
    },
    DeleteItem{
        id:ItemId,
    },
    AddItem{
        remaining:u32,
        id:ItemId,
    },
    RemoveItem{
        removed:u32,
        remaining:u32,
        id:ItemId,
    },
    #[cfg(feature="v1_1")]
    RenameItem{
        id:ItemId,
        new_name:RString,
        old_name:RString,
    },
    #[cfg(feature="v1_1")]
    Many{
        list:RVec<SerdeWrapper<NonExhaustiveFor<ReturnVal>>>
    },
}


impl DeserializeOwned<ReturnVal,ReturnVal_Storage,Command_Interface> for Command_Interface{
    fn deserialize_enum(s: RStr<'_>) -> Result<NonExhaustiveFor<ReturnVal>, RBoxError>{
        ShopMod::get_module().unwrap().deserialize_ret_val()(s).into_result()
    }
}


///////////////////////////////////////////////////////////////////////////////


#[repr(u8)]
#[derive(StableAbi,Debug,Clone,PartialEq)]
#[sabi(kind(WithNonExhaustive(
    size="[usize;6]",
    traits(Debug,Clone,PartialEq),
)))]
pub enum Error{
    #[doc(hidden)]
    __NonExhaustive,
    ItemAlreadyExists{
        id:ItemId,
        name:RString,
    },
    ItemIdNotFound{
        id:ItemId,
    },
    InvalidCommand{
        cmd:RBox<NonExhaustiveFor<Command>>,
    },
}


#[repr(C)]
#[derive(StableAbi)] 
#[sabi(kind(Prefix(prefix_struct="ShopMod")))]
#[sabi(missing_field(panic))]
pub struct ShopModVal {
    pub new:extern "C" fn()->Shop_TO<'static,RBox<()>>,

    pub deserialize_command:
        extern "C" fn(s:RStr<'_>)->RResult<NonExhaustiveFor<Command>,RBoxError>,

    #[sabi(last_prefix_field)]
    pub deserialize_ret_val:
        extern "C" fn(s:RStr<'_>)->RResult<NonExhaustiveFor<ReturnVal>,RBoxError>,
}


impl RootModule for ShopMod {
    declare_root_module_statics!{ShopMod}
    const BASE_NAME: &'static str = "shop";
    const NAME: &'static str = "shop";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}



#[sabi_trait]
pub trait Shop{
    #[sabi(last_prefix_field)]
    fn run_command(
        &mut self,
        cmd:NonExhaustiveFor<Command>,
    ) -> RResult<NonExhaustiveFor<ReturnVal>,NonExhaustiveFor<Error>>;
}
