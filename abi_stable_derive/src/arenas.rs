#![allow(clippy::mut_from_ref)]

use std::fmt;

use typed_arena::Arena;

macro_rules! declare_arenas {
    (
        $( $field_name:ident : $arena_type:ty , )*
    ) => {
        pub(crate) struct Arenas {
            $(pub(crate) $field_name : Arena<$arena_type>, )*
        }

        impl Default for Arenas{
            fn default()->Self{
                Arenas{
                    $( $field_name:Arena::new(), )*
                }
            }
        }

        impl fmt::Debug for Arenas{
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
                fmt::Debug::fmt("Arenas{..}",f)
            }
        }

        pub trait AllocMethods<T>{
            fn alloc(&self, value: T) -> &T {
                self.alloc_mut(value)
            }

            fn alloc_mut(&self, value: T) -> &mut T ;

            fn alloc_extend<I>(&self, iterable: I) -> &[T]
            where
                I: IntoIterator<Item = T>
            {
                self.alloc_extend_mut(iterable)
            }

            fn alloc_extend_mut<I>(&self, iterable: I) -> &mut [T]
            where
                I: IntoIterator<Item = T>;
        }


        $(
            impl AllocMethods<$arena_type> for Arenas{
                fn alloc_mut(&self, value: $arena_type) -> &mut $arena_type {
                    self.$field_name.alloc(value)
                }

                fn alloc_extend_mut<I>(&self, iterable: I) -> &mut [$arena_type]
                where
                    I: IntoIterator<Item = $arena_type>
                {
                    self.$field_name.alloc_extend(iterable)
                }
            }

        )*

    }
}

declare_arenas! {
    vec_meta: Vec<syn::Attribute>,
    vec_expr: Vec<syn::Expr>,
    ident: syn::Ident,
    ident_vec: Vec<syn::Ident>,
    trait_bound: syn::TraitBound,
    lifetimes:syn::Lifetime,
    fields_named: syn::FieldsNamed,
    types: syn::Type,
    // metalists: syn::MetaList,
    lifetime_defs: syn::LifetimeDef,
    tokenstream: proc_macro2::TokenStream,
    expr: syn::Expr,
    strings: String,
    paths: syn::Path,
}
