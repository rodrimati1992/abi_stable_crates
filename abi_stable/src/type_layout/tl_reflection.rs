use crate::std_types::RStr;


////////////////////////////////////////////////////////////////////////////////

/// Whether a field is accessible,and how it is accessed.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub enum FieldAccessor {
    /// Accessible with `self.field_name`
    Direct,
    /// Accessible with `fn field_name(&self)->FieldType`
    Method,
    /// Accessible with `fn name(&self)->FieldType`
    MethodNamed{
        name:RStr<'static>,
    },
    /// Accessible with `fn field_name(&self)->Option<FieldType>`
    MethodOption,
    /// This field is completely inaccessible.
    Opaque,
}


impl FieldAccessor{
    /// Constructs a FieldAccessor for a method named `name`.
    pub const fn method_named(name:RStr<'static>)->Self{
        FieldAccessor::MethodNamed{name}
    }
}


////////////////////////////////////////////////////////////////////////////////


abi_stable_shared::declare_comp_field_accessor!{
    attrs=[ 
        derive(StableAbi),
        sabi(unsafe_sabi_opaque_fields),
    ]
}


impl CompFieldAccessor{

    pub fn expand(self,string:&'static str)->Option<FieldAccessor>{
        Some(match self {
            Self::DIRECT=>
                FieldAccessor::Direct,
            Self::METHOD=>
                FieldAccessor::Method,
            Self::METHOD_NAMED=>
                FieldAccessor::MethodNamed{name:string.into()},
            Self::METHOD_OPTION=>
                FieldAccessor::MethodOption,
            Self::OPAQUE=>
                FieldAccessor::Opaque,
            _=>return None,
        })
    }
}


////////////////////////////////////////////////////////////////////////////////



