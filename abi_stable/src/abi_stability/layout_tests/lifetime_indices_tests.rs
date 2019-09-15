#[allow(unused_imports)]
use core_extensions::{SelfOps,matches};

#[allow(unused_imports)]
use crate::std_types::{Tuple1,Tuple2,Tuple3,Tuple4};

use crate::{
    std_types::{RStr,RSlice},
    type_layout::{
        LifetimeArrayOrSlice,
        LifetimeIndex,
        LifetimeIndexPair,
        LifetimeIndexPair as LAP,
        LifetimeRange,
        TypeLayout,
    },
    StableAbi,
};


const LR0:LifetimeIndex=LifetimeIndex::Param(0);
const LR1:LifetimeIndex=LifetimeIndex::Param(1);
const LR2:LifetimeIndex=LifetimeIndex::Param(2);
const LR3:LifetimeIndex=LifetimeIndex::Param(3);
const LRA:LifetimeIndex=LifetimeIndex::ANONYMOUS;
const LRS:LifetimeIndex=LifetimeIndex::STATIC;
const LRN:LifetimeIndex=LifetimeIndex::NONE;


#[derive(Debug)]
pub struct LRTestParam{
    pub layout:&'static TypeLayout,
    pub shared_vars_lifetimes:Vec<LifetimeIndexPair>,
    pub paramret_lifetimes:Vec<LifetimeRange>,
    pub field_lt_indices:Vec<Vec<LifetimeIndexPair>>,
}




mod loads_of_params{
    use super::*;
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct Struct{
        func:for<'a> extern 
            fn(&'a u8,&u8,&u8,&u8,&u8,&u8,&u8,&u8,&u8,&u8,&u8,&u8,&u8,&u8)->&'a u8,
    }
}

mod loads_of_lifetimes_single_param{
    use super::*;
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct Struct{
        func:for<'a> extern fn(&'a&&&&&u8)->&'a u8,
    }
}

mod four_lifetimes_single_param{
    use super::*;
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct Struct{
        func:for<'a> extern fn(&'a&&u8)->&'a u8,
    }
}

mod three_lifetimes_single_param{
    use super::*;
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct Struct{
        func:for<'a> extern fn(&'a&u8)->&'a u8,
    }
}

mod lifetimes_rep_a_single_param{
    use super::*;
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct Struct{
        func:for<'a,'b> extern fn(&'a&'a&'b&u8)->&'b u8,
    }
}

mod lifetimes_rep_b_single_param{
    use super::*;
    #[repr(C)]
    #[derive(StableAbi)]
    pub struct Struct{
        func:for<'a> extern fn(&'a&'a u8),
    }
}

mod lifetimes_four_params{
    use super::*;
    #[repr(C)]
    #[derive(StableAbi)]
    // #[sabi(debug_print)]
    pub struct Struct<'lt0,'lt1:'lt0>{
        reference_a:&'static (),
        reference_b:&'lt0 (),
        reference_c:&'lt0 &'lt1 &'lt1 &'lt1 &'static &'static (),
        func_abba:for<'a,'b> 
            extern fn( &'b (), &'static (), RSlice<'a,RStr<'_>>, RStr<'a>)->&'static (),
    }

}

mod many_bound_lifetimes{
    use super::*;
    #[repr(C)]
    #[derive(StableAbi)]
    // #[sabi(debug_print)]
    pub struct Struct<'lt0,'lt1>{
        func_abba:for<'a,'b,'c,'d,'e,'f,'g,'h,'i,'j,'k,'l,'m,'n,'o> 
            extern fn(
                &'a (),&'b (),
                &'c (),&'d (),
                &'e (),&'f (),
                &'g (),&'h (),
                &'i (),&'j (),
                &'k (),&'l (),
                &'m (),&'n (),
                &'o (),&'o ()
            ),
        _marker:std::marker::PhantomData<Tuple2<
            Tuple4<&'lt0(),&'lt1(),&'static(),&'lt1()>,
            Tuple4<&'lt0(),&'static(),&'static(),&'lt1()>,
        >>
    }
}

mod many_bound_lifetimes_b{
    use super::*;
    #[repr(C)]
    #[derive(StableAbi)]
    // #[sabi(debug_print)]
    pub struct Struct<'lt0,'lt1>{
        func_abba:for<'a,'b,'c,'d,'e,'f,'g,'h,'i,'j,'k,'l,'m,'n,'o,'p> 
            extern fn(
                &'a (),&'b (),
                &'c (),&'d (),
                &'e (),&'f (),
                &'g (),&'h (),
                &'i (),&'j (),
                &'k (),&'l (),
                &'m (),&'n (),
                &'o (),&'o (),
                &'p (),&'static (),
                &'p (),&'lt0 (),
            ),
        _marker:std::marker::PhantomData<Tuple2<
            Tuple4<&'lt0(),&'lt1(),&'static(),&'lt1()>,
            Tuple4<&'lt0(),&'static(),&'static(),&'lt1()>,
        >>
    }
}

mod nested_fn_pointer{
    use super::*;
    #[repr(C)]
    #[derive(StableAbi)]
    // #[sabi(debug_print)]
    pub struct Struct<'lt0,'lt1:'lt0>{
        funcs:Tuple2<
            Tuple2<
                &'lt0 &'static (),
                for<'a,'b> extern fn(&'a &'a &'a (),RStr<'b>,&'lt0 &'lt1 (),&()),
            >,
            Tuple2<
               &'lt0 &'lt0 &'lt1 &'static &'static (),
                for<'a,'b> extern fn(&'b &'b &'a (),RStr<'a>,&&'lt1 (),&()),
            >,
        >,
        hello: Tuple4<
            &'lt0 RSlice<'lt1,RStr<'static>>,
            &'static (),
            &'lt0 (),
            &'lt1 (),
        >,
        world:Tuple3<
            &'lt0 RSlice<'lt1,RStr<'static>>,
            &'static (),
            &'lt0 (),
        >,
        func_b:for<'a,'b,'c,'d,'e,'f,'g,'h,'i,'j,'k,'l,'m,'n,'o> 
            extern fn(
                &'a (),&'b (),
                &'c (),&'d (),
                &'e (),&'f (),
                &'g (),&'h (),
                &'i (),&'j (),
                &'o (),&'o (),
                &'k (),&'l (),
                &'m (),&'n (),
            ),
    }
}

#[test]
fn test_single_function_lifetime_ranges(){

    let list=vec![
        LRTestParam{
            layout:<loads_of_params::Struct as StableAbi>::LAYOUT,
            shared_vars_lifetimes:vec![
                LAP::new(LR0,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LR0,LRN),
            ],
            paramret_lifetimes:vec![LifetimeRange::from_range(0..8)],
            field_lt_indices:vec![vec![]],
        },
        LRTestParam{
            layout:<loads_of_lifetimes_single_param::Struct as StableAbi>::LAYOUT,
            shared_vars_lifetimes:vec![
                LAP::new(LR0,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LR0,LRN),
            ],
            paramret_lifetimes:vec![LifetimeRange::from_range(0..4)],
            field_lt_indices:vec![vec![]],
        },
        LRTestParam{
            layout:<four_lifetimes_single_param::Struct as StableAbi>::LAYOUT,
            shared_vars_lifetimes:vec![],
            paramret_lifetimes:vec![LifetimeRange::from_array([LR0,LRA,LRA,LR0,LRN])],
            field_lt_indices:vec![vec![]],
        },
        LRTestParam{
            layout:<three_lifetimes_single_param::Struct as StableAbi>::LAYOUT,
            shared_vars_lifetimes:vec![],
            paramret_lifetimes:vec![LifetimeRange::from_array([LR0,LRA,LR0,LRN,LRN])],
            field_lt_indices:vec![vec![]],
        },
        LRTestParam{
            layout:<lifetimes_rep_a_single_param::Struct as StableAbi>::LAYOUT,
            shared_vars_lifetimes:vec![],
            paramret_lifetimes:vec![LifetimeRange::from_array([LR0,LR0,LR1,LRA,LR1])],
            field_lt_indices:vec![vec![]],
        },
        LRTestParam{
            layout:<lifetimes_rep_b_single_param::Struct as StableAbi>::LAYOUT,
            shared_vars_lifetimes:vec![],
            paramret_lifetimes:vec![LifetimeRange::from_array([LR0,LR0,LRN,LRN,LRN])],
            field_lt_indices:vec![vec![]],
        },
        LRTestParam{
            layout:<lifetimes_four_params::Struct as StableAbi>::LAYOUT,
            shared_vars_lifetimes:vec![
                LAP::new(LR0,LR1),
                LAP::new(LR1,LR1),
                LAP::new(LRS,LRS),
                LAP::new(LRA,LRS),
                LAP::new(LR2,LRA),
                LAP::new(LR2,LRS),                
            ],
            paramret_lifetimes:vec![LifetimeRange::from_range(3..6)],
            field_lt_indices:vec![
                vec![LAP::new(LRS,LRN)],
                vec![LAP::new(LR0,LRN)],
                vec![LAP::new(LR0,LR1),LAP::new(LR1,LR1),LAP::new(LRS,LRS)],
                vec![],
            ],
        },
        LRTestParam{
            layout:<many_bound_lifetimes::Struct as StableAbi>::LAYOUT,
            shared_vars_lifetimes:vec![
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LR2,LR2),
                LAP::new(LR0,LR1),
                LAP::new(LRS,LR1),
                LAP::new(LR0,LRS),
                LAP::new(LRS,LR1),
            ],
            paramret_lifetimes:vec![LifetimeRange::from_range(0..8)],
            field_lt_indices:vec![
                vec![],
                vec![
                    LAP::new(LR0,LR1),
                    LAP::new(LRS,LR1),
                    LAP::new(LR0,LRS),
                    LAP::new(LRS,LR1),
                ],
            ],
        },
        LRTestParam{
            layout:<many_bound_lifetimes_b::Struct as StableAbi>::LAYOUT,
            shared_vars_lifetimes:vec![
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LR2,LR2),
                LAP::new(LR3,LRS),
                LAP::new(LR3,LR0),
                LAP::new(LR0,LR1),
                LAP::new(LRS,LR1),
                LAP::new(LR0,LRS),
                LAP::new(LRS,LR1),
            ],
            paramret_lifetimes:vec![LifetimeRange::from_range(0..10)],
            field_lt_indices:vec![
                vec![],
                vec![
                    LAP::new(LR0,LR1),
                    LAP::new(LRS,LR1),
                    LAP::new(LR0,LRS),
                    LAP::new(LRS,LR1),
                ],
            ],
        },
        LRTestParam{
            layout:<nested_fn_pointer::Struct as StableAbi>::LAYOUT,
            shared_vars_lifetimes:vec![
                // funcs field lifetimes(outside function pointers)
                LAP::new(LR0,LRS),
                LAP::new(LR0,LR0),
                LAP::new(LR1,LRS),
                LAP::new(LRS,LRN),
                // Function pointer 0 lifetiems
                LAP::new(LR2,LR2),
                LAP::new(LR2,LRA),
                LAP::new(LR0,LR1),
                LAP::new(LRA,LRN),
                // Function pointer 1 lifetiems
                LAP::new(LR3,LR3),
                LAP::new(LR2,LR2),
                LAP::new(LRA,LR1),
                LAP::new(LRA,LRN),
                // hello field
                LAP::new(LR0,LR1),
                LAP::new(LRS,LRS),
                LAP::new(LR0,LR1),
                // func_b function pointer
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),
                LAP::new(LR2,LR2),
                LAP::new(LRA,LRA),
                LAP::new(LRA,LRA),

            ],
            paramret_lifetimes:vec![
                LifetimeRange::from_range(4..8),
                LifetimeRange::from_range(8..12),
                LifetimeRange::from_range(15..23),
            ],
            field_lt_indices:vec![
                vec![
                    LAP::new(LR0,LRS),
                    LAP::new(LR0,LR0),
                    LAP::new(LR1,LRS),
                    LAP::new(LRS,LRN),
                ],
                vec![
                    LAP::new(LR0,LR1),
                    LAP::new(LRS,LRS),
                    LAP::new(LR0,LR1),
                ],
                vec![
                    LAP::new(LR0,LR1),
                    LAP::new(LRS,LRS),
                    LAP::new(LR0,LRN),
                ],
                vec![],
            ],
        },
    ];

    for test in list {
        assert_eq!(
            test.layout.shared_vars().lifetime_indices(),
            &test.shared_vars_lifetimes[..],
            "module_path:{}",
            test.layout.mod_path(),
        );

        let fields=test.layout.get_fields().unwrap();
        let functions=fields.iter().flat_map(|f| f.function_range() );

        for (func,paramret_lifetimes) in functions.zip(test.paramret_lifetimes){
            assert_eq!(
                &func.paramret_lifetime_indices[..],
                &paramret_lifetimes.slicing(&test.shared_vars_lifetimes[..])[..],
                "module_path:{}",
                test.layout.mod_path(),
            );
        }

        let iter=fields.iter().zip(test.field_lt_indices).enumerate();
        for (field_i,(field,expected_lt_indices)) in iter {
            let lifetime_indices=field.lifetime_indices();
            assert_eq!(
                &lifetime_indices[..],
                &expected_lt_indices[..],
                "\nfield_i:{}\nfield_name:{}\nmod_path:{}\n",
                field_i,
                field.name(),
                test.layout.line(),
            );

            assert_eq!(
                lifetime_indices.len()<=2 || 
                lifetime_indices.len()==3 && lifetime_indices[2].second()==LRN ,
                matches!(LifetimeArrayOrSlice::Array{..}=lifetime_indices),
            );
        }
    }
}