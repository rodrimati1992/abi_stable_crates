use super::SharedVars;

use syn::Ident;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FieldAccessor<'a> {
    /// Accessible with `self.field_name`
    Direct,
    /// Accessible with `fn field_name(&self)->FieldType`
    Method { name: Option<&'a Ident> },
    /// Accessible with `fn field_name(&self)->Option<FieldType>`
    MethodOption,
    /// This field is completely inaccessible.
    Opaque,
}

impl<'a> FieldAccessor<'a> {
    pub(crate) fn compress(self, shared_vars: &mut SharedVars<'a>) -> CompFieldAccessor {
        match self {
            FieldAccessor::Direct => CompFieldAccessor::DIRECT,
            FieldAccessor::Method { name: None } => CompFieldAccessor::METHOD,
            FieldAccessor::Method { name: Some(name) } => {
                let _ = shared_vars.push_ident(name);
                CompFieldAccessor::METHOD_NAMED
            }
            FieldAccessor::MethodOption => CompFieldAccessor::METHOD_OPTION,
            FieldAccessor::Opaque => CompFieldAccessor::OPAQUE,
        }
    }
}

abi_stable_shared::declare_comp_field_accessor! {
    attrs=[]
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ModReflMode<T> {
    Module,
    Opaque,
    DelegateDeref(T),
}
