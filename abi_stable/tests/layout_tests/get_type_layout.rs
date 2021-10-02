use std::marker::PhantomData;

#[allow(unused_imports)]
use core_extensions::SelfOps;

#[allow(unused_imports)]
use abi_stable::std_types::{Tuple1, Tuple2, Tuple3, Tuple4};

use abi_stable::{
    abi_stability::{
        check_layout_compatibility,
        stable_abi_trait::{get_type_layout, GetTypeLayoutCtor, TypeLayoutCtor},
    },
    std_types::*,
    type_layout::TypeLayout,
    StableAbi,
};

use super::shared_types::{
    basic_enum, enum_extra_fields_b, extra_variant, gen_basic, gen_more_lts_b, mod_5, mod_7,
    swapped_fields_first,
};

////////////////////////////////////////////////////////////////////////////////

/// This is to test that function pointers with 5 or more parameters
/// store the TypeLayoutCtor for the remaining parameters after the 5th one.
pub(super) mod many_params {
    use super::RString;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Mod {
        pub function_0: extern "C" fn() -> RString,
        pub function_1: extern "C" fn(&mut u32, u64, RString, i8, u32, i64),
        pub function_2: extern "C" fn((), i64, (), (), ()),
        pub function_3: extern "C" fn(u32, u64, i8, RString) -> &'static mut u32,
        pub function_4: extern "C" fn((), (), (), (), (), ()),
    }
}

////////////////////////////////////////////////////////////////////////////////

struct FnTest {
    params: Vec<usize>,
    ret: usize,
}

struct TypeTest {
    layout: &'static TypeLayout,
    vars_types: Vec<TypeLayoutCtor>,
    field_types: Vec<usize>,
    functions: Vec<Vec<FnTest>>,
}

#[cfg(not(miri))]
fn get_tlc<T>() -> TypeLayoutCtor
where
    T: StableAbi,
{
    GetTypeLayoutCtor::<T>::STABLE_ABI
}

#[cfg(not(miri))]
#[test]
fn assert_types() {
    let list = vec![
        TypeTest {
            layout: get_type_layout::<Tuple1<i32>>(),
            vars_types: vec![get_tlc::<i32>()],
            field_types: vec![0],
            functions: vec![vec![]],
        },
        TypeTest {
            layout: get_type_layout::<Tuple2<i32, i32>>(),
            vars_types: vec![get_tlc::<i32>(), get_tlc::<i32>()],
            field_types: vec![0, 1],
            functions: vec![vec![]],
        },
        TypeTest {
            layout: get_type_layout::<Tuple2<i32, RVec<()>>>(),
            vars_types: vec![get_tlc::<i32>(), get_tlc::<RVec<()>>()],
            field_types: vec![0, 1],
            functions: vec![vec![]],
        },
        TypeTest {
            layout: get_type_layout::<Tuple3<i32, RString, i32>>(),
            vars_types: vec![get_tlc::<i32>(), get_tlc::<RString>(), get_tlc::<i32>()],
            field_types: vec![0, 1, 2],
            functions: vec![vec![]],
        },
        TypeTest {
            layout: get_type_layout::<Tuple3<i32, i32, i32>>(),
            vars_types: vec![get_tlc::<i32>(), get_tlc::<i32>(), get_tlc::<i32>()],
            field_types: vec![0, 1, 2],
            functions: vec![vec![]],
        },
        TypeTest {
            layout: get_type_layout::<gen_basic::Generics<i32>>(),
            vars_types: vec![get_tlc::<&i32>(), get_tlc::<PhantomData<i32>>()],
            field_types: vec![0, 0, 1],
            functions: vec![vec![]],
        },
        TypeTest {
            layout: get_type_layout::<gen_basic::Generics<RString>>(),
            vars_types: vec![get_tlc::<&RString>(), get_tlc::<PhantomData<RString>>()],
            field_types: vec![0, 0, 1],
            functions: vec![vec![]],
        },
        TypeTest {
            layout: get_type_layout::<basic_enum::Enum>(),
            vars_types: vec![get_tlc::<u32>()],
            field_types: vec![0],
            functions: vec![vec![]],
        },
        TypeTest {
            layout: get_type_layout::<enum_extra_fields_b::Enum>(),
            vars_types: vec![get_tlc::<u32>()],
            field_types: vec![0, 0, 0],
            functions: vec![vec![]],
        },
        TypeTest {
            layout: get_type_layout::<extra_variant::Enum>(),
            vars_types: vec![get_tlc::<u32>(), get_tlc::<RString>()],
            field_types: vec![0, 1],
            functions: vec![vec![]],
        },
        TypeTest {
            layout: get_type_layout::<swapped_fields_first::Rectangle>(),
            vars_types: vec![get_tlc::<u32>(), get_tlc::<u16>()],
            field_types: vec![0, 0, 1, 0],
            functions: vec![vec![]],
        },
        TypeTest {
            layout: get_type_layout::<gen_more_lts_b::Generics<'_>>(),
            vars_types: vec![get_tlc::<&()>()],
            field_types: vec![0, 0],
            functions: vec![vec![]],
        },
        TypeTest {
            layout: get_type_layout::<mod_5::Mod>(),
            vars_types: vec![
                get_tlc::<extern "C" fn()>(),
                get_tlc::<RString>(),
                get_tlc::<&mut u32>(),
                get_tlc::<u64>(),
            ],
            field_types: vec![0, 0, 0],
            functions: vec![
                vec![FnTest {
                    params: vec![],
                    ret: 1,
                }],
                vec![FnTest {
                    params: vec![2, 3, 1],
                    ret: !0,
                }],
                vec![FnTest {
                    params: vec![2, 3, 1],
                    ret: !0,
                }],
            ],
        },
        TypeTest {
            layout: get_type_layout::<mod_7::Mod>(),
            vars_types: vec![
                get_tlc::<extern "C" fn()>(),
                get_tlc::<RString>(),
                get_tlc::<&mut u32>(),
                get_tlc::<u64>(),
                get_tlc::<()>(),
            ],
            field_types: vec![0, 0, 0],
            functions: vec![
                vec![FnTest {
                    params: vec![],
                    ret: 1,
                }],
                vec![FnTest {
                    params: vec![2, 3, 1],
                    ret: !0,
                }],
                vec![FnTest {
                    params: vec![4, 4, 4],
                    ret: !0,
                }],
            ],
        },
        TypeTest {
            layout: get_type_layout::<many_params::Mod>(),
            vars_types: vec![
                get_tlc::<extern "C" fn()>(),
                get_tlc::<RString>(),
                get_tlc::<&mut u32>(),
                get_tlc::<u64>(),
                get_tlc::<()>(),
                get_tlc::<i8>(),
                get_tlc::<u32>(),
                get_tlc::<i64>(),
                get_tlc::<()>(),
                get_tlc::<()>(),
            ],
            field_types: vec![0, 0, 0, 0, 0],
            functions: vec![
                vec![FnTest {
                    params: vec![],
                    ret: 1,
                }],
                vec![FnTest {
                    params: vec![2, 3, 1, 5, 6, 7],
                    ret: !0,
                }],
                vec![FnTest {
                    params: vec![4, 7, 4, 4, 4],
                    ret: !0,
                }],
                vec![FnTest {
                    params: vec![6, 3, 5, 1],
                    ret: 2,
                }],
                vec![FnTest {
                    params: vec![4, 4, 4, 4, 8, 9],
                    ret: !0,
                }],
            ],
        },
    ];

    let test_layout = |field_layout, expected_layout| {
        let res = check_layout_compatibility(field_layout, expected_layout);

        assert!(
            res.is_ok(),
            "field:{} expected:{}",
            field_layout.full_type(),
            expected_layout.full_type(),
        );
    };

    let empty_vec = Vec::new();

    for ty_test in list {
        let shared_vars = ty_test.layout.shared_vars();
        let vars_types = &ty_test.vars_types;

        let fields = ty_test.layout.get_fields().unwrap();
        let mut expected_fns_list = ty_test.functions.iter();
        for (field_i, field) in fields.iter().enumerate() {
            let field_layout = field.layout();
            let expected_layout = {
                let x = ty_test.field_types[field_i];
                vars_types[x].get()
            };

            test_layout(field_layout, expected_layout);

            let mut expected_fns = expected_fns_list.next().unwrap_or(&empty_vec).iter();
            for field_func in field.function_range() {
                let expected_fn = expected_fns.next().unwrap();

                let mut expected_params = expected_fn.params.iter();

                for param in field_func.param_type_layouts.iter() {
                    let expected_param = expected_params
                        .next()
                        .and_then(|x| vars_types.get(*x))
                        .unwrap_or_else(|| panic!("mismatched parameter type: {}", field_func));
                    test_layout(param.get(), expected_param.get());
                }

                match (
                    field_func.return_type_layout,
                    vars_types.get(expected_fn.ret),
                ) {
                    (Some(found_ret), Some(expected_ret)) => {
                        test_layout(found_ret.get(), expected_ret.get());
                    }
                    (None, None) => {}
                    _ => panic!(
                        "mismatched return type: {}\n\
                         shared_vars.type_layouts:{:#?}\n\
                         found function parameter/ret:{:#?} {:?}
                         expected function parameter/ret:{:#?} {:?}
                         ",
                        field_func,
                        type_layouts_fmt(shared_vars.type_layouts().iter().cloned()),
                        type_layouts_fmt(field_func.param_type_layouts.iter()),
                        type_layouts_fmt(field_func.return_type_layout),
                        type_layouts_fmt(expected_fn.params.iter().map(|x| vars_types[*x])),
                        type_layouts_fmt(vars_types.get(expected_fn.ret).cloned()),
                    ),
                }
            }
        }
    }
}

#[cfg(not(miri))]
fn type_layouts_fmt(iter: impl IntoIterator<Item = TypeLayoutCtor>) -> Vec<String> {
    iter.into_iter()
        .map(|x| x.get().full_type().to_string())
        .collect::<Vec<String>>()
}
