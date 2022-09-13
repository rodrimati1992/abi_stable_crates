//! Contains items for adding checks to individual types.
//!
//! # Implementing and using ExtraChecks
//!
//! To add extra checks to a type follow these steps:
//!
//! - Create some type and implement ExtraChecks for it,
//!
//! - Apply the `#[sabi(extra_checks = const expression that implements ExtraChecks)]`
//!     attribute to a type that uses `#[derive(StableAbi)]`.
//!
//! # Combination
//!
//! This is how an ExtraChecks can be combined across all
//! dynamic libraries to ensure some property(which can be relied on for safety).
//!
//! This is a very similar process to how abi_stable ensures that
//! vtables and modules are consistent across dynamic libraries.
//!
//! ### Failure
//!
//! Loading many libraries that contain ExtraChecks trait objects that need
//! to be combined can fail if the representative version of the trait objects
//! are incompatible with those of the library,
//! even if both the library and the binary are otherwise compatible.
//!
//! The graphs below uses the `LIBRARY( ExtraChecks trait object )` format,
//! where the trait object is compatible only if the one in the binary
//! is a prefix of the string in the library,
//! and all the libraries have a prefix of the same string.
//!
//! This is fine:
//!
//! ```text
//! A("ab")<---B("abc")
//! \__________C("abcd")
//! ```
//!
//! This is not fine
//!
//! ```text
//!  __________D("abe")
//! /
//! A("ab")<---B("abc")
//! \__________C("abcd")
//! ```
//!
//! The case that is not fine happens when the `ExtraChecks_TO::combine` method returned an error.
//!
//!
//! ### Example
//!
//! ```
//!
//! use abi_stable::{
//!     abi_stability::{
//!         check_layout_compatibility,
//!         extra_checks::{
//!             ExtraChecks, ExtraChecksBox, ExtraChecksError, ExtraChecksRef,
//!             ExtraChecksStaticRef, ForExtraChecksImplementor, TypeCheckerMut,
//!         },
//!     },
//!     marker_type::UnsafeIgnoredType,
//!     sabi_extern_fn,
//!     sabi_trait::prelude::TD_Opaque,
//!     std_types::{RCow, RCowSlice, ROption, RResult, RSome, RStr},
//!     type_layout::TypeLayout,
//!     GetStaticEquivalent, StableAbi,
//! };
//!
//! use std::fmt::{self, Display};
//!
//! const LAYOUT0: &'static TypeLayout = <WithConstant<V1_0> as StableAbi>::LAYOUT;
//! const LAYOUT1: &'static TypeLayout = <WithConstant<V1_1> as StableAbi>::LAYOUT;
//! const LAYOUT1B: &'static TypeLayout =
//!     <WithConstant<V1_1_Incompatible> as StableAbi>::LAYOUT;
//! const LAYOUT2: &'static TypeLayout = <WithConstant<V1_2> as StableAbi>::LAYOUT;
//!
//! fn main() {
//!     // Compared LAYOUT0 to LAYOUT1B,
//!     // then stored LAYOUT0.extra_checks as the ExtraChecks associated with both layouts.
//!     check_layout_compatibility(LAYOUT0, LAYOUT1B).unwrap();
//!
//!     // Compared LAYOUT1 to LAYOUT2,
//!     // then stored LAYOUT2.extra_checks as the ExtraChecks associated with both layouts.
//!     check_layout_compatibility(LAYOUT1, LAYOUT2).unwrap();
//!
//!     // Compared LAYOUT0 to LAYOUT2:
//!     // - the comparison succeeded,
//!     // - then both are combined.
//!     // - The combined trait object is attempted to be combined with the
//!     //      ExtraChecks in the global map associated to both LAYOUT0 and LAYOUT2,
//!     //      which are LAYOUT1B.extra_checks and LAYOUT2.extra_checks respectively.
//!     // - Combining the trait objects with the ones in the global map fails because
//!     //      the one from LAYOUT1B is incompatible with the one from LAYOUT2.
//!     check_layout_compatibility(LAYOUT0, LAYOUT2).unwrap_err();
//! }
//!
//! //////////////////////////////////////////////////////////////////////////////////
//!
//! #[repr(C)]
//! #[derive(StableAbi)]
//! #[sabi(
//!     // Replaces the C:StableAbi constraint with `C:GetStaticEquivalent`
//!     // (a supertrait of StableAbi).
//!     not_stableabi(C),
//!     bound(C: GetConstant),
//!     extra_checks = Self::CHECKER
//! )]
//! struct WithConstant<C> {
//!     // UnsafeIgnoredType is equivalent to PhantomData,
//!     // except that all `UnsafeIgnoredType` are considered the same type by `StableAbi`.
//!     _marker: UnsafeIgnoredType<C>,
//! }
//!
//! impl<C> WithConstant<C> {
//!     const NEW: Self = Self {
//!         _marker: UnsafeIgnoredType::NEW,
//!     };
//! }
//!
//! impl<C> WithConstant<C>
//! where
//!     C: GetConstant,
//! {
//!     const CHECKER: ConstChecker = ConstChecker {
//!         chars: RStr::from_str(C::CHARS),
//!     };
//! }
//!
//! trait GetConstant {
//!     const CHARS: &'static str;
//! }
//!
//! use self::constants::*;
//!
//! #[allow(non_camel_case_types)]
//! mod constants {
//!     use super::*;
//!
//!     #[derive(GetStaticEquivalent)]
//!     pub struct V1_0;
//!
//!     impl GetConstant for V1_0 {
//!         const CHARS: &'static str = "ab";
//!     }
//!
//!     #[derive(GetStaticEquivalent)]
//!     pub struct V1_1;
//!
//!     impl GetConstant for V1_1 {
//!         const CHARS: &'static str = "abc";
//!     }
//!
//!     #[derive(GetStaticEquivalent)]
//!     pub struct V1_1_Incompatible;
//!
//!     impl GetConstant for V1_1_Incompatible {
//!         const CHARS: &'static str = "abd";
//!     }
//!
//!     #[derive(GetStaticEquivalent)]
//!     pub struct V1_2;
//!
//!     impl GetConstant for V1_2 {
//!         const CHARS: &'static str = "abcd";
//!     }
//! }
//!
//! /////////////////////////////////////////
//!
//! #[repr(C)]
//! #[derive(Debug, Clone, StableAbi)]
//! pub struct ConstChecker {
//!     chars: RStr<'static>,
//! }
//!
//! impl Display for ConstChecker {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         writeln!(
//!             f,
//!             "ConstChecker: \
//!         Checks that the associated constant for \
//!         the other type is compatible with:\n{}\n.\
//!     ",
//!             self.chars
//!         )
//!     }
//! }
//!
//! impl ConstChecker {
//!     fn check_compatible_inner(
//!         &self,
//!         other: &ConstChecker,
//!     ) -> Result<(), UnequalConstError> {
//!         if other.chars.starts_with(&*self.chars) {
//!             Ok(())
//!         } else {
//!             Err(UnequalConstError {
//!                 expected: self.chars,
//!                 found: other.chars,
//!             })
//!         }
//!     }
//! }
//! unsafe impl ExtraChecks for ConstChecker {
//!     fn type_layout(&self) -> &'static TypeLayout {
//!         <Self as StableAbi>::LAYOUT
//!     }
//!
//!     fn check_compatibility(
//!         &self,
//!         _layout_containing_self: &'static TypeLayout,
//!         layout_containing_other: &'static TypeLayout,
//!         checker: TypeCheckerMut<'_>,
//!     ) -> RResult<(), ExtraChecksError> {
//!         Self::downcast_with_layout(layout_containing_other, checker, |other, _| {
//!             self.check_compatible_inner(other)
//!         })
//!     }
//!
//!     fn nested_type_layouts(&self) -> RCowSlice<'_, &'static TypeLayout> {
//!         RCow::from_slice(&[])
//!     }
//!
//!     fn combine(
//!         &self,
//!         other: ExtraChecksRef<'_>,
//!         checker: TypeCheckerMut<'_>,
//!     ) -> RResult<ROption<ExtraChecksBox>, ExtraChecksError> {
//!         Self::downcast_with_object(other, checker, |other, _| {
//!             let (min, max) = min_max_by(self, other, |x| x.chars.len());
//!             min.check_compatible_inner(max)
//!                 .map(|_| RSome(ExtraChecksBox::from_value(max.clone(), TD_Opaque)))
//!         })
//!     }
//! }
//!
//! #[derive(Debug, Clone)]
//! pub struct UnequalConstError {
//!     expected: RStr<'static>,
//!     found: RStr<'static>,
//! }
//!
//! impl Display for UnequalConstError {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         writeln!(
//!     f,
//!     "Expected the `GetConstant::CHARS` associated constant to be compatible with:\
//!      \n    {}\
//!      \nFound:\
//!      \n    {}\
//!     ",
//!     self.expected,
//!     self.found,
//! )
//!     }
//! }
//!
//! impl std::error::Error for UnequalConstError {}
//!
//! pub(crate) fn min_max_by<T, F, K>(l: T, r: T, mut f: F) -> (T, T)
//! where
//!     F: FnMut(&T) -> K,
//!     K: Ord,
//! {
//!     if f(&l) < f(&r) {
//!         (l, r)
//!     } else {
//!         (r, l)
//!     }
//! }
//!
//!
//! ```
//!
//!
//! # Examples
//!
//! ### Alphabetic.
//!
//! This defines an ExtraChecks which checks that fields are alphabetically sorted
//!
//! ```
//! use abi_stable::{
//!     abi_stability::{
//!         check_layout_compatibility,
//!         extra_checks::{
//!             ExtraChecks, ExtraChecksError, ExtraChecksStaticRef,
//!             ForExtraChecksImplementor, TypeCheckerMut,
//!         },
//!     },
//!     sabi_extern_fn,
//!     sabi_trait::prelude::TD_Opaque,
//!     std_types::{RCow, RCowSlice, RDuration, ROption, RResult, RStr, RString},
//!     type_layout::TypeLayout,
//!     StableAbi,
//! };
//!
//! use std::fmt::{self, Display};
//!
//! fn main() {
//!     let rect_layout = <Rectangle as StableAbi>::LAYOUT;
//!     let person_layout = <Person as StableAbi>::LAYOUT;
//!
//!     // This passes because the fields are in order
//!     check_layout_compatibility(rect_layout, rect_layout)
//!         .unwrap_or_else(|e| panic!("{}", e));
//!
//!     // This errors because the struct's fields aren't in order
//!     check_layout_compatibility(person_layout, person_layout).unwrap_err();
//! }
//!
//! #[repr(C)]
//! #[derive(StableAbi)]
//! #[sabi(extra_checks = InOrderChecker)]
//! struct Rectangle {
//!     x: u32,
//!     y: u32,
//!     z: u32,
//! }
//!
//! #[repr(C)]
//! #[derive(StableAbi)]
//! #[sabi(extra_checks = InOrderChecker)]
//! struct Person {
//!     name: RString,
//!     surname: RString,
//!     age: RDuration,
//! }
//!
//! /////////////////////////////////////////
//!
//! #[repr(C)]
//! #[derive(Debug, Clone, StableAbi)]
//! pub struct InOrderChecker;
//!
//! impl Display for InOrderChecker {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         f.write_str(
//!             "InOrderChecker: Checks that field names are sorted alphabetically.",
//!         )
//!     }
//! }
//!
//! unsafe impl ExtraChecks for InOrderChecker {
//!     fn type_layout(&self) -> &'static TypeLayout {
//!         <Self as StableAbi>::LAYOUT
//!     }
//!
//!     fn check_compatibility(
//!         &self,
//!         layout_containing_self: &'static TypeLayout,
//!         layout_containing_other: &'static TypeLayout,
//!         checker: TypeCheckerMut<'_>,
//!     ) -> RResult<(), ExtraChecksError> {
//!         Self::downcast_with_layout(layout_containing_other, checker, |_, _| {
//!             let fields = match layout_containing_self.get_fields() {
//!                 Some(fields) if !fields.is_empty() => fields,
//!                 _ => return Ok(()),
//!             };
//!
//!             let mut prev = fields.iter().next().unwrap();
//!             for curr in fields {
//!                 if prev.name() > curr.name() {
//!                     return Err(OutOfOrderError {
//!                         previous_one: prev.name(),
//!                         first_one: curr.name(),
//!                     });
//!                 }
//!                 prev = curr;
//!             }
//!             Ok(())
//!         })
//!     }
//!
//!     fn nested_type_layouts(&self) -> RCowSlice<'_, &'static TypeLayout> {
//!         RCow::from_slice(&[])
//!     }
//! }
//!
//! #[derive(Debug, Clone)]
//! pub struct OutOfOrderError {
//!     previous_one: &'static str,
//!
//!     /// The first field that is out of order.
//!     first_one: &'static str,
//! }
//!
//! impl Display for OutOfOrderError {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         writeln!(
//!             f,
//!             "Expected fields to be alphabetically sorted.\n\
//!          Found field '{}' before '{}'\
//!         ",
//!             self.previous_one, self.first_one,
//!         )
//!     }
//! }
//!
//! impl std::error::Error for OutOfOrderError {}
//!
//!
//! ```
//!
//! ### Associated Constant.
//!
//! This defines an ExtraChecks which checks that an associated constant is
//! the same for both types.
//!
//! ```
//! use abi_stable::{
//!     abi_stability::{
//!         check_layout_compatibility,
//!         extra_checks::{
//!             ExtraChecks, ExtraChecksError, ExtraChecksStaticRef,
//!             ForExtraChecksImplementor, TypeCheckerMut,
//!         },
//!     },
//!     marker_type::UnsafeIgnoredType,
//!     sabi_extern_fn,
//!     sabi_trait::prelude::TD_Opaque,
//!     std_types::{RCow, RCowSlice, RDuration, RResult, RStr, RString},
//!     type_layout::TypeLayout,
//!     GetStaticEquivalent, StableAbi,
//! };
//!
//! use std::fmt::{self, Display};
//!
//! fn main() {
//!     let const0 = <WithConstant<N0> as StableAbi>::LAYOUT;
//!     let const_second_0 = <WithConstant<SecondN0> as StableAbi>::LAYOUT;
//!     let const1 = <WithConstant<N1> as StableAbi>::LAYOUT;
//!     let const2 = <WithConstant<N2> as StableAbi>::LAYOUT;
//!
//!     check_layout_compatibility(const0, const0).unwrap();
//!     check_layout_compatibility(const_second_0, const_second_0).unwrap();
//!     check_layout_compatibility(const1, const1).unwrap();
//!     check_layout_compatibility(const2, const2).unwrap();
//!
//!     ////////////
//!     // WithConstant<SecondN0> and WithConstant<N0> are compatible with each other
//!     // because their `GetConstant::NUMBER` associated constant is the same value.
//!     check_layout_compatibility(const0, const_second_0).unwrap();
//!     check_layout_compatibility(const_second_0, const0).unwrap();
//!
//!     ////////////
//!     // None of the lines below are compatible because their
//!     // `GetConstant::NUMBER` associated constant isn't the same value.
//!     check_layout_compatibility(const0, const1).unwrap_err();
//!     check_layout_compatibility(const0, const2).unwrap_err();
//!
//!     check_layout_compatibility(const1, const0).unwrap_err();
//!     check_layout_compatibility(const1, const2).unwrap_err();
//!
//!     check_layout_compatibility(const2, const0).unwrap_err();
//!     check_layout_compatibility(const2, const1).unwrap_err();
//! }
//!
//! #[repr(C)]
//! #[derive(StableAbi)]
//! #[sabi(
//!     // Replaces the C:StableAbi constraint with `C:GetStaticEquivalent`
//!     // (a supertrait of StableAbi).
//!     not_stableabi(C),
//!     bound(C:GetConstant),
//!     extra_checks = Self::CHECKER,
//! )]
//! struct WithConstant<C> {
//!     // UnsafeIgnoredType is equivalent to PhantomData,
//!     // except that all `UnsafeIgnoredType` are considered the same type by `StableAbi`.
//!     _marker: UnsafeIgnoredType<C>,
//! }
//!
//! impl<C> WithConstant<C> {
//!     const NEW: Self = Self {
//!         _marker: UnsafeIgnoredType::NEW,
//!     };
//! }
//!
//! impl<C> WithConstant<C>
//! where
//!     C: GetConstant,
//! {
//!     const CHECKER: ConstChecker = ConstChecker { number: C::NUMBER };
//! }
//!
//! trait GetConstant {
//!     const NUMBER: u64;
//! }
//!
//! #[derive(GetStaticEquivalent)]
//! struct N0;
//! impl GetConstant for N0 {
//!     const NUMBER: u64 = 0;
//! }
//!
//! #[derive(GetStaticEquivalent)]
//! struct SecondN0;
//! impl GetConstant for SecondN0 {
//!     const NUMBER: u64 = 0;
//! }
//!
//! #[derive(GetStaticEquivalent)]
//! struct N1;
//! impl GetConstant for N1 {
//!     const NUMBER: u64 = 1;
//! }
//!
//! #[derive(GetStaticEquivalent)]
//! struct N2;
//! impl GetConstant for N2 {
//!     const NUMBER: u64 = 2;
//! }
//!
//! /////////////////////////////////////////
//!
//! #[repr(C)]
//! #[derive(Debug, Clone, StableAbi)]
//! pub struct ConstChecker {
//!     number: u64,
//! }
//!
//! impl Display for ConstChecker {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         writeln!(
//!             f,
//!             "ConstChecker: \
//!             Checks that the associated constant for \
//!             for the other type is {}.\
//!         ",
//!             self.number
//!         )
//!     }
//! }
//!
//! unsafe impl ExtraChecks for ConstChecker {
//!     fn type_layout(&self) -> &'static TypeLayout {
//!         <Self as StableAbi>::LAYOUT
//!     }
//!
//!     fn check_compatibility(
//!         &self,
//!         layout_containing_self: &'static TypeLayout,
//!         layout_containing_other: &'static TypeLayout,
//!         checker: TypeCheckerMut<'_>,
//!     ) -> RResult<(), ExtraChecksError> {
//!         Self::downcast_with_layout(layout_containing_other, checker, |other, _| {
//!             if self.number == other.number {
//!                 Ok(())
//!             } else {
//!                 Err(UnequalConstError {
//!                     expected: self.number,
//!                     found: other.number,
//!                 })
//!             }
//!         })
//!     }
//!
//!     fn nested_type_layouts(&self) -> RCowSlice<'_, &'static TypeLayout> {
//!         RCow::from_slice(&[])
//!     }
//! }
//!
//! #[derive(Debug, Clone)]
//! pub struct UnequalConstError {
//!     expected: u64,
//!     found: u64,
//! }
//!
//! impl Display for UnequalConstError {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         writeln!(
//!             f,
//!             "Expected the `GetConstant::NUMBER` associated constant to be:\
//!          \n    {}\
//!          \nFound:\
//!          \n    {}\
//!         ",
//!             self.expected, self.found,
//!         )
//!     }
//! }
//!
//! impl std::error::Error for UnequalConstError {}
//!
//!
//! ```
//!

use crate::{
    rtry, sabi_trait,
    sabi_types::{RMut, RRef},
    std_types::{RBox, RBoxError, RCowSlice, RNone, ROk, ROption, RResult},
    traits::IntoReprC,
    type_layout::TypeLayout,
    StableAbi,
};

use std::{
    error::Error as ErrorTrait,
    fmt::{self, Display},
};

use core_extensions::SelfOps;

#[sabi_trait]
/// This checks that the layout of types coming from dynamic libraries
/// are compatible with those of the binary/dynlib that loads them.
///
/// # Safety
///
/// This trait must not be implemented outside of `abi_stable`.
///
#[sabi(no_trait_impl)]
// #[sabi(debug_print_trait)]
// #[sabi(debug_print)]
pub unsafe trait TypeChecker: 'static + Send + Sync {
    /// Checks that `ìnterface` is compatible with `implementation.`
    ///
    /// This is equivalent to `check_layout_compatibility`,
    /// except that it can also be called re-entrantly
    /// (while `check_layout_compatibility` cannot be called re-entrantly)
    ///
    /// # Errors
    ///
    /// When calling the implementation of this trait used in `check_layout_compatibility`,
    /// the errors detected in this method are always propagated by the free function,
    /// to prevent the propagation of errors call the `local_check_compatibility` method.
    ///
    fn check_compatibility(
        &mut self,
        interface: &'static TypeLayout,
        implementation: &'static TypeLayout,
    ) -> RResult<(), ExtraChecksError>;

    /// Checks that `ìnterface` is compatible with `implementation.`
    ///
    /// This is equivalent to the `check_compatibility` method,
    /// except that it does not propagate errors automatically,
    /// they must be returned as part of the error of the `ExtraChecks` that calls this.
    #[sabi(last_prefix_field)]
    fn local_check_compatibility(
        &mut self,
        interface: &'static TypeLayout,
        implementation: &'static TypeLayout,
    ) -> RResult<(), ExtraChecksError>;
}

/// An ffi-safe equivalent of &'b mut dyn TypeChecker
pub type TypeCheckerMut<'b> = TypeChecker_TO<RMut<'b, ()>>;

#[sabi_trait]
/// Allows defining extra checks for a type.
///
/// Look at the
/// [module level documentation](./index.html)
/// for more details.
///
/// # Safety
///
/// The `type_layout` method must be defined as `<Self as ::abi_stable::StableAbi>::LAYOUT`,
/// or equivalent.
///
/// All of the methods must be deterministic,
/// always returning the same value with the same arguments.
///
#[sabi(no_trait_impl)]
// #[sabi(debug_print_trait)]
// #[sabi(debug_print)]
pub unsafe trait ExtraChecks: 'static + Debug + Display + Clone + Send + Sync {
    /// Gets the type layout of `Self`(the type that implements ExtraChecks)
    ///
    /// This is used to downcast the trait object in
    /// `ForExtraChecksImplementor::downcast_*` methods,
    /// by ensuring that its type layout is
    /// compatible with that of another ExtraChecks trait object
    ///
    /// It can't use `UTypeId`s to check compatibility of trait objects
    /// from different dynamic libraries,
    /// because `UTypeId`s from different dynamic libraries are incompatible.
    fn type_layout(&self) -> &'static TypeLayout;

    /// Checks that `self` is compatible another type which implements ExtraChecks.
    ///
    /// Calling `check_layout_compatibility` from here will immediately return an error,
    /// prefer doing `checker.check_compatibility(...)` instead.
    ///
    /// # Parameters
    ///
    /// `layout_containing_self`:
    /// The TypeLayout that contains this `ExtraChecks` trait object in the extra_checks field.
    ///
    /// `layout_containing_other`:
    /// The TypeLayout that contains the `other` `ExtraChecks` trait object in the extra_checks field,
    /// which `self` is compared to.
    ///
    /// `checker`:
    /// The type checker,which allows this function to check type layouts.
    ///
    ///
    fn check_compatibility(
        &self,
        layout_containing_self: &'static TypeLayout,
        layout_containing_other: &'static TypeLayout,
        checker: TypeCheckerMut<'_>,
    ) -> RResult<(), ExtraChecksError>;

    /// Returns the `TypeLayout`s owned or referenced by `self`.
    ///
    /// This is necessary for the Debug implementation of `TypeLayout`.
    fn nested_type_layouts(&self) -> RCowSlice<'_, &'static TypeLayout>;

    /// Combines this ExtraChecks trait object with another one.
    ///
    /// To guarantee that the `extra_checks`
    /// associated with a type (inside `<TheType as StableAbi>::LAYOUT.extra_cheks` )
    /// has a single representative value across all dynamic libraries,
    /// you must override this method,
    /// and return `ROk(RSome(_))` by combining `self` and `other` in some way.
    ///
    ///
    /// # Parameters
    ///
    /// `other`:
    /// The other ExtraChecks trait object that this is combined with..
    ///
    /// `checker`:
    /// The trait object of the type checker,which allows this function to check type layouts.
    ///
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
    #[sabi(last_prefix_field)]
    fn combine(
        &self,
        _other: ExtraChecksRef<'_>,
        _checker: TypeCheckerMut<'_>,
    ) -> RResult<ROption<ExtraChecksBox>, ExtraChecksError> {
        ROk(RNone)
    }
}

/// The version of `ExtraChecks` that is stored in `TypeLayout`.
pub type StoredExtraChecks = ExtraChecks_CTO<'static>;

/// An ffi-safe equivalent of `&'static dyn ExtraChecks`.
pub type ExtraChecksStaticRef = ExtraChecks_TO<RRef<'static, ()>>;

/// An ffi-safe equivalent of `&'a dyn ExtraChecks`.
pub type ExtraChecksRef<'a> = ExtraChecks_TO<RRef<'a, ()>>;

/// An ffi-safe equivalent of `Box<dyn ExtraChecks>`.
pub type ExtraChecksBox = ExtraChecks_TO<RBox<()>>;

/// An extension trait for `ExtraChecks` implementors.
pub trait ForExtraChecksImplementor: StableAbi + ExtraChecks {
    /// Accesses the `ExtraChecks` field in `layout_containing_other`, downcasted into `Self`.
    ///
    /// If the closure returns an `ExtraChecksError`,it'll be returned unmodified and unwrapped.
    ///
    /// # Returns
    ///
    /// - ROk(_):
    ///     If `other` was downcasted to `Self`,and `f` did not return any errors.
    ///
    /// - RErr(ExtraChecksError::NoneExtraChecks):
    ///     If`layout_containing_other` does not contain an ExtraChecks trait object.
    ///
    /// - RErr(ExtraChecksError::TypeChecker):
    ///     If there is an error while type checking.
    ///
    /// - RErr(ExtraChecksError::ExtraChecks(_)):
    ///     If there is an custom error within the function.
    ///
    fn downcast_with_layout<F, R, E>(
        layout_containing_other: &'static TypeLayout,
        checker: TypeCheckerMut<'_>,
        f: F,
    ) -> RResult<R, ExtraChecksError>
    where
        Self: 'static,
        F: FnOnce(&Self, TypeCheckerMut<'_>) -> Result<R, E>,
        E: Send + Sync + ErrorTrait + 'static,
    {
        let other = rtry!(layout_containing_other
            .extra_checks()
            .ok_or(ExtraChecksError::NoneExtraChecks));

        Self::downcast_with_object(other, checker, f)
    }

    /// Allows one to access `other` downcast into `Self`.
    ///
    /// If the closure returns an `ExtraChecksError`,it'll be returned unmodified and unwrapped.
    ///
    /// # Returns
    ///
    /// - ROk(_):
    ///     If `other` could be downcasted to `Self`,and `f` did not return any errors.
    ///
    /// - RErr(ExtraChecksError::TypeChecker):
    ///     If there is an error while type checking.
    ///
    /// - RErr(ExtraChecksError::ExtraChecks(_)):
    ///     If there is an custom error within the function.
    fn downcast_with_object<F, R, E>(
        other: ExtraChecksRef<'_>,
        mut checker: TypeCheckerMut<'_>,
        f: F,
    ) -> RResult<R, ExtraChecksError>
    where
        F: FnOnce(&Self, TypeCheckerMut<'_>) -> Result<R, E>,
        E: Send + Sync + ErrorTrait + 'static,
    {
        // This checks that the layouts of `this` and `other` are compatible,
        // so that calling the `unchecked_downcast_into` method is sound.
        rtry!(checker.check_compatibility(<Self as StableAbi>::LAYOUT, other.type_layout()));
        let other_ue = unsafe { other.obj.unchecked_downcast_into::<Self>() };

        f(other_ue.get(), checker)
            .map_err(|e| match RBoxError::new(e).downcast::<ExtraChecksError>() {
                Ok(x) => x.piped(RBox::into_inner),
                Err(e) => ExtraChecksError::from_extra_checks(e),
            })
            .into_c()
    }
}

impl<This> ForExtraChecksImplementor for This where This: ?Sized + StableAbi + ExtraChecks {}

///////////////////////////////////////////////////////////////////////////////

/// The errors returned from `ExtraChecks` and `ForExtraChecksImplementor` methods.
#[repr(u8)]
#[derive(Debug, StableAbi)]
pub enum ExtraChecksError {
    /// When a type checking error happens within `TypeChecker`.
    TypeChecker,
    /// Errors returned from `TypeChecker::local_check_compatibility`
    TypeCheckerErrors(RBoxError),
    /// When trying to get a ExtraChecks trait object from `TypeLayout.extra_checks==None` .
    NoneExtraChecks,
    /// A custom error returned by the ExtraChecker or
    /// the closures in `ForExtraChecksImplementor::downcast_*`.
    ExtraChecks(RBoxError),
}

impl ExtraChecksError {
    /// Constructs a `ExtraChecksError::ExtraChecks` from an error.
    pub fn from_extra_checks<E>(err: E) -> ExtraChecksError
    where
        E: Send + Sync + ErrorTrait + 'static,
    {
        let x = RBoxError::new(err);
        ExtraChecksError::ExtraChecks(x)
    }
}

impl Display for ExtraChecksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtraChecksError::TypeChecker => Display::fmt("A type checker error happened.", f),
            ExtraChecksError::TypeCheckerErrors(e) => {
                writeln!(f, "Errors that happened during type checking:\n{}", e)
            }
            ExtraChecksError::NoneExtraChecks => {
                Display::fmt("No `ExtraChecks` in the implementation.", f)
            }
            ExtraChecksError::ExtraChecks(e) => Display::fmt(e, f),
        }
    }
}

impl std::error::Error for ExtraChecksError {}
