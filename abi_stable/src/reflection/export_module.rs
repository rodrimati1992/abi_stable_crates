//! Data structures to export a type as though it were a module.

use std::fmt::{self, Display};

use core_extensions::SelfOps;

use crate::{reflection::ModReflMode, type_layout::*};

#[derive(Debug, Serialize, Deserialize)]
pub struct MRItem {
    item_name: String,
    type_: String,
    field_accessor: MRFieldAccessor,
    #[serde(flatten)]
    variant: MRItemVariant,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MRNameType {
    name: String,
    type_: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "variant")]
pub enum MRItemVariant {
    Function(MRFunction),
    Module(MRModule),
    Static,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MRFunction {
    params: Vec<MRNameType>,
    returns: MRNameType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MRModule {
    mod_refl_mode: MRModReflMode,
    items: Vec<MRItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MRModReflMode {
    Module,
    Opaque,
    DelegateDeref,
}

#[repr(u8)]
#[derive(Debug, Serialize, Deserialize)]
pub enum MRFieldAccessor {
    /// Accessible with `self.field_name`
    Direct,
    /// Accessible with `fn field_name(&self)->FieldType`
    Method { name: Option<String> },
    /// Accessible with `fn field_name(&self)->Option<FieldType>`
    MethodOption,
    /// This field is completely inaccessible.
    Opaque,
}

impl MRItem {
    pub fn from_type_layout(layout: &'static TypeLayout) -> Self {
        let type_ = layout.full_type().to_string();

        let variant = Self::get_item_variant(layout);

        Self {
            item_name: "root".into(),
            type_,
            field_accessor: MRFieldAccessor::Direct,
            variant,
        }
    }

    fn get_item_variant(layout: &'static TypeLayout) -> MRItemVariant {
        match layout.mod_refl_mode() {
            ModReflMode::Module => {
                let fields = match layout.data() {
                    TLData::Struct { fields } => fields,
                    TLData::PrefixType(prefix) => prefix.fields,
                    TLData::Primitive { .. }
                    | TLData::Opaque { .. }
                    | TLData::Union { .. }
                    | TLData::Enum { .. } => return MRItemVariant::Static,
                };

                let items = fields
                    .iter()
                    .filter(|f| f.field_accessor() != FieldAccessor::Opaque)
                    .map(|field| {
                        let (type_, variant) = if field.is_function() {
                            let func = MRFunction::from(&field.function_range().index(0));
                            (func.to_string(), MRItemVariant::Function(func))
                        } else {
                            let layout = field.layout();
                            (
                                layout.full_type().to_string(),
                                Self::get_item_variant(layout),
                            )
                        };
                        MRItem {
                            item_name: field.name().to_string(),
                            type_,
                            field_accessor: field.field_accessor().into(),
                            variant,
                        }
                    })
                    .collect::<Vec<_>>();
                MRItemVariant::Module(MRModule {
                    mod_refl_mode: layout.mod_refl_mode().into(),
                    items,
                })
            }
            ModReflMode::Opaque => MRItemVariant::Static,
            ModReflMode::DelegateDeref { layout_index } => {
                let delegate_to = layout.shared_vars().type_layouts()[layout_index as usize];
                let inner_layout = delegate_to();
                Self::get_item_variant(inner_layout)
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<'a> From<&'a TLFunction> for MRFunction {
    fn from(this: &'a TLFunction) -> Self {
        Self {
            params: this.get_params().map(MRNameType::from).collect::<Vec<_>>(),
            returns: this.get_return().into_::<MRNameType>(),
        }
    }
}

impl Display for MRFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn(")?;
        let param_count = self.params.len();
        for (param_i, param) in self.params.iter().enumerate() {
            Display::fmt(param, f)?;
            if param_i + 1 != param_count {
                Display::fmt(&", ", f)?;
            }
        }
        write!(f, ")")?;

        let returns = &self.returns;
        Display::fmt(&"->", f)?;
        Display::fmt(returns, f)?;

        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////

impl From<TLField> for MRNameType {
    fn from(field: TLField) -> Self {
        let name = field.name().to_string();
        let type_ = if field.is_function() {
            field.function_range().index(0).to_string()
        } else {
            field.layout().full_type().to_string()
        };

        Self { name, type_ }
    }
}

impl Display for MRNameType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.type_)
    }
}

///////////////////////////////////////////////////////////////////////////////

impl From<ModReflMode> for MRModReflMode {
    fn from(this: ModReflMode) -> Self {
        match this {
            ModReflMode::Module { .. } => MRModReflMode::Module,
            ModReflMode::Opaque { .. } => MRModReflMode::Opaque,
            ModReflMode::DelegateDeref { .. } => MRModReflMode::DelegateDeref,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

impl From<FieldAccessor> for MRFieldAccessor {
    fn from(this: FieldAccessor) -> MRFieldAccessor {
        match this {
            FieldAccessor::Direct => MRFieldAccessor::Direct,
            FieldAccessor::Method => MRFieldAccessor::Method { name: None },
            FieldAccessor::MethodNamed { name } => MRFieldAccessor::Method {
                name: Some(name.to_string()),
            },
            FieldAccessor::MethodOption => MRFieldAccessor::MethodOption,
            FieldAccessor::Opaque => MRFieldAccessor::Opaque,
        }
    }
}
