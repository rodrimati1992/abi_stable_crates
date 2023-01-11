// Only used by non-unit zero-sized types
//
macro_rules! monomorphic_marker_type {
    ($name:ident, $field:ty) => {
        #[allow(non_upper_case_globals)]
        const _: () = {
            monomorphic_marker_type! {@inner $name, $field}
        };
    };
    (@inner $name:ident, $field:ty) => {
        const _item_info_const_: abi_stable::type_layout::ItemInfo = abi_stable::make_item_info!();
        const _SHARED_VARS_STRINGS_: ::abi_stable::std_types::RStr<'static> =
            abi_stable::std_types::RStr::from_str("_marker;");

        use ::abi_stable::derive_macro_reexports::{self as __sabi_re, renamed::*};
        pub struct _static_(extern "C" fn());
        unsafe impl __GetStaticEquivalent_ for $name {
            type StaticEquivalent = _static_;
        }
        #[doc(hidden)]
        pub(super) const _MONO_LAYOUT_: &'static __sabi_re::MonoTypeLayout =
            &__sabi_re::MonoTypeLayout::from_derive(__sabi_re::_private_MonoTypeLayoutDerive {
                name: abi_stable::std_types::RStr::from_str(stringify!($name)),
                item_info: _item_info_const_,
                data: __sabi_re::MonoTLData::derive_struct(__CompTLFields::new(
                    abi_stable::std_types::RSlice::from_slice(&[562949953880064u64]),
                    None,
                )),
                generics: abi_stable ::
                                           tl_genparams !
                                                   (; __StartLen :: new(0u16, 0u16) ; __StartLen ::
                                            new(0u16, 0u16)),
        mod_refl_mode: __ModReflMode::Opaque,
        repr_attr: __ReprAttr::C,
        phantom_fields: abi_stable::std_types::RSlice::from_slice(&[]),
                shared_vars: abi_stable::type_layout::MonoSharedVars::new(
                    _SHARED_VARS_STRINGS_,
                    abi_stable::std_types::RSlice::from_slice(&[]),
                ),
            });
        impl $name {
            const __SABI_SHARED_VARS: &'static __sabi_re::SharedVars =
                &abi_stable::type_layout::SharedVars::new(
                    _MONO_LAYOUT_.shared_vars_static(),
                    abi_stable::_sabi_type_layouts!($field,),
                    __sabi_re::RSlice::from_slice(&[]),
                );
        }
        unsafe impl __sabi_re::StableAbi for $name {
            type IsNonZeroType = __sabi_re::False;
            const LAYOUT: &'static __sabi_re::TypeLayout = {
                zst_assert! {Self}

                &__sabi_re::TypeLayout::from_derive::<Self>(__sabi_re::_private_TypeLayoutDerive {
                    shared_vars: Self::__SABI_SHARED_VARS,
                    mono: _MONO_LAYOUT_,
                    abi_consts: Self::ABI_CONSTS,
                    data: __sabi_re::GenericTLData::Struct,
                    tag: None,
                    extra_checks: None,
                })
            };
        }
    };
}
