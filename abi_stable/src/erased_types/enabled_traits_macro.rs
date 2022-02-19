macro_rules! declare_enabled_traits {
    (declare_index;($value:expr,$ty:ty);$which_impl0:ident,$which_impl1:ident $(,$rest:ident)* )=>{
        pub const $which_impl0:$ty=$value;
        pub const $which_impl1:$ty=$value << 1;

        declare_enabled_traits!{declare_index;($value << 2,$ty); $($rest),* }
    };
    (declare_index;($value:expr,$ty:ty);$which_impl:ident)=>{
        pub const $which_impl:$ty=$value;
    };
    (declare_index;($value:expr,$ty:ty);)=>{

    };

    (
        auto_traits[
            $($auto_trait:ident),* $(,)*
        ]

        regular_traits[
            $($regular_trait:ident),* $(,)*
        ]
    ) => (
        use std::fmt::{self,Debug,Display};

        use crate::{
            abi_stability::extra_checks::{
                TypeCheckerMut,ExtraChecks,
                ForExtraChecksImplementor,ExtraChecksError,
            },
            type_layout::TypeLayout,
            std_types::{RCowSlice,RResult},
            StableAbi,
        };

        use core_extensions::strings::StringExt;

        #[allow(non_upper_case_globals)]
        pub mod auto_trait_mask{
            declare_enabled_traits!{declare_index;(1,u16); $($auto_trait),* }
        }

        #[allow(non_upper_case_globals)]
        pub mod regular_trait_mask{
            declare_enabled_traits!{declare_index;(1,u64); $($regular_trait),* }
        }


        #[repr(C)]
        #[derive(Copy,Clone,StableAbi)]
        pub struct EnabledTraits{
            pub auto_traits:u16,
            pub regular_traits:u64,
        }

        impl Debug for EnabledTraits{
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
                use self::debug_impl_details::{EnabledAutoTraits,EnabledRegularTraits};

                f.debug_struct("EnabledTraits")
                 .field("auto_traits_bits",&self.auto_traits)
                 .field("auto_traits",&EnabledAutoTraits{traits:self.auto_traits})
                 .field("regular_traits_bits",&self.regular_traits)
                 .field("regular_traits",&EnabledRegularTraits{traits:self.regular_traits})
                 .finish()
            }
        }

        impl Display for EnabledTraits{
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
                f.write_str("EnabledTraits\n")?;

                f.write_str("Auto traits:")?;
                if self.auto_traits==0 {
                    f.write_str("<no_traits>")?;
                }else{
                    $(
                        if (self.auto_traits&auto_trait_mask::$auto_trait)!=0 {
                            f.write_str(concat!(" ",stringify!($auto_trait)))?;
                        }
                    )*
                }
                writeln!(f,)?;

                f.write_str("Impld traits:")?;
                if self.regular_traits==0 {
                    f.write_str("<no_traits>")?;
                }else{
                    $(
                        if (self.regular_traits&regular_trait_mask::$regular_trait)!=0 {
                            f.write_str(concat!(" ",stringify!($regular_trait)))?;
                        }
                    )*
                }
                writeln!(f,)?;

                Ok(())
            }
        }


        unsafe impl ExtraChecks for EnabledTraits {
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
                    if self.auto_traits!=other.auto_traits {
                        Err(ImpldTraitsError{
                            kind:ImpldTraitsErrorKind::MismatchedAutoTraits,
                            expected:self.clone(),
                            found:other.clone(),
                        })
                    }else if (self.regular_traits&other.regular_traits)!=self.regular_traits {
                        Err(ImpldTraitsError{
                            kind:ImpldTraitsErrorKind::UnimpldTraits,
                            expected:self.clone(),
                            found:other.clone(),
                        })
                    }else{
                        Ok(())
                    }
                })
            }

            fn nested_type_layouts(&self) -> RCowSlice<'_, &'static TypeLayout>{
                RCowSlice::from_slice(&[])
            }
        }

        mod debug_impl_details{
            use super::*;

            pub(super) struct EnabledAutoTraits{
                pub(super) traits:u16,
            }

            impl Debug for EnabledAutoTraits{
                fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
                    let mut ds=f.debug_set();
                    $(
                        if (self.traits&auto_trait_mask::$auto_trait)!=0 {
                            ds.entry(&stringify!( $auto_trait ));
                        }
                    )*
                    ds.finish()
                }
            }


            pub(super) struct EnabledRegularTraits{
                pub(super) traits:u64,
            }

            impl Debug for EnabledRegularTraits{
                fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
                    let mut ds=f.debug_set();
                    $(
                        if (self.traits&regular_trait_mask::$regular_trait)!=0 {
                            ds.entry(&stringify!( $regular_trait ));
                        }
                    )*
                    ds.finish()
                }
            }
        }


        ////////////////////////////////////////////////////////////////////////


        #[derive(Debug,Clone)]
        pub struct ImpldTraitsError{
            kind:ImpldTraitsErrorKind,
            expected:EnabledTraits,
            found:EnabledTraits,
        }

        #[derive(Debug,Clone)]
        pub enum ImpldTraitsErrorKind{
            MismatchedAutoTraits,
            UnimpldTraits,
        }

        impl Display for ImpldTraitsError{
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{

                let msg=match self.kind {
                    ImpldTraitsErrorKind::MismatchedAutoTraits=>
                        "Expected auto traits to be exactly the same",
                    ImpldTraitsErrorKind::UnimpldTraits=>
                        "`Expected` does not contain a subset of the traits in`Found`",
                };
                f.write_str(msg)?;
                writeln!(f,)?;

                writeln!(f,"Expected:\n{}",self.expected.to_string().left_padder(4))?;
                writeln!(f,"Found:\n{}",self.found.to_string().left_padder(4))?;

                Ok(())
            }
        }

        impl std::error::Error for ImpldTraitsError{}

    )
}
