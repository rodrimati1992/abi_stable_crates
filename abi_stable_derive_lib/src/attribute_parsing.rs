
use syn::{
    Meta, NestedMeta,
};


/// Iterates over an iterator of syn::NestedMeta,
/// unwrapping it into a syn::Meta and passing it into the `f` closure.
pub(crate) fn with_nested_meta<I, F>(attr_name: &str, iter: I, mut f: F)->Result<(),syn::Error>
where
    F: FnMut(Meta)->Result<(),syn::Error>,
    I: IntoIterator<Item = NestedMeta>,
{
    for repr in iter {
        match repr {
            NestedMeta::Meta(attr) => {
                f(attr)?;
            }
            NestedMeta::Lit(lit) => {
                return_spanned_err!(
                    lit,
                    "the #[{}(...)] attribute does not allow literals in the attribute list",
                    attr_name,
                );
            }
        }
    }
    Ok(())
}



///////////////////////////////////////////////////////////////////////////////






///////////////////////////////////////////////////////////////////////////////





