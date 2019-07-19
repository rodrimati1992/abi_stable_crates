use proc_macro2::TokenStream as TokenStream2;

use quote::{quote, ToTokens};

use syn::token::Comma;

use crate::{
    AllocMethods,
    gen_params_in::{InWhat},
    sabi_trait::{
        WhichSelf,
        WithAssocTys,
        TokenizerParams,
    },
    to_token_fn::ToTokenFnMut,
};

/// Generates the code that delegates the implementation of the supertraits 
/// to the wrapped DynTrait or RObject.
pub(super) fn delegated_impls<'a>(
    TokenizerParams{
        arenas,ctokens,config,totrait_def,trait_to,trait_backend,trait_interface,
        ..
    }:TokenizerParams<'a>,
    mod_:&mut TokenStream2,
){
    let where_preds=&totrait_def.where_preds;

    let impls=totrait_def.trait_flags;

    let erased_ptr_bounds=totrait_def.erased_ptr_preds();

    let gen_params_deser_header=
        totrait_def.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_lt_de_erasedptr,
        );

    let gen_params_header=
        totrait_def.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_lt_erasedptr,
        );
    let gen_params_use_to=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_lt_erasedptr,
        );

    let gen_params_use_to_static=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_staticlt_erasedptr,
        );

    let trait_interface_header=totrait_def.generics_tokenizer(
        InWhat::ImplHeader,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_lt,
    );

    let trait_interface_use=totrait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_empty,
    );


    if impls.debug {
        quote!(
            impl<#gen_params_header> std::fmt::Debug
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__DerefTrait<Target=()>,
                #(#where_preds,)*
            {
                #[inline]
                fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
                    std::fmt::Debug::fmt(&self.obj,f)
                }
            }
        ).to_tokens(mod_);
    }
    if impls.display {
        quote!(
            impl<#gen_params_header> std::fmt::Display
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__DerefTrait<Target=()>,
                #(#where_preds,)*
            {
                #[inline]
                fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
                    std::fmt::Display::fmt(&self.obj,f)
                }
            }
        ).to_tokens(mod_);
    }
    if impls.error {
        quote!(
            impl<#gen_params_header> std::error::Error
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__DerefTrait<Target=()>,
                #(#where_preds,)*
            {}
        ).to_tokens(mod_);
    }
    if impls.clone {
        quote!(
            impl<#gen_params_header> std::clone::Clone
            for #trait_to<#gen_params_use_to>
            where
                #trait_backend<#gen_params_use_to>:Clone,
                _ErasedPtr:__DerefTrait<Target=()>,
                #(#where_preds,)*
            {
                #[inline]
                fn clone(&self)->Self{
                    Self{
                        obj:std::clone::Clone::clone(&self.obj),
                        _marker:__sabi_re::UnsafeIgnoredType::DEFAULT,
                    }
                }
            }
        ).to_tokens(mod_);
    }
    if impls.hash {
        quote!(
            impl<#gen_params_header> std::hash::Hash
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__DerefTrait<Target=()>,
                #(#where_preds,)*
            {
                #[inline]
                fn hash<H>(&self, state: &mut H)
                where
                    H: std::hash::Hasher
                {
                    std::hash::Hash::hash(&self.obj,state)
                }
            }
        ).to_tokens(mod_);
    }
    if impls.send {
        quote!(
            unsafe impl<#gen_params_header> std::marker::Send
            for #trait_to<#gen_params_use_to>
            where
                #(#where_preds,)*
            {}
        ).to_tokens(mod_);
    }
    if impls.sync {
        quote!(
            unsafe impl<#gen_params_header> std::marker::Sync
            for #trait_to<#gen_params_use_to>
            where
                #(#where_preds,)*
            {}
        ).to_tokens(mod_);
    }
    if impls.fmt_write {
        quote!(
            impl<#gen_params_header> std::fmt::Write
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__DerefMutTrait<Target=()>,
                #(#where_preds,)*
            {
                #[inline]
                fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error>{
                    std::fmt::Write::write_str(&mut self.obj,s)
                }
            }
        ).to_tokens(mod_);
    }
    if impls.io_write {
        quote!(
            impl<#gen_params_header> std::io::Write
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__DerefMutTrait<Target=()>,
                #(#where_preds,)*
            {
                fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>{
                    std::io::Write::write(&mut self.obj,buf)
                }
                fn flush(&mut self) -> std::io::Result<()>{
                    std::io::Write::flush(&mut self.obj)
                }
                fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
                    std::io::Write::write_all(&mut self.obj,buf)
                }
            }
        ).to_tokens(mod_);
    }
    if impls.io_read {
        quote!(
            impl<#gen_params_header> std::io::Read
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__DerefMutTrait<Target=()>,
                #(#where_preds,)*
            {
                fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>{
                    std::io::Read::read(&mut self.obj,buf)
                }

                fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
                    std::io::Read::read_exact(&mut self.obj,buf)
                }
            }
        ).to_tokens(mod_);
    }
    if impls.io_buf_read {
        quote!(
            impl<#gen_params_header> std::io::BufRead
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__DerefMutTrait<Target=()>,
                #(#where_preds,)*
            {
                fn fill_buf(&mut self) -> std::io::Result<&[u8]>{
                    std::io::BufRead::fill_buf(&mut self.obj)
                }

                fn consume(&mut self, ammount:usize ){
                    std::io::BufRead::consume(&mut self.obj,ammount)
                }
            }
        ).to_tokens(mod_);
    }
    if impls.io_seek {
        quote!(
            impl<#gen_params_header> std::io::Seek
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__DerefMutTrait<Target=()>,
                #(#where_preds,)*
            {
                fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64>{
                    std::io::Seek::seek(&mut self.obj,pos)
                }
            }
        ).to_tokens(mod_);
    }

    let gen_params_header_and2=
        totrait_def.generics_tokenizer(
            InWhat::ImplHeader,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_erasedptr_and2,
        );
    let gen_params_use_2=
        totrait_def.generics_tokenizer(
            InWhat::ItemUse,
            WithAssocTys::Yes(WhichSelf::NoSelf),
            &ctokens.ts_staticlt_erasedptr2,
        );

    if impls.eq{
        quote!(
            impl<#gen_params_header> std::cmp::Eq
            for #trait_to<#gen_params_use_to_static>
            where
                _ErasedPtr:__DerefTrait<Target=()>,
                #(#where_preds,)*
            {}
        ).to_tokens(mod_);
    }
    if impls.partial_eq{
        quote!(
            impl<#gen_params_header_and2> std::cmp::PartialEq<#trait_to<#gen_params_use_2>>
            for #trait_to<#gen_params_use_to_static>
            where
                _ErasedPtr:__DerefTrait<Target=()>,
                _ErasedPtr2:__DerefTrait<Target=()>,
                #(#where_preds,)*
            {
                fn eq(&self,other:&#trait_to<#gen_params_use_2>)->bool{
                    std::cmp::PartialEq::eq(
                        &self.obj,
                        &other.obj
                    )
                }
            }
        ).to_tokens(mod_);

    }
    if impls.ord{
        quote!(
            impl<#gen_params_header> std::cmp::Ord
            for #trait_to<#gen_params_use_to_static>
            where
                _ErasedPtr:__DerefTrait<Target=()>,
                #(#where_preds,)*
            {
                fn cmp(&self,other:&Self)->std::cmp::Ordering{
                    std::cmp::Ord::cmp(
                        &self.obj,
                        &other.obj
                    )
                }
            }
        ).to_tokens(mod_);
    }
    if impls.partial_ord{
        quote!(
            impl<#gen_params_header_and2> std::cmp::PartialOrd<#trait_to<#gen_params_use_2>>
            for #trait_to<#gen_params_use_to_static>
            where
                _ErasedPtr:__DerefTrait<Target=()>,
                _ErasedPtr2:__DerefTrait<Target=()>,
                #(#where_preds,)*
            {
                fn partial_cmp(
                    &self,
                    other:&#trait_to<#gen_params_use_2>
                )->Option<std::cmp::Ordering> {
                    std::cmp::PartialOrd::partial_cmp(
                        &self.obj,
                        &other.obj
                    )
                }
            }
        ).to_tokens(mod_);
    }

    // if let Some(deserialize_bound)=&totrait_def.deserialize_bound {
    //     let deserialize_path=&deserialize_bound.bound.path;

    //     let lifetimes=deserialize_bound.bound.lifetimes.as_ref()
    //         .map(|x|ToTokenFnMut::new(move|ts|{
    //             for lt in &x.lifetimes {
    //                 lt.to_tokens(ts);
    //                 Comma::default().to_tokens(ts);
    //             }
    //         }));

    //     let suffix=&ctokens.ts_lt_erasedptr;

    //     let header_generics=arenas.alloc(quote!( #lifetimes #suffix ));

    //     let gen_params_header=
    //         totrait_def.generics_tokenizer(
    //             InWhat::ImplHeader,
    //             WithAssocTys::Yes(WhichSelf::NoSelf),
    //             header_generics,
    //         );

    //     let lifetime_param=deserialize_bound.lifetime;

    //     quote!(
    //         impl<#gen_params_header> #deserialize_path for #trait_to<#gen_params_use_to>
    //         where
    //             #trait_backend<#gen_params_use_to>: #deserialize_path,
    //             _ErasedPtr:__DerefTrait<Target=()>,
    //             #(#where_preds,)*
    //         {
    //             fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    //             where
    //                 D: ::serde::Deserializer<#lifetime_param>,
    //             {
    //                 #trait_backend::<#gen_params_use_to>::deserialize(deserializer)
    //                     .map(Self::from_sabi)
    //             }
    //         }
    //     ).to_tokens(mod_);
    // }
    
    // if impls.serialize{
    //     quote!(
    //         impl<#gen_params_header> ::serde::Serialize for #trait_to<#gen_params_use_to>
    //         where
    //             #trait_backend<#gen_params_use_to>: ::serde::Serialize,
    //             _ErasedPtr:__DerefTrait<Target=()>,
    //             #(#where_preds,)*
    //         {
    //             fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    //             where
    //                 S: ::serde::Serializer,
    //             {
    //                 self.obj.serialize(serializer)
    //             }
    //         }
    //     ).to_tokens(mod_);
    // }

    if let Some(iter_item)=&totrait_def.iterator_item {
        let ty_params=totrait_def.generics.type_params().map(|x|&x.ident);
        let assoc_tys=totrait_def.assoc_tys.keys();
        quote!(
            impl<#trait_interface_header>
                abi_stable::erased_types::IteratorItem<'lt>
            for #trait_interface<#trait_interface_use>
            where
                #iter_item:'lt,
            {
                type Item=#iter_item;
            }
        ).to_tokens(mod_);
    }
    if impls.iterator {
        quote!(
            impl<#gen_params_header> std::iter::Iterator for #trait_to<#gen_params_use_to>
            where
                #trait_backend<#gen_params_use_to>:std::iter::Iterator,
            {
                type Item=<#trait_backend<#gen_params_use_to>as std::iter::Iterator>::Item;

                fn next(&mut self)->Option<Self::Item>{
                    self.obj.next()
                }

                fn nth(&mut self,nth:usize)->Option<Self::Item>{
                    self.obj.nth(nth)
                }

                fn size_hint(&self)->(usize,Option<usize>){
                    self.obj.size_hint()
                }

                fn count(mut self)->usize{
                    self.obj.count()
                }

                fn last(mut self)->Option<Self::Item>{
                    self.obj.last()
                }
            }
        ).to_tokens(mod_);
    }
    if impls.double_ended_iterator {
        quote!(
            impl<#gen_params_header> std::iter::DoubleEndedIterator 
            for #trait_to<#gen_params_use_to>
            where
                #trait_backend<#gen_params_use_to>:std::iter::DoubleEndedIterator,
            {
                fn next_back(&mut self)->Option<Self::Item>{
                    self.obj.next_back()
                }
            }
        ).to_tokens(mod_);
    }


}
