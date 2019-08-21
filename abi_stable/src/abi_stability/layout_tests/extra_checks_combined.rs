use crate::{
    abi_stability::{
        abi_checking::{AbiInstability,CheckingGlobals,check_layout_compatibility_with_globals},
        TypeCheckerMut,
        ExtraChecks,ExtraChecksStaticRef,ExtraChecksBox,ExtraChecksRef,
        ForExtraChecksImplementor,ExtraChecksError
    },
    marker_type::UnsafeIgnoredType,
    type_layout::TypeLayout,
    sabi_trait::prelude::TU_Opaque,
    std_types::{RCow,RResult,ROption,RSome,StaticStr},
    sabi_extern_fn,
    utils,
    GetStaticEquivalent,
    StableAbi,
};

use std::fmt::{self,Display};

use core_extensions::matches;


fn check_subsets<F>(list:&[&'static TypeLayout],mut f:F)
where
    F:FnMut(&[AbiInstability])
{
    let globals=CheckingGlobals::new();
    for (l_i,l_abi) in list.iter().enumerate() {
        for (r_i,r_abi) in list.iter().enumerate() {

            let res=check_layout_compatibility_with_globals(l_abi,r_abi,&globals);

            if l_i <= r_i {
                assert_eq!(res,Ok(()),"\n\nl_i:{} r_i:{}\n\n",l_i,r_i);
            }else{
                if let Ok(_)=res {
                    let _=dbg!(l_i);
                    let _=dbg!(r_i);
                }
                let errs=res.unwrap_err().flatten_errors();

                f(&*errs);
            }
        }
    }
}


const LAYOUT0:&'static TypeLayout= <WithConstant<V1_0> as StableAbi>::LAYOUT;
const LAYOUT1:&'static TypeLayout= <WithConstant<V1_1> as StableAbi>::LAYOUT;
const LAYOUT1B:&'static TypeLayout=<WithConstant<V1_1_Incompatible> as StableAbi>::LAYOUT;
const LAYOUT2:&'static TypeLayout= <WithConstant<V1_2> as StableAbi>::LAYOUT;
const LAYOUT3:&'static TypeLayout= <WithConstant<V1_3> as StableAbi>::LAYOUT;
const LAYOUT3B:&'static TypeLayout= <WithConstant<V1_3_Incompatible> as StableAbi>::LAYOUT;


#[test]
fn test_subsets(){
    check_subsets(&[LAYOUT0,LAYOUT1,LAYOUT2,LAYOUT3],|errs|{
        assert!(
            errs
            .iter()
            .any(|err| matches!(AbiInstability::ExtraCheckError{..}=err))
        );
    });
}

#[test]
fn test_incompatible(){
    {
        let globals=CheckingGlobals::new();

        check_layout_compatibility_with_globals(LAYOUT0,LAYOUT1,&globals).unwrap();
        check_layout_compatibility_with_globals(LAYOUT1,LAYOUT1B,&globals).unwrap_err();
        check_layout_compatibility_with_globals(LAYOUT1B,LAYOUT1B,&globals).unwrap();
        check_layout_compatibility_with_globals(LAYOUT1,LAYOUT2,&globals).unwrap();
    }
    {
        let globals=CheckingGlobals::new();

        check_layout_compatibility_with_globals(LAYOUT1,LAYOUT2,&globals).unwrap();
        check_layout_compatibility_with_globals(LAYOUT0,LAYOUT1,&globals).unwrap();
    }
    {
        let globals=CheckingGlobals::new();

        check_layout_compatibility_with_globals(LAYOUT0,LAYOUT1B,&globals).unwrap();
        check_layout_compatibility_with_globals(LAYOUT1,LAYOUT2,&globals).unwrap();
        check_layout_compatibility_with_globals(LAYOUT0,LAYOUT2,&globals).unwrap_err();
    }
    {
        let globals=CheckingGlobals::new();

        check_layout_compatibility_with_globals(LAYOUT0,LAYOUT3,&globals).unwrap();
        check_layout_compatibility_with_globals(LAYOUT0,LAYOUT3B,&globals).unwrap_err();

        check_layout_compatibility_with_globals(LAYOUT1,LAYOUT3B,&globals).unwrap();
        check_layout_compatibility_with_globals(LAYOUT1,LAYOUT3,&globals).unwrap_err();

        check_layout_compatibility_with_globals(LAYOUT2,LAYOUT3,&globals).unwrap();
        check_layout_compatibility_with_globals(LAYOUT3,LAYOUT3B,&globals).unwrap_err();
        
        check_layout_compatibility_with_globals(LAYOUT0,LAYOUT1,&globals).unwrap_err();
        check_layout_compatibility_with_globals(LAYOUT1,LAYOUT2,&globals).unwrap_err();

        check_layout_compatibility_with_globals(LAYOUT0,LAYOUT0,&globals).unwrap();
        check_layout_compatibility_with_globals(LAYOUT1,LAYOUT1,&globals).unwrap();
        check_layout_compatibility_with_globals(LAYOUT2,LAYOUT2,&globals).unwrap();
        check_layout_compatibility_with_globals(LAYOUT3,LAYOUT3,&globals).unwrap();
        check_layout_compatibility_with_globals(LAYOUT3B,LAYOUT3B,&globals).unwrap();
        

    }
}



//////////////////////////////////////////////////////////////////////////////////



#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    // Replaces the C:StableAbi constraint with `C:GetStaticEquivalent` 
    // (a supertrait of StableAbi).
    not_stableabi(C),
    bound="C:GetConstant",
    extra_checks="Self::get_const_checker"
)]
struct WithConstant<C>{
    // UnsafeIgnoredType is equivalent to PhantomData,
    // except that all `UnsafeIgnoredType` are considered the same type by `StableAbi`.
    _marker:UnsafeIgnoredType<C>,
}

impl<C> WithConstant<C>
where 
    C:GetConstant
{
    const CHECKER:&'static ConstChecker=
        &ConstChecker{
            chars:StaticStr::new(C::CHARS)
        };

    #[sabi_extern_fn]
    pub fn get_const_checker()->ExtraChecksStaticRef{
        ExtraChecksStaticRef::from_ptr(
            Self::CHECKER,
            TU_Opaque,
        )
    }
}


trait GetConstant{
    const CHARS:&'static str;
}

macro_rules! declare_consts {
    (
        $( const $ty:ident = $slice:expr ; )*
    ) => (
        $(
            #[derive(GetStaticEquivalent)]
            struct $ty;

            impl GetConstant for $ty{
                const CHARS:&'static str=$slice;
            }
        )*
    )
}

declare_consts!{
    const V1_0="ab";
    const V1_1="abc";
    const V1_1_Incompatible="abd";
    const V1_2="abcd";
    const V1_3="abcde";
    const V1_3_Incompatible="abcdf";
}



/////////////////////////////////////////

#[repr(C)]
#[derive(Debug,Clone,StableAbi)]
pub struct ConstChecker{
    chars:StaticStr,
}


impl Display for ConstChecker{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        writeln!(
            f,
            "ConstChecker: \
                Checks that the associated constant for \
                the other type is compatible with:\n{}\n.\
            ",
            self.chars
        )
    }
}


impl ConstChecker {
    fn check_compatible_inner(&self,other:&ConstChecker)->Result<(), UnequalConstError> {
        if other.chars.starts_with(&*self.chars) {
            Ok(())
        }else{
            Err(UnequalConstError{
                expected:self.chars,
                found:other.chars,
            })
        }
    }
}
impl ExtraChecks for ConstChecker {
    fn type_layout(&self)->&'static TypeLayout{
        <Self as StableAbi>::LAYOUT
    }

    fn check_compatibility(
        &self,
        _layout_containing_self:&'static TypeLayout,
        layout_containing_other:&'static TypeLayout,
        checker:TypeCheckerMut<'_,'_>,
    )->RResult<(), ExtraChecksError> {
        Self::downcast_with_layout(layout_containing_other,checker,|other|{
            self.check_compatible_inner(other)
        })
    }

    fn nested_type_layouts(&self)->RCow<'_,[&'static TypeLayout]>{
        RCow::from_slice(&[])
    }

    fn combine(
        &self,
        other:ExtraChecksRef<'_>,
        checker:TypeCheckerMut<'_,'_>
    )->RResult<ROption<ExtraChecksBox>, ExtraChecksError>{
        Self::downcast_with_object(other,checker,|other|{
            let (min,max)=utils::min_max_by(self,other,|x|x.chars.len());
            min.check_compatible_inner(max)
                .map(|_| RSome( ExtraChecksBox::from_value(max.clone(),TU_Opaque) ) )
        })
    }
}



#[derive(Debug,Clone)]
pub struct UnequalConstError{
    expected:StaticStr,
    found:StaticStr,
}

impl Display for UnequalConstError{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        writeln!(
            f,
            "Expected the `GetConstant::CHARS` associated constant to be compatible with:\
             \n    {}\
             \nFound:\
             \n    {}\
            ",
            self.expected,
            self.found,
        )
    }
}

impl std::error::Error for UnequalConstError{}
