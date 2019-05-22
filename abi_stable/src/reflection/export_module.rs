/*!
Data structures to export a type as though it were a module.
*/

use std::{
    fmt::{self,Display},
};


use crate::{
    reflection::{ModReflMode},
    abi_stability::{
        type_layout::*,
    },
};

#[derive(Debug,Serialize,Deserialize)]
pub struct MRItem{
    item_name:String,
    type_:String,
    field_accessor:MRFieldAccessor,
    #[serde(flatten)]
    variant:MRItemVariant,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct MRNameType{
    name:String,
    type_:String,
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(tag="variant")]
pub enum MRItemVariant{
    Function(MRFunction),
    Module(MRModule),
    Static,
}


#[derive(Debug,Serialize,Deserialize)]
pub struct MRFunction{
    params:Vec<MRNameType>,
    returns:Option<MRNameType>,
}


#[derive(Debug,Serialize,Deserialize)]
pub struct MRModule{
    mod_refl_mode:MRModReflMode,
    items:Vec<MRItem>,
}

#[derive(Debug,Serialize,Deserialize)]
pub enum MRModReflMode{
    Module,
    Opaque,
    DelegateDeref,
}

#[repr(u8)]
#[derive(Debug,Serialize,Deserialize)]
pub enum MRFieldAccessor {
    /// Accessible with `self.field_name`
    Direct,
    /// Accessible with `fn field_name(&self)->FieldType`
    Method{
        name:Option<String>,
    },
    /// Accessible with `fn field_name(&self)->Option<FieldType>`
    MethodOption,
    /// This field is completely inaccessible.
    Opaque,
}


impl MRItem{
    pub fn from_abi_info(
        layout:&'static TypeLayout,
    )->Self{
        let type_=layout.full_type.to_string();

        let variant=Self::get_item_variant(layout);

        Self{
            item_name:"root".into(),
            type_,
            field_accessor:MRFieldAccessor::Direct,
            variant,
        }
    }

    fn get_item_variant(layout:&'static TypeLayout)->MRItemVariant {
        match layout.mod_refl_mode {
            ModReflMode::Module=>{
                let fields=match layout.data {
                    TLData::Primitive{..}=>
                        return MRItemVariant::Static,
                    TLData::Struct { fields }=>
                        fields.as_slice(),
                    TLData::Enum {..}=>
                        return MRItemVariant::Static,
                    TLData::PrefixType(prefix)=>
                        prefix.fields.as_slice(),
                };

                let items=fields.iter()
                    .filter(|f| f.field_accessor!=FieldAccessor::Opaque )
                    .map(|field|{
                        let (type_,variant)=if field.is_function {
                            let func=MRFunction::from(&field.functions[0]);
                            (
                                func.to_string(),
                                MRItemVariant::Function(func),
                            )
                        }else{
                            let layout=field.abi_info.get().layout;
                            (
                                layout.full_type.to_string(),
                                Self::get_item_variant(layout),
                            )
                        };
                        MRItem{
                            item_name:field.name.to_string(),
                            type_,
                            field_accessor:field.field_accessor.into(),
                            variant,
                        }
                    })
                    .collect::<Vec<_>>();
                MRItemVariant::Module(MRModule{
                    mod_refl_mode:layout.mod_refl_mode.into(),
                    items,
                })
            }
            ModReflMode::Opaque=>
                MRItemVariant::Static,
            ModReflMode::DelegateDeref{phantom_field_index}=>{
                let delegate_to=layout.phantom_fields[phantom_field_index];
                let inner_layout=delegate_to.abi_info.get().layout;
                Self::get_item_variant(inner_layout)
            }
        }
    }
}


///////////////////////////////////////////////////////////////////////////////


impl<'a> From<&'a TLFunction> for MRFunction{
    fn from(this:&'a TLFunction)->Self{
        Self{
            params:this.params.iter().map(MRNameType::from).collect::<Vec<_>>(),
            returns:this.returns.as_ref().map(MRNameType::from).into_option() ,
        }
    }
}

impl Display for MRFunction{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        write!(f,"fn(")?;
        let param_count=self.params.len();
        for (param_i,param) in self.params.iter().enumerate() {
            Display::fmt(param,f)?;
            if param_i+1!=param_count {
                Display::fmt(&", ",f)?;
            }
        }
        write!(f,")")?;
        if let Some(returns)=&self.returns {
            Display::fmt(&"->",f)?;
            Display::fmt(returns,f)?;
        }
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////


impl<'a> From<&'a TLField> for MRNameType{
    fn from(field:&'a TLField)->Self{
        let name=field.name.to_string();
        let type_=if field.is_function{
            field.functions[0].to_string()
        }else{
            field.abi_info.get().layout.full_type.to_string()
        };

        Self{
            name,
            type_,
        }
    }
}

impl Display for MRNameType{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        write!(f,"{}:{}",self.name,self.type_)
    }
}

///////////////////////////////////////////////////////////////////////////////


impl From<ModReflMode> for MRModReflMode{
    fn from(this:ModReflMode)->Self{
        match this {
            ModReflMode::Module{..}=>
                MRModReflMode::Module,
            ModReflMode::Opaque{..}=>
                MRModReflMode::Opaque,
            ModReflMode::DelegateDeref{..}=>
                MRModReflMode::DelegateDeref,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////


impl From<FieldAccessor> for MRFieldAccessor{
    fn from(this:FieldAccessor)->MRFieldAccessor{
        match this{
            FieldAccessor::Direct=>
                MRFieldAccessor::Direct,
            FieldAccessor::Method{name}=>
                MRFieldAccessor::Method{
                    name:name.map(|s| s.to_string() ).into_option(),
                },
            FieldAccessor::MethodOption=>
                MRFieldAccessor::MethodOption,
            FieldAccessor::Opaque=>
                MRFieldAccessor::Opaque,
        }
    }
}