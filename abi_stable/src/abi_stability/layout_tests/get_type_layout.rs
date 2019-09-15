
use std::marker::PhantomData;

#[allow(unused_imports)]
use core_extensions::SelfOps;

#[allow(unused_imports)]
use crate::std_types::{Tuple1,Tuple2,Tuple3,Tuple4};

use crate::{
    abi_stability::{
        stable_abi_trait::{
            TypeLayoutCtor,
            GetTypeLayoutCtor,
            get_type_layout,
        },
        check_layout_compatibility,
    },
    std_types::*,
    type_layout::TypeLayout,
    StableAbi,
};

use super::shared_types::{
    basic_enum,
    gen_basic,
    gen_more_lts,
    enum_extra_fields_b,
    extra_variant,
    swapped_fields_first,
    gen_more_lts_b,
    mod_5,
    mod_7,
};


struct FnTest{
    params:Vec<usize>,
    ret:usize,
}

struct TypeTest{
    layout:&'static TypeLayout,
    vars_types:Vec<TypeLayoutCtor>,
    field_types:Vec<usize>,
    functions:Vec<FnTest>,
}

fn get_tlc<T>()->TypeLayoutCtor
where
    T:StableAbi
{
    GetTypeLayoutCtor::<T>::STABLE_ABI
}

#[test]
fn assert_types(){
    let list=vec![
        TypeTest{
            layout:get_type_layout::<Tuple1<i32>>(),
            vars_types:vec![get_tlc::<i32>()],
            field_types:vec![0],
            functions:vec![],
        },

        TypeTest{
            layout:get_type_layout::<Tuple2<i32,i32>>(),
            vars_types:vec![get_tlc::<i32>(), get_tlc::<i32>()],
            field_types:vec![0,1],
            functions:vec![],
        },
        TypeTest{
            layout:get_type_layout::<Tuple2<i32,RVec<()>>>(),
            vars_types:vec![get_tlc::<i32>(), get_tlc::<RVec<()>>()],
            field_types:vec![0,1],
            functions:vec![],
        },

        TypeTest{
            layout:get_type_layout::<Tuple3<i32,RString,i32>>(),
            vars_types:vec![ get_tlc::<i32>(), get_tlc::<RString>(), get_tlc::<i32>() ],
            field_types:vec![0,1,2],
            functions:vec![],
        },
        TypeTest{
            layout:get_type_layout::<Tuple3<i32,i32,i32>>(),
            vars_types:vec![ get_tlc::<i32>(), get_tlc::<i32>(), get_tlc::<i32>() ],
            field_types:vec![0,1,2],
            functions:vec![],
        },

        TypeTest{
            layout:get_type_layout::<gen_basic::Generics<i32>>(),
            vars_types:vec![ get_tlc::<&i32>(), get_tlc::<PhantomData<i32>>() ],
            field_types:vec![0,0,1],
            functions:vec![],
        },
        TypeTest{
            layout:get_type_layout::<gen_basic::Generics<RString>>(),
            vars_types:vec![ get_tlc::<&RString>(), get_tlc::<PhantomData<RString>>() ],
            field_types:vec![0,0,1],
            functions:vec![],
        },

        TypeTest{
            layout:get_type_layout::<basic_enum::Enum>(),
            vars_types:vec![ get_tlc::<u32>() ],
            field_types:vec![0],
            functions:vec![],
        },

        TypeTest{
            layout:get_type_layout::<enum_extra_fields_b::Enum>(),
            vars_types:vec![ get_tlc::<u32>() ],
            field_types:vec![0,0,0],
            functions:vec![],
        },

        TypeTest{
            layout:get_type_layout::<extra_variant::Enum>(),
            vars_types:vec![ get_tlc::<u32>(), get_tlc::<RString>() ],
            field_types:vec![0,1],
            functions:vec![],
        },

        TypeTest{
            layout:get_type_layout::<swapped_fields_first::Rectangle>(),
            vars_types:vec![ get_tlc::<u32>(), get_tlc::<u16>() ],
            field_types:vec![0,0,1,0],
            functions:vec![],
        },

        TypeTest{
            layout:get_type_layout::<gen_more_lts_b::Generics<'_>>(),
            vars_types:vec![ get_tlc::<&()>() ],
            field_types:vec![0,0],
            functions:vec![],
        },

        TypeTest{
            layout:get_type_layout::<mod_5::Mod>(),
            vars_types:vec![
                get_tlc::<extern "C" fn()>(),
                get_tlc::<RString>(),
                get_tlc::<&mut u32>(),
                get_tlc::<u64>(),
            ],
            field_types:vec![0,0,0],
            functions:vec![
                FnTest{ params:vec![] ,ret:1 },
                FnTest{ params:vec![2,3,1] ,ret:!0 },
                FnTest{ params:vec![2,3,1] ,ret:!0 },
            ],
        },

        TypeTest{
            layout:get_type_layout::<mod_7::Mod>(),
            vars_types:vec![
                get_tlc::<extern "C" fn()>(),
                get_tlc::<RString>(),
                get_tlc::<&mut u32>(),
                get_tlc::<u64>(),
                get_tlc::<()>(),
            ],
            field_types:vec![0,0,0],
            functions:vec![
                FnTest{ params:vec![] ,ret:1 },
                FnTest{ params:vec![2,3,1] ,ret:!0},
                FnTest{ params:vec![4,4,4] ,ret:!0 },
            ],
        },
    ];

    let test_layout=|field_layout,expected_layout|{
        let res=check_layout_compatibility(field_layout,expected_layout);

        assert!(
            res.is_ok(),
            "field:{} expected:{}",
            field_layout.full_type(),
            expected_layout.full_type(),
        );
    };

    for ty_test in list {
        let shared_vars=ty_test.layout.shared_vars();

        let fields=ty_test.layout.get_fields().unwrap();
        for (field_i,field) in fields.iter().enumerate() {
            let field_layout=field.layout();
            let expected_layout={
                let x=ty_test.field_types[field_i];
                ty_test.vars_types[x].get()
            };

            test_layout(field_layout,expected_layout);
        }
        
        let mut expected_fns=ty_test.functions.iter();
        for func in fields.iter().flat_map(|f| f.function_range() ){
            let expected_fn=expected_fns.next().unwrap();
            let mut expected_params=expected_fn.params.iter();

            for param in func.param_type_layouts.iter() {
                let expected_param=expected_params.next()
                    .and_then(|x| ty_test.vars_types.get(*x) )
                    .unwrap_or_else(|| panic!("mismatched parameter type: {}",func) );
                test_layout( param.get(), expected_param.get() );
            }

            match ( func.return_type_layout, ty_test.vars_types.get(expected_fn.ret) ) {
                (Some(found_ret),Some(expected_ret))=>{
                    test_layout( found_ret.get(), expected_ret.get() );
                }
                (None,None)=>{}
                _=>panic!(
                    "mismatched return type: {}\n\
                     shared_vars.type_layouts:{:#?}\n\
                     found function parameter/ret:{:#?} {:?}
                     expected function parameter/ret:{:#?} {:?}
                     ",
                    func,
                    type_layouts_fmt(shared_vars.type_layouts().iter().cloned()),
                    type_layouts_fmt(func.param_type_layouts.iter()),
                    type_layouts_fmt(func.return_type_layout),
                    type_layouts_fmt(expected_fn.params.iter().map(|x| ty_test.vars_types[*x] )),
                    type_layouts_fmt(ty_test.vars_types.get(expected_fn.ret).cloned()),
                )
            }
        }
    }
}


fn type_layouts_fmt(iter:impl IntoIterator<Item=TypeLayoutCtor>)->Vec<String>{
    iter.into_iter()
        .map(|x|x.get().full_type().to_string())
        .collect::<Vec<String>>()
}