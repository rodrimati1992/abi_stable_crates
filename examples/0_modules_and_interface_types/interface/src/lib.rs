/*!
This is an example `interface crate`,
where all publically available modules(structs of function pointers) and types are declared,

To load the library and the modules together,
call `<TextOpsMod as RootModule>::load_from_directory`,
which will load the dynamic library from a directory(folder),
and all the modules inside of the library.


*/


use abi_stable::{
    StableAbi,
    external_types::{RawValueRef,RawValueBox},
    package_version_strings,
    declare_root_module_statics,
    library::RootModule,
    type_level::bools::*,
    erased_types::{
        InterfaceType,DeserializeDyn,SerializeProxyType,IteratorItem
    },
    DynTrait,
    sabi_types::VersionStrings,
    std_types::{RBox, RStr, RString,RArc,RCow,RBoxError,RResult},
};




/// The root module of the `text_operations` dynamic library.
/// With all the functions/modules related to processing text.
///
/// To load this module,
/// call <TextOpsMod as RootModule>::load_from_directory(some_directory_path)
#[repr(C)]
#[derive(StableAbi)] 
#[sabi(kind(Prefix(prefix_struct="TextOpsMod")))]
#[sabi(missing_field(panic))]
pub struct TextOpsModVal {
    /// Constructs TOStateBox,state that is passed to other functions in this module.
    pub new: extern "C" fn() -> TOStateBox,

    #[sabi(last_prefix_field)]    
    pub deserializers:&'static DeserializerMod,

    /// Reverses the order of the lines.
    pub reverse_lines: extern "C" fn(&mut TOStateBox,RStr<'_>) -> RString,
    
    /// Removes the `param.words` words from the `param.string` string.
    pub remove_words: 
        extern "C" fn(&mut TOStateBox,param:RemoveWords<'_,'_>) -> RString,
    
    /// Gets the ammount (in bytes) of text that was processed
    pub get_processed_bytes: extern "C" fn(&TOStateBox) -> u64,
 
    pub run_command: 
        extern "C" fn(&mut TOStateBox,command:TOCommandBox<'static>)->TOReturnValueArc,
}


impl RootModule for TextOpsMod {
    declare_root_module_statics!{TextOpsMod}
    const BASE_NAME: &'static str = "text_operations";
    const NAME: &'static str = "text_operations";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}


/// A module for all deserialization functions.
#[repr(C)]
#[derive(StableAbi)] 
#[sabi(kind(Prefix(prefix_struct="DeserializerMod")))]
#[sabi(missing_field(panic))]
pub struct DeserializerModVal {
    pub something: std::marker::PhantomData<()>,
    // pub something: std::marker::PhantomData<abi_stable::std_types::RHashMap<RString,RString>>,
    #[sabi(last_prefix_field)]
    /// The implementation for how TOStateBox is going to be deserialized.
    pub deserialize_state: extern "C" fn(RStr<'_>) -> RResult<TOStateBox, RBoxError>,

    /// The implementation for how TOCommandBox is going to be deserialized.
    pub deserialize_command: 
        for<'a> extern "C" fn(RStr<'a>) -> RResult<TOCommandBox<'static>, RBoxError>,
    
    /// The implementation for how TOCommandBox is going to be deserialized,
    /// borrowing from the input string.
    pub deserialize_command_borrowing: 
        for<'borr> extern "C" fn(RStr<'borr>) -> RResult<TOCommandBox<'borr>, RBoxError>,
    
    /// The implementation for how TOReturnValueArc is going to be deserialized.
    pub deserialize_return_value: extern "C" fn(RStr<'_>) -> RResult<TOReturnValueArc, RBoxError>,
}





///////////////////////////////////////////////


/// An `InterfaceType` describing which traits are implemented by TOStateBox.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Serialize,Deserialize,PartialEq))]
pub struct TOState;

/// The state passed to most functions in the TextOpsMod module.
pub type TOStateBox = DynTrait<'static,RBox<()>,TOState>;


/// First <ConcreteType as DeserializeImplType>::serialize_impl returns 
/// a RawValueBox containing the serialized data,
/// then the returned RawValueBox is serialized.
impl SerializeProxyType for TOState{
    type Proxy=RawValueBox;
}


impl<'borr> DeserializeDyn<'borr,TOStateBox> for TOState {
    type Proxy = RawValueRef<'borr>;

    fn deserialize_dyn(s: RawValueRef<'borr>) -> Result<TOStateBox, RBoxError> {
        TextOpsMod::get_module().unwrap()
            .deserializers()
            .deserialize_state()(s.get_rstr())
            .into_result()
    }
}


///////////////////////////////////////////////////////////////////////////////


/// An `InterfaceType` describing which traits are implemented by TOCommandBox.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send,Sync,Debug,Serialize,Deserialize,PartialEq,Iterator))]
pub struct TOCommand;

/// A de/serializable opaque command enum,used in the TextOpsMod::run_command function.
pub type TOCommandBox<'borr> = DynTrait<'borr,RBox<()>,TOCommand>;

impl<'a> IteratorItem<'a> for TOCommand{
    type Item=&'a mut RString;
}


/// First <ConcreteType as DeserializeImplType>::serialize_impl returns 
/// a RawValueBox containing the serialized data,
/// then the returned RawValueBox is serialized.
impl SerializeProxyType for TOCommand{
    type Proxy=RawValueBox;
}

/// Describes how TOCommandBox is deserialized
impl<'borr> DeserializeDyn<'borr,TOCommandBox<'static>> for TOCommand {
    type Proxy = RawValueRef<'borr> ;

    fn deserialize_dyn(s: RawValueRef<'borr>) -> Result<TOCommandBox<'static>, RBoxError> {
        TextOpsMod::get_module().unwrap()
            .deserializers()
            .deserialize_command()(s.get_rstr())
            .into_result()
    }
}


///////////////////////////////////////////////////////////////////////////////


/// An `InterfaceType` describing which traits are implemented by TOReturnValueArc.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Sync,Send,Debug,Serialize,Deserialize,PartialEq))]
pub struct TOReturnValue;

/// A de/serializable opaque command enum,returned by the TextOpsMod::run_command function.
pub type TOReturnValueArc = DynTrait<'static,RArc<()>,TOReturnValue>;


/// First <ConcreteType as DeserializeImplType>::serialize_impl returns 
/// a RawValueBox containing the serialized data,
/// then the returned RawValueBox is serialized.
impl SerializeProxyType for TOReturnValue{
    type Proxy=RawValueBox;
}

/// Describes how TOCommandBox is deserialized
impl<'borr> DeserializeDyn<'borr,TOReturnValueArc> for TOReturnValue {
    type Proxy = RawValueRef<'borr>;

    fn deserialize_dyn(s: RawValueRef<'borr>) -> Result<TOReturnValueArc, RBoxError> {
        TextOpsMod::get_module().unwrap()
            .deserializers()
            .deserialize_return_value()(s.get_rstr())
            .into_result()
    }
}


///////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Sync,Send,Iterator))]
pub struct CowStrIter;

impl<'a> IteratorItem<'a> for CowStrIter{
    type Item=RCow<'a,str>;
}



/// The parameters for every `TextOpsMod.remove_words_*` function.
#[repr(C)]
#[derive(StableAbi)] 
pub struct RemoveWords<'a,'b>{
    /// The string we're processing.
    pub string:RStr<'a>,
    /// The words that will be removed from self.string.
    ///
    /// An iterator over `RCow<'a,str>`,
    /// constructed from a `&'b mut impl Iterator<RCow<'a,str>>`
    /// with `DynTrait::from_borrowing_ptr(iter,CowStrIter)`.
    pub words:DynTrait<'a,&'b mut (),CowStrIter>,
}


///////////////////////////////////////////////////////////////////////////////
