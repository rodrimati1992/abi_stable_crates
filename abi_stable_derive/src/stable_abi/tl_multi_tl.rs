use super::*;

use crate::fn_pointer_extractor::FnParamRet;

abi_stable_shared::declare_type_layout_index! {
    attrs=[]
}

impl TypeLayoutIndex {
    /// Used to recover from syn errors,
    /// this value shouldn't be used in the layout constant since it's reserved
    /// for errors.
    pub const DUMMY: Self = Self::from_u10(!0);
}

abi_stable_shared::declare_multi_tl_types! {
    attrs=[]
}

impl TypeLayoutRange {
    pub(crate) fn compress_params<'a>(
        params: &[FnParamRet<'a>],
        shared_vars: &mut SharedVars<'a>,
    ) -> Self {
        let param_len = params.len();
        if param_len <= TypeLayoutRange::STORED_INLINE {
            let mut arr = [0u16; TypeLayoutRange::STORED_INLINE];

            for (p_i, param) in params.iter().enumerate() {
                let ty: &'a syn::Type = param.ty;
                arr[p_i] = shared_vars
                    .push_type(LayoutConstructor::Regular, ty)
                    .to_u10();
            }

            Self::with_up_to_5(param_len, arr)
        } else {
            const I_BEFORE: usize = TypeLayoutRange::STORED_INLINE - 1;
            let mut arr = [0u16; TypeLayoutRange::STORED_INLINE];

            let mut iter = params.iter().map(|p| -> &'a syn::Type { p.ty });
            for (p_i, ty) in iter.by_ref().take(I_BEFORE).enumerate() {
                arr[p_i] = shared_vars
                    .push_type(LayoutConstructor::Regular, ty)
                    .to_u10();
            }

            arr[I_BEFORE] = shared_vars
                .extend_type(LayoutConstructor::Regular, iter)
                .start;

            Self::with_more_than_5(param_len, arr)
        }
    }
}
