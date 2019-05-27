use syn::{Meta, NestedMeta};




/// Iterates over an iterator of syn::NestedMeta,
/// unwrapping it into a syn::Meta and passing it into the `f` closure.
pub(crate) fn with_nested_meta<I, F>(attr_name: &str, iter: I, mut f: F)
where
    F: FnMut(Meta),
    I: IntoIterator<Item = NestedMeta>,
{
    for repr in iter {
        match repr {
            NestedMeta::Meta(attr) => {
                f(attr);
            }
            NestedMeta::Literal(lit) => {
                panic!(
                    "\
                     the #[{}(...)] attribute does not allow \
                     literals in the attribute list:\n{:?}\
                     ",
                    attr_name, lit
                );
            }
        }
    }
}


