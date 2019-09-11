use super::*;

use crate::{
    fn_pointer_extractor::FnParamRet,
};

abi_stable_shared:: declare_multi_tl_types!{
    attrs=[]
}


impl TypeLayoutRange{
    pub(crate) fn compress_params<'a>(
        params:&[FnParamRet<'a>],
        shared_vars:&mut SharedVars<'a>,
    )->Self{
        let param_len=params.len();
        if param_len <= 4 {
            let mut arr=[0u16;4];

            for (p_i,param) in params.iter().enumerate() {
                let ty:&'a syn::Type=param.ty;
                arr[p_i]=shared_vars.push_type( LayoutConstructor::Regular,ty );
            }

            Self::with_up_to_4(param_len,arr[0],arr[1],arr[2],arr[3])
        }else{
            let mut arr=[0u16;3];

            let mut iter=params.iter().map(|p|->&'a syn::Type{ p.ty });
            for (p_i,ty) in iter.by_ref().take(3).enumerate() {
                arr[p_i]=shared_vars.push_type( LayoutConstructor::Regular,ty );
            }

            let rem=shared_vars.extend_type(LayoutConstructor::Regular,iter).start;

            Self::with_more_than_4(param_len,arr[0],arr[1],arr[2],rem)
        }
    }


}