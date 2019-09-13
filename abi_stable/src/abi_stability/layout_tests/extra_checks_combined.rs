use crate::{
    abi_stability::{
        abi_checking::{
            AbiInstability,CheckingGlobals,
            check_layout_compatibility_with_globals,
        },
        TypeCheckerMut,
        ExtraChecks,ExtraChecksStaticRef,ExtraChecksBox,ExtraChecksRef,
        ForExtraChecksImplementor,ExtraChecksError
    },
    marker_type::UnsafeIgnoredType,
    type_layout::TypeLayout,
    sabi_trait::prelude::TU_Opaque,
    sabi_types::{Constructor,CmpIgnored},
    std_types::*,
    sabi_extern_fn,
    utils::{self,leak_value},
    GetStaticEquivalent,
    StableAbi,
};

use std::{
    fmt::{self,Display},
    marker::PhantomData,
    ptr,
};

use core_extensions::{matches,SelfOps};


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
        checker:TypeCheckerMut<'_>,
    )->RResult<(), ExtraChecksError> {
        Self::downcast_with_layout(layout_containing_other,checker,|other,_|{
            self.check_compatible_inner(other)
        })
    }

    fn nested_type_layouts(&self)->RCow<'_,[&'static TypeLayout]>{
        RCow::from_slice(&[])
    }

    fn combine(
        &self,
        other:ExtraChecksRef<'_>,
        checker:TypeCheckerMut<'_>
    )->RResult<ROption<ExtraChecksBox>, ExtraChecksError>{
        Self::downcast_with_object(other,checker,|other,_|{
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

//////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////


/// This is used to check that type checking within ExtraChecks works as expected.
#[repr(C)]
#[derive(Debug,Clone,StableAbi)]
pub struct IdentityChecker{
    type_layout:&'static TypeLayout,
}

impl Display for IdentityChecker{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Display::fmt(self.type_layout,f)
    }
}


impl ExtraChecks for IdentityChecker {
    fn type_layout(&self)->&'static TypeLayout{
        <Self as StableAbi>::LAYOUT
    }

    fn check_compatibility(
        &self,
        _layout_containing_self:&'static TypeLayout,
        layout_containing_other:&'static TypeLayout,
        ty_checker:TypeCheckerMut<'_>,
    )->RResult<(), ExtraChecksError> {
        Self::downcast_with_layout(layout_containing_other,ty_checker,|other,mut ty_checker|{
            ty_checker.check_compatibility(self.type_layout,other.type_layout).into_result()
        }).observe(|res|{
            assert!(
                matches!(ROk(_)|RErr(ExtraChecksError::TypeChecker)=res),
                "It isn't either ROk or an RErr(TypeChecker):\n{:?}",
                res
            )
        })
    }

    fn nested_type_layouts(&self)->RCow<'_,[&'static TypeLayout]>{
        std::slice::from_ref(&self.type_layout)
            .into()
    }
}


#[repr(transparent)]
#[derive(StableAbi)]
struct Blah<T>(T);

#[test]
fn test_identity_extra_checker() {

    struct MakeExtraChecks<T>(T);

    impl<T> MakeExtraChecks<T>
    where
        T:StableAbi
    {
        const NEW:&'static IdentityChecker=
            &IdentityChecker{ type_layout:T::LAYOUT };
        
        extern fn construct_extra_checks()->ExtraChecksStaticRef{
            ExtraChecksStaticRef::from_ptr(
                Self::NEW,
                TU_Opaque
            )
        }

    }

    fn wrap_type_layout<T>()->&'static TypeLayout
    where
        T:StableAbi
    {
        <()>::LAYOUT.clone()
            ._set_extra_checks(
                Constructor(MakeExtraChecks::<T>::construct_extra_checks)
                .piped(Some)
                .piped(CmpIgnored::new)
            )
            ._set_type_id(Constructor(
                crate::std_types::utypeid::new_utypeid::<Blah<T::StaticEquivalent>>
            ))
            .piped(leak_value)
    }

    use crate::{
        external_types::ROnce,
    };

    let list = vec![
        wrap_type_layout::<u32>(),
        wrap_type_layout::<[(); 0]>(),
        wrap_type_layout::<[(); 1]>(),
        wrap_type_layout::<[u32; 3]>(),
        wrap_type_layout::<i32>(),
        wrap_type_layout::<bool>(),
        wrap_type_layout::<&mut ()>(),
        wrap_type_layout::<&mut i32>(),
        wrap_type_layout::<&()>(),
        wrap_type_layout::<&i32>(),
        wrap_type_layout::<&'static &'static ()>(),
        wrap_type_layout::<&'static mut &'static ()>(),
        wrap_type_layout::<&'static &'static mut ()>(),
        wrap_type_layout::<ptr::NonNull<()>>(),
        wrap_type_layout::<ptr::NonNull<i32>>(),
        wrap_type_layout::<RHashMap<RString,RString>>(),
        wrap_type_layout::<RHashMap<RString,i32>>(),
        wrap_type_layout::<RHashMap<i32,RString>>(),
        wrap_type_layout::<RHashMap<i32,i32>>(),
        wrap_type_layout::<Option<&()>>(),
        wrap_type_layout::<Option<&u32>>(),
        wrap_type_layout::<Option<extern "C" fn()>>(),
        wrap_type_layout::<ROption<()>>(),
        wrap_type_layout::<ROption<u32>>(),
        wrap_type_layout::<RCow<'_, str>>(),
        wrap_type_layout::<RCow<'_, [u32]>>(),
        wrap_type_layout::<RArc<()>>(),
        wrap_type_layout::<RArc<u32>>(),
        wrap_type_layout::<RBox<()>>(),
        wrap_type_layout::<RBox<u32>>(),
        wrap_type_layout::<RBoxError>(),
        wrap_type_layout::<PhantomData<()>>(),
        wrap_type_layout::<PhantomData<RString>>(),
        wrap_type_layout::<ROnce>(),
        wrap_type_layout::<TypeLayout>(),
    ];

    let globals=CheckingGlobals::new();

    let (_dur, ()) = core_extensions::measure_time::measure(|| {
        for (i, this) in list.iter().cloned().enumerate() {
            for (j, other) in list.iter().cloned().enumerate() {
                let res=check_layout_compatibility_with_globals(this, other, &globals);

                if i == j {
                    assert_eq!(res, Ok(()));
                } else {
                    assert_ne!(
                        res,
                        Ok(()),
                        "\n\nInterface:{:#?}\n\nimplementation:{:#?}",
                        this,
                        other,
                    );

                    let found_extra_checks_error=res
                        .unwrap_err()
                        .flatten_errors()
                        .into_iter()
                        .any(|err| matches!(AbiInstability::ExtraCheckError{..}=err) );

                    assert!(!found_extra_checks_error);
                    

                }
            }
        }
    });
}

//////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(StableAbi)]
#[sabi(extra_checks="Self::construct_extra_checks")]
struct WithCyclicExtraChecker;

impl WithCyclicExtraChecker {
    const NEW:&'static IdentityChecker=
        &IdentityChecker{ type_layout:Self::LAYOUT };
    
    extern fn construct_extra_checks()->ExtraChecksStaticRef{
        ExtraChecksStaticRef::from_ptr(
            Self::NEW,
            TU_Opaque
        )
    }
}


/// This is used to check that ExtraChecks that contain the type that they are checking 
/// alway return an error.
#[test]
fn test_cyclic_extra_checker() {

    use crate::{
        external_types::ROnce,
    };

    let layout = <WithCyclicExtraChecker as StableAbi>::LAYOUT;

    let globals=CheckingGlobals::new();

    let res=check_layout_compatibility_with_globals(layout, layout, &globals);

    assert_ne!(res,Ok(()),"layout:{:#?}",layout);

    let found_extra_checks_error=res
        .unwrap_err()
        .flatten_errors()
        .into_iter()
        .any(|err| matches!(AbiInstability::CyclicTypeChecking{..}=err) );

    assert!(found_extra_checks_error);
}


//////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////

