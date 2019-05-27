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
    ident: syn::Ident,
    ident_vec: Vec<syn::Ident>,
    // paths: syn::Path,
    fields_named: syn::FieldsNamed,
    types: syn::Type,
    // metalists: syn::MetaList,
    // visibilities: syn::Visibility,
    // tokenstream: TokenStream,
    meta_attr: syn::Meta,
    expr: syn::Expr,
    strings: String,
    paths: syn::Path,
}

