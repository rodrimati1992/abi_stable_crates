/*!
The implementation of the `#[export_root_module]` attribute.
*/

use super::*;

use syn::Ident;

use proc_macro2::Span;

use abi_stable_shared::mangled_root_module_loader_name;




#[doc(hidden)]
pub fn mangle_library_getter_attr(_attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    parse_or_compile_err( item, mangle_library_getter_inner ).into()
}

#[cfg(test)]
fn mangle_library_getter_str(item: &str)-> Result<TokenStream2,syn::Error> {
    syn::parse_str(item)
        .and_then(mangle_library_getter_inner)
}


fn mangle_library_getter_inner(mut input:ItemFn)->Result<TokenStream2,syn::Error>{
    let vis=&input.vis;

    let unsafe_no_layout_constant_path=
        syn::parse_str::<syn::Path>("unsafe_no_layout_constant").expect("BUG");

    let mut found_unsafe_no_layout_constant=false;
    input.attrs.retain(|attr|{
        let is_it=attr.path==unsafe_no_layout_constant_path;
        found_unsafe_no_layout_constant=found_unsafe_no_layout_constant||is_it;
        !is_it
    });
    let assoc_constant=Ident::new(
        if found_unsafe_no_layout_constant { "CONSTANTS_NO_ABI_INFO" }else{ "CONSTANTS" },
        Span::call_site(),
    );

    let ret_ty=match &input.sig.output {
        syn::ReturnType::Default=>
            return_spanned_err!(
                input.sig.ident,
                "The return type can't be `()`"
            ),
        syn::ReturnType::Type(_,ty)=>
            ty,
    };
    
    let original_fn_ident=&input.sig.ident;

    let export_name=Ident::new(
        &mangled_root_module_loader_name(),
        Span::call_site(),
    );

    Ok(quote!(
        #input

        #[no_mangle]
        #vis static #export_name:abi_stable::library::LibHeader={
            use abi_stable::{
                library::{LibHeader as __LibHeader},
                StableAbi,
            };

            pub extern "C" fn _sabi_erased_module(
            )->&'static abi_stable::marker_type::ErasedObject {
                ::abi_stable::extern_fn_panic_handling!(
                    let ret:#ret_ty=#original_fn_ident();
                    let _=abi_stable::library::RootModule::load_module_with(||{
                        Ok::<_,()>(ret)
                    });
                    unsafe{
                        abi_stable::utils::transmute_reference(ret)
                    }
                )
            }

            type __ReturnTy=#ret_ty;
            type __ModuleTy=<__ReturnTy as std::ops::Deref>::Target;
            
            unsafe{
                __LibHeader::from_constructor::<__ModuleTy>(
                    abi_stable::sabi_types::Constructor(_sabi_erased_module),
                    <__ModuleTy as abi_stable::library::RootModule>::#assoc_constant,
                )
            }
        };
    ))
}



#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_output(){
        let list=vec![
            (
                r##"
                    pub fn hello()->RString{}
                "##,
                "RootModule>::CONSTANTS"
            ),
            (
                r##"
                    #[unsafe_no_layout_constant]
                    pub fn hello()->RString{}
                "##,
                "RootModule>::CONSTANTS_NO_ABI_INFO"
            ),
            (
                r##"
                    #[hello]
                    #[unsafe_no_layout_constant]
                    pub fn hello()->RString{}
                "##,
                "RootModule>::CONSTANTS_NO_ABI_INFO"
            ),
            (
                r##"
                    #[hello]
                    #[unsafe_no_layout_constant]
                    #[hello]
                    pub fn hello()->RString{}
                "##,
                "RootModule>::CONSTANTS_NO_ABI_INFO"
            ),
            (
                r##"
                    #[unsafe_no_layout_constant]
                    #[hello]
                    pub fn hello()->RString{}
                "##,
                "RootModule>::CONSTANTS_NO_ABI_INFO"
            ),
        ];

        for (item,expected_const) in list {
            let str_out=mangle_library_getter_str(item).unwrap().to_string()
                .chars()
                .filter(|c|!c.is_whitespace())
                .collect::<String>();
            assert!(str_out.contains(expected_const));
        }
    }
}