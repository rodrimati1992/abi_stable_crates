use abi_stable::{
    abi_stability::{
        abi_checking::{
            AbiInstability,CheckingGlobals,
            check_layout_compatibility_with_globals,
        },
        extra_checks::{
            TypeCheckerMut,
            ExtraChecks,ExtraChecksBox,ExtraChecksRef,
            StoredExtraChecks,ExtraChecks_MV,
            ForExtraChecksImplementor,ExtraChecksError,
        },
        stable_abi_trait::get_type_layout,
    },
    const_utils::abs_sub_usize,
    external_types::ROnce,
    marker_type::UnsafeIgnoredType,
    type_layout::TypeLayout,
    sabi_trait::prelude::TU_Opaque,
    sabi_types::{Constructor,CmpIgnored},
    std_types::*,
    utils::{self,leak_value},
    GetStaticEquivalent,
    StableAbi,
};

use std::{
    fmt::{self,Display},
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr,
};

use core_extensions::{matches,SelfOps};


#[cfg(not(miri))]
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
    fn asserts(errs: &[AbiInstability]){
        assert!(
            errs
            .iter()
            .any(|err| matches!(AbiInstability::ExtraCheckError{..}=err))
        );
    }

    #[cfg(not(miri))]
    check_subsets(&[LAYOUT0,LAYOUT1,LAYOUT2,LAYOUT3],asserts);
    
    #[cfg(miri)]
    {
        let globals=CheckingGlobals::new();

        assert_eq!(check_layout_compatibility_with_globals(LAYOUT0, LAYOUT0, &globals), Ok(()));
        
        asserts(
            &check_layout_compatibility_with_globals(LAYOUT1, LAYOUT0, &globals)
                .unwrap_err()
                .flatten_errors()  
        );
    }
}

#[cfg(not(miri))]
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


#[cfg(miri)]
#[test]
fn test_incompatible(){
    let globals=CheckingGlobals::new();

    check_layout_compatibility_with_globals(LAYOUT0,LAYOUT1,&globals).unwrap();
    check_layout_compatibility_with_globals(LAYOUT1,LAYOUT1B,&globals).unwrap_err();
}

//////////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(abi_stable::StableAbi)]
#[sabi(
    // Replaces the C:StableAbi constraint with `C:GetStaticEquivalent` 
    // (a supertrait of StableAbi).
    not_stableabi(C),
    bound="C:GetConstant",
    extra_checks="Self::CHECKER"
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
    const CHECKER:ConstChecker=
        ConstChecker{
            chars:RStr::from_str(C::CHARS)
        };
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
    chars:RStr<'static>,
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
    expected:RStr<'static>,
    found:RStr<'static>,
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
    type_layout:Constructor<&'static TypeLayout>,
}

impl Display for IdentityChecker{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Display::fmt(&self.type_layout,f)
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
            let t_lay=self.type_layout.get();
            let o_lay=other.type_layout.get();
            ty_checker.check_compatibility(t_lay,o_lay).into_result()
        }).observe(|res|{
            assert!(
                matches!(ROk(_)|RErr(ExtraChecksError::TypeChecker)=res),
                "It isn't either ROk or an RErr(TypeChecker):\n{:?}",
                res
            )
        })
    }

    fn nested_type_layouts(&self)->RCow<'_,[&'static TypeLayout]>{
        vec![self.type_layout.get()].into()
    }
}


#[repr(transparent)]
#[derive(abi_stable::StableAbi)]
struct Blah<T>(T);


struct WrapTypeLayout<T>(T);

impl<T> WrapTypeLayout<T>
where
    T:StableAbi
{
    const EXTRA_CHECKS:&'static ManuallyDrop<StoredExtraChecks>=
        &ManuallyDrop::new(StoredExtraChecks::from_const(
            &IdentityChecker{ 
                type_layout:Constructor(get_type_layout::<T>)
            },
            TU_Opaque,
            ExtraChecks_MV::VTABLE,
        ));
}

fn wrap_type_layout<T>()->&'static TypeLayout
where
    T:StableAbi
{
    <()>::LAYOUT.clone()
        ._set_extra_checks(
            WrapTypeLayout::<T>::EXTRA_CHECKS
            .piped(Some)
            .piped(CmpIgnored::new)
        )
        ._set_type_id(Constructor(
            abi_stable::std_types::utypeid::new_utypeid::<Blah<T::StaticEquivalent>>
        ))
        .piped(leak_value)
}


#[cfg(not(miri))]
#[test]
fn test_identity_extra_checker() {
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

    {
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
    }
}

#[cfg(miri)]
#[test]
fn test_identity_extra_checker() {
    let globals=CheckingGlobals::new();

    let interf0=wrap_type_layout::<RHashMap<i32,RString>>();
    let interf1=wrap_type_layout::<TypeLayout>();
    
    assert_eq!(check_layout_compatibility_with_globals(interf0, interf0, &globals), Ok(()));
    assert_ne!(check_layout_compatibility_with_globals(interf0, interf1, &globals), Ok(()));
}


//////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(abi_stable::StableAbi)]
#[sabi(extra_checks="Self::NEW")]
struct WithCyclicExtraChecker;

impl WithCyclicExtraChecker {
    const NEW:IdentityChecker=
        IdentityChecker{ type_layout:Constructor(get_type_layout::<Self>) };
}


/// This is used to check that ExtraChecks that contain the type that they are checking 
/// alway return an error.
#[test]
fn test_cyclic_extra_checker() {

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



#[repr(C)]
#[derive(abi_stable::StableAbi)]
#[sabi(extra_checks="Self::EXTRA_CHECKER")]
struct WithLocalExtraChecker<C0,C1>{
    _marker:UnsafeIgnoredType<(C0,C1)>
}

impl<C0,C1> WithLocalExtraChecker<C0,C1>
where
    C0:StableAbi,
    C1:StableAbi,
{
    const EXTRA_CHECKER:LocalExtraChecker=
        LocalExtraChecker{
            comp0:<C0 as StableAbi>::LAYOUT,
            comp1:<C1 as StableAbi>::LAYOUT,
        };
}


#[repr(C)]
#[derive(Debug,Clone,StableAbi)]
pub struct LocalExtraChecker{
    comp0:&'static TypeLayout,
    comp1:&'static TypeLayout,
}

impl Display for LocalExtraChecker{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        writeln!(
            f,
            "\ncomp0 type_layout:\n{}\ncomp1 type_layout:\n{}\n",
            self.comp0,
            self.comp1,
        )
    }
}


impl ExtraChecks for LocalExtraChecker {
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
            ty_checker.local_check_compatibility(self.comp0,other.comp0)
                .or_else(|_| ty_checker.local_check_compatibility(self.comp0,other.comp1) )
                .or_else(|_| ty_checker.local_check_compatibility(self.comp1,other.comp0) )
                .or_else(|_| ty_checker.local_check_compatibility(self.comp1,other.comp1) )
                .into_result()
        })
    }

    fn nested_type_layouts(&self)->RCow<'_,[&'static TypeLayout]>{
        vec![self.comp0,self.comp1].into()
    }
}




/// This is used to check that ExtraChecks that contain the type that they are checking 
/// alway return an error.
#[cfg(not(miri))]
#[test]
fn test_local_extra_checker() {

    use abi_stable::{
        external_types::ROnce,
    };

    let list = vec![
        WithLocalExtraChecker::<i32,u32>::LAYOUT,
        WithLocalExtraChecker::<RString,i32>::LAYOUT,
        WithLocalExtraChecker::<(),RString>::LAYOUT,
        WithLocalExtraChecker::<RVec<()>,()>::LAYOUT,
        WithLocalExtraChecker::<ROnce,RVec<()>>::LAYOUT,
    ];

    let globals=CheckingGlobals::new();

    for window in list.windows(2) {
        let res=check_layout_compatibility_with_globals(window[0], window[1], &globals);
        assert_eq!(res,Ok(()));
    }

    for (i, this) in list.iter().cloned().enumerate() {
        for (j, other) in list.iter().cloned().enumerate() {
            let res=check_layout_compatibility_with_globals(this, other, &globals);

            if abs_sub_usize(j,i)<=1 {
                assert_eq!(res, Ok(()), "j:{} i:{}",j,i);
            } else {
                assert_ne!(
                    res,
                    Ok(()),
                    "\n\nInterface:{:#?}\n\nimplementation:{:#?}",
                    this,
                    other,
                );

                let mut found_extra_checks_error=false;
                let mut found_name_error=false;
                for err in res.unwrap_err().flatten_errors().into_iter() {
                    match err {
                        AbiInstability::ExtraCheckError{..}=>found_extra_checks_error=true,
                        AbiInstability::Name{..}=>found_name_error=true,
                        _=>{}
                    }
                }

                assert!(
                    !found_extra_checks_error&&found_name_error,
                    "\n\nInterface:{:#?}\n\nimplementation:{:#?}",
                    this,
                    other,
                );
            }
        }
    }
}

#[cfg(miri)]
#[test]
fn test_local_extra_checker() {
    let globals=CheckingGlobals::new();
    
    let interf0=WithLocalExtraChecker::<i32,u32>::LAYOUT;
    let interf1=WithLocalExtraChecker::<RString,i32>::LAYOUT;
    let interf2=WithLocalExtraChecker::<(),RString>::LAYOUT;

    assert_eq!(check_layout_compatibility_with_globals(interf0, interf1, &globals), Ok(()));
    assert_ne!(check_layout_compatibility_with_globals(interf0, interf2, &globals), Ok(()));
}






