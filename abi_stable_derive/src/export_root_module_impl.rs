//! The implementation of the `#[export_root_module]` attribute.

use super::*;

use as_derive_utils::return_spanned_err;

use syn::Ident;

use proc_macro2::Span;

use abi_stable_shared::mangled_root_module_loader_name;

#[doc(hidden)]
pub fn export_root_module_attr(_attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    parse_or_compile_err(item, export_root_module_inner).into()
}

#[cfg(test)]
fn export_root_module_str(item: &str) -> Result<TokenStream2, syn::Error> {
    syn::parse_str(item).and_then(export_root_module_inner)
}

fn export_root_module_inner(mut input: ItemFn) -> Result<TokenStream2, syn::Error> {
    let vis = &input.vis;

    let unsafe_no_layout_constant_path =
        syn::parse_str::<syn::Path>("unsafe_no_layout_constant").expect("BUG");

    let mut found_unsafe_no_layout_constant = false;
    input.attrs.retain(|attr| {
        let is_it = attr.path == unsafe_no_layout_constant_path;
        found_unsafe_no_layout_constant = found_unsafe_no_layout_constant || is_it;
        !is_it
    });
    let check_ty_layout_variant = Ident::new(
        if found_unsafe_no_layout_constant {
            "No"
        } else {
            "Yes"
        },
        Span::call_site(),
    );

    let ret_ty = match &input.sig.output {
        syn::ReturnType::Default => {
            return_spanned_err!(input.sig.ident, "The return type can't be `()`")
        }
        syn::ReturnType::Type(_, ty) => ty,
    };

    let original_fn_ident = &input.sig.ident;

    let export_name = Ident::new(&mangled_root_module_loader_name(), Span::call_site());

    Ok(quote!(
        #input

        #[no_mangle]
        #vis static #export_name: ::abi_stable::library::LibHeader = {

            pub extern "C" fn _sabi_erased_module()-> ::abi_stable::library::RootModuleResult {
                ::abi_stable::library::__call_root_module_loader(#original_fn_ident)
            }

            type __SABI_Module = <#ret_ty as ::abi_stable::library::IntoRootModuleResult>::Module;
            unsafe{
                ::abi_stable::library::LibHeader::from_constructor::<__SABI_Module>(
                    _sabi_erased_module,
                    ::abi_stable::library::CheckTypeLayout::#check_ty_layout_variant,
                )
            }
        };
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output() {
        let list = vec![
            (
                r##"
                    pub fn hello()->RString{}
                "##,
                "CheckTypeLayout::Yes",
            ),
            (
                r##"
                    #[unsafe_no_layout_constant]
                    pub fn hello()->RString{}
                "##,
                "CheckTypeLayout::No",
            ),
            (
                r##"
                    #[hello]
                    #[unsafe_no_layout_constant]
                    pub fn hello()->RString{}
                "##,
                "CheckTypeLayout::No",
            ),
            (
                r##"
                    #[hello]
                    #[unsafe_no_layout_constant]
                    #[hello]
                    pub fn hello()->RString{}
                "##,
                "CheckTypeLayout::No",
            ),
            (
                r##"
                    #[unsafe_no_layout_constant]
                    #[hello]
                    pub fn hello()->RString{}
                "##,
                "CheckTypeLayout::No",
            ),
        ];

        for (item, expected_const) in list {
            let str_out = export_root_module_str(item)
                .unwrap()
                .to_string()
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>();
            assert!(str_out.contains(expected_const));
        }
    }
}
