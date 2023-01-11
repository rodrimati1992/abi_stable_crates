use proc_macro2::TokenStream as TokenStream2;

use quote::{quote_spanned, ToTokens};

use as_derive_utils::gen_params_in::InWhat;

use crate::sabi_trait::{TokenizerParams, WhichSelf, WithAssocTys};

/// Generates the code that delegates the implementation of the traits
/// to the wrapped DynTrait or RObject.
pub(super) fn delegated_impls(
    TokenizerParams {
        arenas: _,
        ctokens,
        totrait_def,
        trait_to,
        trait_backend,
        trait_interface,
        lt_tokens,
        ..
    }: TokenizerParams<'_>,
    mod_: &mut TokenStream2,
) {
    let where_preds = &totrait_def.where_preds;

    let impls = totrait_def.trait_flags;
    let spans = &totrait_def.trait_spans;

    // let gen_params_deser_header=
    //     totrait_def.generics_tokenizer(
    //         InWhat::ImplHeader,
    //         WithAssocTys::Yes(WhichSelf::NoSelf),
    //         &lt_tokens.lt_de_erasedptr,
    //     );

    let gen_params_header = totrait_def.generics_tokenizer(
        InWhat::ImplHeader,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &lt_tokens.lt_erasedptr,
    );
    let gen_params_use_to = totrait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &lt_tokens.lt_erasedptr,
    );

    let gen_params_use_to_static = totrait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &lt_tokens.staticlt_erasedptr,
    );

    let trait_interface_header = totrait_def.generics_tokenizer(
        InWhat::ImplHeader,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &lt_tokens.lt,
    );

    let trait_interface_use = totrait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_empty,
    );

    if impls.debug {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.debug=>
            impl<#gen_params_header> std::fmt::Debug
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__sabi_re::AsPtr<PtrTarget=()>,
                #(#where_preds,)*
            {
                #[inline]
                fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
                    std::fmt::Debug::fmt(&self.obj,f)
                }
            }
        )
        .to_tokens(mod_);
    }
    if impls.display {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.display=>
            impl<#gen_params_header> std::fmt::Display
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__sabi_re::AsPtr<PtrTarget=()>,
                #(#where_preds,)*
            {
                #[inline]
                fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
                    std::fmt::Display::fmt(&self.obj,f)
                }
            }
        )
        .to_tokens(mod_);
    }
    if impls.error {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.error=>
            impl<#gen_params_header> std::error::Error
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__sabi_re::AsPtr<PtrTarget=()>,
                #(#where_preds,)*
            {}
        )
        .to_tokens(mod_);
    }
    if impls.clone {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.clone=>
            impl<#gen_params_header> std::clone::Clone
            for #trait_to<#gen_params_use_to>
            where
                #trait_backend<#gen_params_use_to>:Clone,
                _ErasedPtr:__sabi_re::AsPtr<PtrTarget=()>,
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
        )
        .to_tokens(mod_);
    }
    if impls.hash {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.hash=>
            impl<#gen_params_header> std::hash::Hash
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__sabi_re::AsPtr<PtrTarget=()>,
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
        )
        .to_tokens(mod_);
    }
    if impls.send {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.send=>
            unsafe impl<#gen_params_header> std::marker::Send
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__GetPointerKind,
                #(#where_preds,)*
            {}
        )
        .to_tokens(mod_);
    }
    if impls.sync {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.sync=>
            unsafe impl<#gen_params_header> std::marker::Sync
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__GetPointerKind,
                #(#where_preds,)*
            {}
        )
        .to_tokens(mod_);
    }
    if impls.fmt_write {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.fmt_write=>
            impl<#gen_params_header> std::fmt::Write
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__sabi_re::AsMutPtr<PtrTarget=()>,
                #(#where_preds,)*
            {
                #[inline]
                fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error>{
                    std::fmt::Write::write_str(&mut self.obj,s)
                }
            }
        )
        .to_tokens(mod_);
    }
    if impls.io_write {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.io_write=>
            impl<#gen_params_header> std::io::Write
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__sabi_re::AsMutPtr<PtrTarget=()>,
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
        )
        .to_tokens(mod_);
    }
    if impls.io_read {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.io_read=>
            impl<#gen_params_header> std::io::Read
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__sabi_re::AsMutPtr<PtrTarget=()>,
                #(#where_preds,)*
            {
                fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>{
                    std::io::Read::read(&mut self.obj,buf)
                }

                fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
                    std::io::Read::read_exact(&mut self.obj,buf)
                }
            }
        )
        .to_tokens(mod_);
    }
    if impls.io_buf_read {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.io_buf_read=>
            impl<#gen_params_header> std::io::BufRead
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__sabi_re::AsMutPtr<PtrTarget=()>,
                #(#where_preds,)*
            {
                fn fill_buf(&mut self) -> std::io::Result<&[u8]>{
                    std::io::BufRead::fill_buf(&mut self.obj)
                }

                fn consume(&mut self, amount:usize ){
                    std::io::BufRead::consume(&mut self.obj,amount)
                }
            }
        )
        .to_tokens(mod_);
    }
    if impls.io_seek {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.io_seek=>
            impl<#gen_params_header> std::io::Seek
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__sabi_re::AsMutPtr<PtrTarget=()>,
                #(#where_preds,)*
            {
                fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64>{
                    std::io::Seek::seek(&mut self.obj,pos)
                }
            }
        )
        .to_tokens(mod_);
    }

    let gen_params_header_and2 = totrait_def.generics_tokenizer(
        InWhat::ImplHeader,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &ctokens.ts_erasedptr_and2,
    );
    let gen_params_use_2 = totrait_def.generics_tokenizer(
        InWhat::ItemUse,
        WithAssocTys::Yes(WhichSelf::NoSelf),
        &lt_tokens.staticlt_erasedptr2,
    );

    if impls.eq {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.eq=>
            #[allow(clippy::extra_unused_lifetimes)]
            impl<#gen_params_header> std::cmp::Eq
            for #trait_to<#gen_params_use_to_static>
            where
                _ErasedPtr:__sabi_re::AsPtr<PtrTarget=()>,
                #(#where_preds,)*
            {}
        )
        .to_tokens(mod_);
    }
    if impls.partial_eq {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.partial_eq=>
            #[allow(clippy::extra_unused_lifetimes)]
            impl<#gen_params_header_and2> std::cmp::PartialEq<#trait_to<#gen_params_use_2>>
            for #trait_to<#gen_params_use_to_static>
            where
                _ErasedPtr:__sabi_re::AsPtr<PtrTarget=()>,
                _ErasedPtr2:__sabi_re::AsPtr<PtrTarget=()>,
                #(#where_preds,)*
            {
                fn eq(&self,other:&#trait_to<#gen_params_use_2>)->bool{
                    std::cmp::PartialEq::eq(
                        &self.obj,
                        &other.obj
                    )
                }
            }
        )
        .to_tokens(mod_);
    }
    if impls.ord {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.ord=>
            #[allow(clippy::extra_unused_lifetimes)]
            impl<#gen_params_header> std::cmp::Ord
            for #trait_to<#gen_params_use_to_static>
            where
                _ErasedPtr:__sabi_re::AsPtr<PtrTarget=()>,
                #(#where_preds,)*
            {
                fn cmp(&self,other:&Self)->std::cmp::Ordering{
                    std::cmp::Ord::cmp(
                        &self.obj,
                        &other.obj
                    )
                }
            }
        )
        .to_tokens(mod_);
    }
    if impls.partial_ord {
        let where_preds = where_preds.into_iter();

        quote_spanned!(spans.partial_ord=>
            #[allow(clippy::extra_unused_lifetimes)]
            impl<#gen_params_header_and2> std::cmp::PartialOrd<#trait_to<#gen_params_use_2>>
            for #trait_to<#gen_params_use_to_static>
            where
                _ErasedPtr:__sabi_re::AsPtr<PtrTarget=()>,
                _ErasedPtr2:__sabi_re::AsPtr<PtrTarget=()>,
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
        )
        .to_tokens(mod_);
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

    //     let suffix=&lt_tokens.lt_erasedptr;

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
    //             _ErasedPtr:__sabi_re::AsPtr<PtrTarget=()>,
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
    //             _ErasedPtr:__sabi_re::AsPtr<PtrTarget=()>,
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

    if let Some(iter_item) = &totrait_def.iterator_item {
        let one_lt = &lt_tokens.one_lt;
        quote_spanned!(spans.iterator=>
            impl<#trait_interface_header>
                abi_stable::erased_types::IteratorItem<#one_lt>
            for #trait_interface<#trait_interface_use>
            where
                #iter_item:#one_lt
            {
                type Item=#iter_item;
            }
        )
        .to_tokens(mod_);
    }
    if impls.iterator {
        quote_spanned!(spans.iterator=>
            impl<#gen_params_header> std::iter::Iterator for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__GetPointerKind,
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

                fn count(self)->usize{
                    self.obj.count()
                }

                fn last(self)->Option<Self::Item>{
                    self.obj.last()
                }
            }
        )
        .to_tokens(mod_);
    }
    if impls.double_ended_iterator {
        quote_spanned!(spans.double_ended_iterator=>
            impl<#gen_params_header> std::iter::DoubleEndedIterator
            for #trait_to<#gen_params_use_to>
            where
                _ErasedPtr:__GetPointerKind,
                #trait_backend<#gen_params_use_to>:std::iter::DoubleEndedIterator,
            {
                fn next_back(&mut self)->Option<Self::Item>{
                    self.obj.next_back()
                }
            }
        )
        .to_tokens(mod_);
    }
}
