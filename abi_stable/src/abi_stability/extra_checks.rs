use crate::{
    std_types::{RBoxError,RCow,RResult},
    type_layout::TypeLayout,
    traits::IntoReprC,
    rtry,
    sabi_trait,
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


#[sabi_trait]
pub unsafe trait ExtraChecks:Debug+Display+Clone+'static{
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
}


//TODO
pub type ExtraChecksRef=ExtraChecks_TO<'static,&'static ()>;


#[repr(transparent)]
#[derive(StableAbi)]
pub struct What(Option<crate::utils::Constructor<ExtraChecksRef>>);


/// An extension trait for `ExtraChecks` implementors.
pub trait ExtraChecksExt:ExtraChecks{
    fn with_both_extra_checks<F,R,E>(
        &self,
        layout_containing_other:&'static TypeLayout,
        mut checker:TypeChecker_TO<'_,&mut ()>,
        f:F,
    )->RResult<R, ExtraChecksError>
    where
        F:FnOnce(&'static Self)->Result<R,E>,
        E:Send+Sync+ErrorTrait+'static,
    {
        let other=rtry!(
            layout_containing_other.extra_checks().ok_or(ExtraChecksError::NoneExtraChecks)
        );

        // This checks that the layouts of `this` and `other` are compatible,
        // so that calling the `unchecked_into_unerased` method is sound.
        rtry!( checker.check_compatibility(self.type_layout(),other.type_layout()) );
        let other_ue=unsafe{ other.obj.unchecked_into_unerased::<Self>() };

        f(other_ue).map_err(ExtraChecksError::from_extra_checks).into_c()
    }
}


impl<This> ExtraChecksExt for This
where
    This:?Sized+ExtraChecks
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



