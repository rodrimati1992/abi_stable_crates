use crate::{
    std_types::{RBox,RBoxError,RCow,RResult,ROption,RNone,ROk},
    type_layout::TypeLayout,
    traits::IntoReprC,
    rtry,
    sabi_trait,
    StableAbi,
};

use std::{
    error::Error as ErrorTrait,
    fmt::{self,Display},
};


#[sabi_trait]
pub unsafe trait TypeChecker{
    /// Checks that `ìnterface` is compatible with `implementation.` 
    fn check_compatibility(
        &mut self,
        interface:&'static TypeLayout,
        implementation:&'static TypeLayout,
    )->RResult<(), ExtraChecksError>;
}


/**
Allows defining extra checks for a type.


# Usage

To use a type to add extra checks follow these steps:

- Implement this trait for that type,

- Declare a `extern "C" fn()->ExtraChecksStaticRef` function,
    which constructs ExtraChecksStaticRef with `ExtraChecksStaticRef::from_ptr`.

- Derive StableAbi for some type,using the `#[sabi(extra_checks="the_function")]` attribute.

# Examples

### Alphabetic.

This defines an ExtraChecks which checks that fields are alphabetically sorted

```
use abi_stable::{
    abi_stability::{
        check_layout_compatibility,
        TypeChecker_TO,ExtraChecks,ExtraChecksStaticRef,ExtraChecksExt,ExtraChecksError
    },
    type_layout::TypeLayout,
    sabi_trait::prelude::TU_Opaque,
    std_types::{RCow,RDuration,RResult,RString,StaticStr},
    sabi_extern_fn,
    StableAbi,
};

use std::fmt::{self,Display};

fn main(){

    let rect_layout=<Rectangle as StableAbi>::LAYOUT;
    let person_layout=<Person as StableAbi>::LAYOUT;
    
    // This passes because the fields are in order
    check_layout_compatibility(rect_layout,rect_layout)
        .unwrap_or_else(|e| panic!("{}",e) );

    // This errors because the struct's fields aren't in order
    check_layout_compatibility(person_layout,person_layout)
        .unwrap_err();

}


#[repr(C)]
#[derive(StableAbi)]
#[sabi(extra_checks="get_in_order_checker")]
struct Rectangle{
    x:u32,
    y:u32,
    z:u32,
}


#[repr(C)]
#[derive(StableAbi)]
#[sabi(extra_checks="get_in_order_checker")]
struct Person{
    name:RString,
    surname:RString,
    age:RDuration,
}


/////////////////////////////////////////

#[sabi_extern_fn]
pub extern "C" fn get_in_order_checker()->ExtraChecksStaticRef{
    ExtraChecksStaticRef::from_ptr(
        &InOrderChecker,
        TU_Opaque,
    )
}


#[repr(C)]
#[derive(Debug,Clone,StableAbi)]
pub struct InOrderChecker;


impl Display for InOrderChecker{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        f.write_str("InOrderChecker: Checks that field names are sorted alphabetically.")
    }
}


impl ExtraChecks for InOrderChecker {
    fn type_layout(&self)->&'static TypeLayout{
        <Self as StableAbi>::LAYOUT
    }

    fn check_compatibility(
        &self,
        layout_containing_self:&'static TypeLayout,
        layout_containing_other:&'static TypeLayout,
        checker:TypeChecker_TO<'_,&mut ()>,
    )->RResult<(), ExtraChecksError> {
        Self::downcast_with_layout(layout_containing_other,checker,|_|{
            let fields=layout_containing_self.get_fields().unwrap_or_default();

            if fields.is_empty() {
                return Ok(());
            }

            let mut prev=fields.iter().next().unwrap();
            for curr in fields {
                if prev.name > curr.name {
                    return Err(OutOfOrderError{
                        previous_one:prev.name,
                        first_one:curr.name,
                    });
                }
                prev=curr;
            }
            Ok(())
        })
    }

    fn nested_type_layouts(&self)->RCow<'_,[&'static TypeLayout]>{
        RCow::from_slice(&[])
    }
}



#[derive(Debug,Clone)]
pub struct OutOfOrderError{
    previous_one:StaticStr,

    /// The first field that is out of order.
    first_one:StaticStr,
}

impl Display for OutOfOrderError{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        writeln!(
            f,
            "Expected fields to be alphabetically sorted.\n\
             Found field '{}' before '{}'\
            ",
            self.previous_one,
            self.first_one,
        )
    }
}

impl std::error::Error for OutOfOrderError{}


```

### Associated Constant.

This defines an ExtraChecks which checks that an associated constant is 
the same for both types.

```
use abi_stable::{
    abi_stability::{
        check_layout_compatibility,
        TypeChecker_TO,ExtraChecks,ExtraChecksStaticRef,ExtraChecksExt,ExtraChecksError
    },
    marker_type::UnsafeIgnoredType,
    type_layout::TypeLayout,
    sabi_trait::prelude::TU_Opaque,
    std_types::{RCow,RDuration,RResult,RString,StaticStr},
    sabi_extern_fn,
    GetStaticEquivalent,
    StableAbi,
};

use std::fmt::{self,Display};

fn main(){

    let const0= <WithConstant<N0> as StableAbi>::LAYOUT;
    let const_second_0= <WithConstant<SecondN0> as StableAbi>::LAYOUT;
    let const1= <WithConstant<N1> as StableAbi>::LAYOUT;
    let const2= <WithConstant<N2> as StableAbi>::LAYOUT;

    check_layout_compatibility(const0,const0).unwrap();
    check_layout_compatibility(const_second_0,const_second_0).unwrap();
    check_layout_compatibility(const1,const1).unwrap();
    check_layout_compatibility(const2,const2).unwrap();

    ////////////
    // WithConstant<SecondN0> and WithConstant<N0> are compatible with each other
    // because their `GetConstant::NUMBER` associated constant is the same value.
    check_layout_compatibility(const0,const_second_0).unwrap();
    check_layout_compatibility(const_second_0,const0).unwrap();

    
    ////////////
    // None of the lines bellow are compatible because their 
    // `GetConstant::NUMBER` associated constant isn't the same value.
    check_layout_compatibility(const0,const1).unwrap_err();
    check_layout_compatibility(const0,const2).unwrap_err();

    check_layout_compatibility(const1,const0).unwrap_err();
    check_layout_compatibility(const1,const2).unwrap_err();

    check_layout_compatibility(const2,const0).unwrap_err();
    check_layout_compatibility(const2,const1).unwrap_err();

}


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

impl<C> WithConstant<C>{
    const NEW:Self=Self{
        _marker:UnsafeIgnoredType::NEW,
    };
}

impl<C> WithConstant<C>
where 
    C:GetConstant
{
    const CHECKER:&'static ConstChecker=
        &ConstChecker{number:C::NUMBER};

    #[sabi_extern_fn]
    pub fn get_const_checker()->ExtraChecksStaticRef{
        ExtraChecksStaticRef::from_ptr(
            Self::CHECKER,
            TU_Opaque,
        )
    }
}


trait GetConstant{
    const NUMBER:u64;
}

#[derive(GetStaticEquivalent)]
struct N0;
impl GetConstant for N0{
    const NUMBER:u64=0;
}

#[derive(GetStaticEquivalent)]
struct SecondN0;
impl GetConstant for SecondN0{
    const NUMBER:u64=0;
}

#[derive(GetStaticEquivalent)]
struct N1;
impl GetConstant for N1{
    const NUMBER:u64=1;
}

#[derive(GetStaticEquivalent)]
struct N2;
impl GetConstant for N2{
    const NUMBER:u64=2;
}


/////////////////////////////////////////

#[repr(C)]
#[derive(Debug,Clone,StableAbi)]
pub struct ConstChecker{
    number:u64
}


impl Display for ConstChecker{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        writeln!(
            f,
            "ConstChecker: \
                Checks that the associated constant for \
                for the other type is {}.\
            ",
            self.number
        )
    }
}


impl ExtraChecks for ConstChecker {
    fn type_layout(&self)->&'static TypeLayout{
        <Self as StableAbi>::LAYOUT
    }

    fn check_compatibility(
        &self,
        layout_containing_self:&'static TypeLayout,
        layout_containing_other:&'static TypeLayout,
        checker:TypeChecker_TO<'_,&mut ()>,
    )->RResult<(), ExtraChecksError> {
        Self::downcast_with_layout(layout_containing_other,checker,|other|{
            if self.number==other.number {
                Ok(())
            }else{
                Err(UnequalConstError{
                    expected:self.number,
                    found:other.number,
                })
            }
        })
    }

    fn nested_type_layouts(&self)->RCow<'_,[&'static TypeLayout]>{
        RCow::from_slice(&[])
    }
}



#[derive(Debug,Clone)]
pub struct UnequalConstError{
    expected:u64,
    found:u64,
}

impl Display for UnequalConstError{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        writeln!(
            f,
            "Expected the `GetConstant::NUMBER` associated constant to be:\
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


```


# Combination

This is how an ExtraChecks can be combined across all 
dynamic libraries to ensure some property(which can be relied on for safety).

This is a very similar process to how abi_stable ensures that 
vtables and modules are consistent across dynamic libraries.

### Failure

Loading many libraries that contain ExtraChecks trait objects that need 
to be combined can fail if the representative version of the trait objects 
are incompatible with those of the library,
even if both the library and the binary are otherwise compatible.

The graphs bellow uses the `LIBRARY( ExtraChecks trait object )` format,
where the trait object is compatible only if the one in the binary 
is a prefix of the string in the library,
and all the libraries have a prefix of the same string.

This is fine:

```text
A("ab")<---B("abc")
\__________C("abcd")
```

This is not fine

```text
 __________D("abe")
/
A("ab")<---B("abc")
\__________C("abcd")
```


### Example

```

use abi_stable::{
    abi_stability::{
        check_layout_compatibility,
        TypeChecker_TO,
        ExtraChecks,ExtraChecksStaticRef,ExtraChecks_TO,CombineResult,
        ExtraChecksExt,ExtraChecksError
    },
    marker_type::UnsafeIgnoredType,
    type_layout::TypeLayout,
    sabi_trait::prelude::TU_Opaque,
    std_types::{RCow,RResult,RSome,StaticStr},
    sabi_extern_fn,
    GetStaticEquivalent,
    StableAbi,
};

use std::fmt::{self,Display};


const LAYOUT0:&'static TypeLayout= <WithConstant<V1_0> as StableAbi>::LAYOUT;
const LAYOUT1:&'static TypeLayout= <WithConstant<V1_1> as StableAbi>::LAYOUT;
const LAYOUT1B:&'static TypeLayout=<WithConstant<V1_1_Incompatible> as StableAbi>::LAYOUT;
const LAYOUT2:&'static TypeLayout= <WithConstant<V1_2> as StableAbi>::LAYOUT;


fn main(){
    // Compared LAYOUT0 to LAYOUT1B,stored LAYOUT0.extra_checks associated with both layouts.
    check_layout_compatibility(LAYOUT0,LAYOUT1B).unwrap();

    // Compared LAYOUT1 to LAYOUT2,stored LAYOUT2.extra_checks associated with both layouts.
    check_layout_compatibility(LAYOUT1,LAYOUT2).unwrap();

    // Compared LAYOUT0 to LAYOUT2:
    // - the comparison succeeded,
    // - then both are combined.
    // - The combined trait object is attempted to be combined with the
    //      ExtraChecks in the global map associated to both LAYOUT0 and LAYOUT2,
    //      which are LAYOUT1B.extra_checks and LAYOUT2.extra_checks respectively.
    // - Combining the trait objects with the ones in the global map fails because 
    //      the one from LAYOUT1B is incompatible with the one from LAYOUT2.
    check_layout_compatibility(LAYOUT0,LAYOUT2).unwrap_err();
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

impl<C> WithConstant<C>{
    const NEW:Self=Self{
        _marker:UnsafeIgnoredType::NEW,
    };
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

use self::constants::*;

#[allow(non_camel_case_types)]
mod constants{
    use super::*;
    
    #[derive(GetStaticEquivalent)]
    pub struct V1_0;

    impl GetConstant for V1_0{
        const CHARS:&'static str="ab";
    }


    #[derive(GetStaticEquivalent)]
    pub struct V1_1;

    impl GetConstant for V1_1{
        const CHARS:&'static str="abc";
    }


    #[derive(GetStaticEquivalent)]
    pub struct V1_1_Incompatible;

    impl GetConstant for V1_1_Incompatible{
        const CHARS:&'static str="abd";
    }


    #[derive(GetStaticEquivalent)]
    pub struct V1_2;

    impl GetConstant for V1_2{
        const CHARS:&'static str="abcd";
    }
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
        checker:TypeChecker_TO<'_,&mut ()>,
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
        other:ExtraChecks_TO<'static,&()>,
        checker:TypeChecker_TO<'_,&mut ()>
    )->CombineResult{
        Self::downcast_with_object(other,checker,|other|{
            let (min,max)=min_max_by(self,other,|x|x.chars.len());
            min.check_compatible_inner(max)
                .map(|_| RSome( ExtraChecks_TO::from_value(max.clone(),TU_Opaque) ) )
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


pub(crate) fn min_max_by<T,F,K>(l:T,r:T,mut f:F)->(T,T)
where 
    F:FnMut(&T)->K,
    K:Ord,
{
    if f(&l) < f(&r) {
        (l,r)
    }else{
        (r,l)
    }
}



```

*/
#[sabi_trait]
//#[sabi(debug_print_trait)]
pub unsafe trait ExtraChecks:Debug+Display+Clone{
    /// Gets the type layout of `Self`(the type that implements ExtraChecks)
    fn type_layout(&self)->&'static TypeLayout;

/**

Checks that `self` is compatible another type which implements ExtraChecks.



`layout_containing_self`:
The TypeLayout containing `self` in the extra_checks field.

`layout_containing_other`:
The TypeLayout containing the other ExtraChecks implementor that this is compared with,
in the extra_checks field.

*/
    fn check_compatibility(
        &self,
        layout_containing_self:&'static TypeLayout,
        layout_containing_other:&'static TypeLayout,
        checker:TypeChecker_TO<'_,&mut ()>,
    )->RResult<(), ExtraChecksError>;

    /// Returns the `TypeLayout`s owned or referenced by `self`.
    /// 
    /// This is necessary for the Debug implementation of `TypeLayout`.
    fn nested_type_layouts(&self)->RCow<'_,[&'static TypeLayout]>;

    /// Combines this ExtraChecks trait object with another one.
    ///
    /// To guarantee that the `extra_checks` 
    /// associated with a type (inside `<TheType as StableAbi>::LAYOUT.extra_cheks` )
    /// has a single representative value across all dynamic libraries,
    /// you must override this method,
    /// and return `ROk(RSome(_))` by combining `self` and `other` in some way.
    ///
    /// # Return value
    ///
    /// This returns:
    ///
    /// - `ROk(RNone)`: 
    ///     If `self` doesn't need to be unified across all dynamic libraries,
    ///     or the representative version doesn't need to be updated.
    ///
    /// - `ROk(RSome(_))`: 
    ///     If `self` needs to be unified across all dynamic libraries,
    ///     returning the combined `self` and `other`.
    ///
    /// - `RErr(_)`: If there was a problem unifying `self` and `other`.
    ///
    fn combine(
        &self,
        _other:ExtraChecks_TO<'static,&'_ ()>,
        _checker:TypeChecker_TO<'_,&'_ mut ()>
    )->CombineResult{
        ROk(RNone)
    }
}



/// An ffi-safe trait object that adds extra checks to a StableAbi type.
pub type ExtraChecksStaticRef=ExtraChecks_TO<'static,&'static ()>;

pub type ExtraChecksBox=ExtraChecks_TO<'static,RBox<()>>;

pub type CombineResult=RResult<ROption<ExtraChecksBox>, ExtraChecksError>;

/// An extension trait for `ExtraChecks` implementors.
pub trait ExtraChecksExt:StableAbi+ExtraChecks{
    fn downcast_with_layout<F,R,E>(
        layout_containing_other:&'static TypeLayout,
        checker:TypeChecker_TO<'_,&mut ()>,
        f:F,
    )->RResult<R, ExtraChecksError>
    where
        Self:'static,
        F:FnOnce(&Self)->Result<R,E>,
        E:Send+Sync+ErrorTrait+'static,
    {
        let other=rtry!(
            layout_containing_other.extra_checks().ok_or(ExtraChecksError::NoneExtraChecks)
        );

        Self::downcast_with_object(other,checker,f)
    }
    fn downcast_with_object<F,R,E>(
        other:ExtraChecks_TO<'static,&()>,
        mut checker:TypeChecker_TO<'_,&mut ()>,
        f:F,
    )->RResult<R, ExtraChecksError>
    where
        F:FnOnce(&Self)->Result<R,E>,
        E:Send+Sync+ErrorTrait+'static,
    {
        // This checks that the layouts of `this` and `other` are compatible,
        // so that calling the `unchecked_into_unerased` method is sound.
        rtry!( checker.check_compatibility(<Self as StableAbi>::LAYOUT,other.type_layout()) );
        let other_ue=unsafe{ other.obj.unchecked_into_unerased::<Self>() };

        f(other_ue).map_err(ExtraChecksError::from_extra_checks).into_c()
    }
}


impl<This> ExtraChecksExt for This
where
    This:?Sized+StableAbi+ExtraChecks
{}


///////////////////////////////////////////////////////////////////////////////


#[repr(u8)]
#[derive(Debug,StableAbi)]
pub enum ExtraChecksError{
    TypeChecker,
    /// When `extra_checks==Some(_)` in the interface type layout,
    /// but the `extra_checks==None` in the implementation type layout.
    NoneExtraChecks,
    ExtraChecks(RBoxError),
}


impl ExtraChecksError {
    pub fn from_extra_checks<E>(err:E)->ExtraChecksError
    where
        E:Send+Sync+ErrorTrait+'static,
    {
        let x=RBoxError::new(err);
        ExtraChecksError::ExtraChecks(x)
    }
}


impl Display for ExtraChecksError{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        match self {
            ExtraChecksError::TypeChecker=>
                Display::fmt("A type checker error happened.",f),
            ExtraChecksError::NoneExtraChecks=>
                Display::fmt("No `ExtraChecks` in the implementation.",f),
            ExtraChecksError::ExtraChecks(e)=>
                Display::fmt(e,f),
        }
    }
}

impl std::error::Error for ExtraChecksError{}



