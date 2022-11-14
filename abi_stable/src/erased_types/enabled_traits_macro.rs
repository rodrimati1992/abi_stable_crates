macro_rules! declare_enabled_traits {
    (
        auto_traits[
            $(($auto_trait:ident, $auto_trait_query:ident, $auto_trait_path:path)),* $(,)*
        ]

        regular_traits[
            $(($regular_trait:ident, $regular_trait_query:ident, $regular_trait_path:path)),* $(,)*
        ]
    ) => (
        use crate::{
            abi_stability::extra_checks::{
                TypeCheckerMut,ExtraChecks,
                ForExtraChecksImplementor,ExtraChecksError,
            },
            type_layout::TypeLayout,
            std_types::{RCowSlice,RResult},
        };

        use core_extensions::strings::StringExt;

        #[allow(non_upper_case_globals)]
        mod auto_trait_mask{
            #[repr(u32)]
            enum __Index {
                $($auto_trait,)*
            }
            $(pub(super) const $auto_trait: u16 = 1u16 << __Index::$auto_trait as u32;)*
        }

        #[allow(non_upper_case_globals)]
        mod regular_trait_mask{
            #[repr(u32)]
            enum __Index {
                $($regular_trait,)*
            }
            $(pub(super) const $regular_trait: u64 = 1u64 << __Index::$regular_trait as u32;)*
        }


        /// Describes which traits are required and enabled by the `I: `[`InterfaceType`]
        /// that this `RequiredTraits` is created from.
        ///
        /// # Purpose
        ///
        /// This is what [`DynTrait`] uses to check that the traits it
        /// requires are compatible between library versions,
        /// by using this type in
        /// [`#[sabi(extra_checks = <here>)]`](derive@crate::StableAbi#sabi_extra_checks_attr).
        ///
        /// This type requires that auto traits are the same across versions.
        /// Non-auto traits can be added in newer versions of a library.
        #[repr(C)]
        #[derive(Copy,Clone,StableAbi)]
        pub struct RequiredTraits{
            auto_traits:u16,
            regular_traits:u64,
        }

        impl RequiredTraits {
            /// Constructs an RequiredTraits.
            pub const fn new<I: InterfaceType>() -> Self {
                use crate::type_level::impl_enum::Implementability;

                RequiredTraits {
                    auto_traits: $(
                        if <I::$auto_trait as Implementability>::IS_IMPLD {
                            auto_trait_mask::$auto_trait
                        } else {
                            0
                        }
                    )|*,
                    regular_traits: $(
                        if <I::$regular_trait as Implementability>::IS_IMPLD {
                            regular_trait_mask::$regular_trait
                        } else {
                            0
                        }
                    )|*
                }
            }

            $(
                #[doc = concat!(
                    "Whether the [`",
                    stringify!($auto_trait_path),
                    "`] trait is required",
                )]
                pub const fn $auto_trait_query(self) -> bool {
                    (self.auto_traits & auto_trait_mask::$auto_trait) != 0
                }
            )*
            $(
                #[doc = concat!(
                    "Whether the [`",
                    stringify!($regular_trait_path),
                    "`] trait is required",
                )]
                pub const fn $regular_trait_query(self) -> bool {
                    (self.regular_traits & regular_trait_mask::$regular_trait) != 0
                }
            )*
        }

        impl Debug for RequiredTraits{
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
                use self::debug_impl_details::{EnabledAutoTraits,EnabledRegularTraits};

                f.debug_struct("RequiredTraits")
                 .field("auto_traits_bits",&self.auto_traits)
                 .field("auto_traits",&EnabledAutoTraits{traits:self.auto_traits})
                 .field("regular_traits_bits",&self.regular_traits)
                 .field("regular_traits",&EnabledRegularTraits{traits:self.regular_traits})
                 .finish()
            }
        }

        impl Display for RequiredTraits{
            fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
                f.write_str("RequiredTraits\n")?;

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


        unsafe impl ExtraChecks for RequiredTraits {
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

            fn nested_type_layouts(&self)->RCowSlice<'_, &'static TypeLayout>{
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
            expected:RequiredTraits,
            found:RequiredTraits,
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
