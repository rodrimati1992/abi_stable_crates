/*!
Functions and types related to the layout checking.
*/

use std::{cmp::Ordering, fmt};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use std::collections::HashSet;
// use std::collections::HashSet;

use super::{
    AbiInfo, AbiInfoWrapper, StableAbi, TLData, TLDataDiscriminant, TLEnumVariant, TLField,
    TLFieldAndType, FullType,
};
use crate::{
    version::{ParseVersionError, VersionStrings},
    std_types::{RVec, StaticSlice, StaticStr},
};

/// All the errors from checking the layout of every nested type in AbiInfo.
#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct AbiInstabilityErrors {
    pub interface: &'static AbiInfo,
    pub implementation: &'static AbiInfo,
    pub errors: RVec<AbiInstabilityError>,
    _priv:(),
}

/// All the shallow errors from checking an individual type.
///
/// Error that happen lower or higher on the stack are stored in separate
///  `AbiInstabilityError`s.
#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct AbiInstabilityError {
    pub stack_trace: RVec<TLFieldAndType>,
    pub errs: RVec<AbiInstability>,
    pub index: usize,
    _priv:(),
}

/// An individual error from checking the layout of some type.
#[derive(Debug, PartialEq)]
#[repr(C)]
pub enum AbiInstability {
    IsPrefix(ExpectedFoundError<bool>),
    NonZeroness(ExpectedFoundError<bool>),
    Name(ExpectedFoundError<FullType>),
    Package(ExpectedFoundError<StaticStr>),
    PackageVersionParseError(ParseVersionError),
    PackageVersion(ExpectedFoundError<VersionStrings>),
    Size(ExpectedFoundError<usize>),
    Alignment(ExpectedFoundError<usize>),
    GenericParamCount(ExpectedFoundError<FullType>),
    TLDataDiscriminant(ExpectedFoundError<TLDataDiscriminant>),
    FieldCountMismatch(ExpectedFoundError<usize>),
    FieldLifetimeMismatch(ExpectedFoundError<&'static TLField>),
    UnexpectedField(ExpectedFoundError<&'static TLField>),
    TooManyVariants(ExpectedFoundError<usize>),
    UnexpectedVariant(ExpectedFoundError<TLEnumVariant>),
}

use self::AbiInstability as AI;

impl AbiInstabilityErrors {
    pub fn flatten_errors(self) -> RVec<AbiInstability> {
        self.errors
            .into_iter()
            .flat_map(|x| x.errs)
            .collect::<RVec<AbiInstability>>()
    }
}

impl fmt::Display for AbiInstabilityErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Compared <this>:\n{}\nTo <other>:\n{}\n",
            self.interface.layout.full_type().to_string().left_padder(4),
            self.implementation
                .layout
                .full_type()
                .to_string()
                .left_padder(4),
        )?;
        for err in &self.errors {
            fmt::Display::fmt(err, f)?;
        }
        Ok(())
    }
}
impl fmt::Display for AbiInstabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} error(s) at:", self.errs.len())?;
        write!(f, "<this>")?;
        for field in &self.stack_trace {
            write!(f, ".{}", field.name())?;
        }
        if let Some(last) = self.stack_trace.last() {
            write!(f, ":{}", last.full_type())?;
        }
        writeln!(f)?;
        for err in &self.errs {
            let (error_msg, expected_err): (&'static str, ExpectedFoundError<String>) = match err {
                AI::IsPrefix(v) => ("mismatched prefixness", v.debug_str()),
                AI::NonZeroness(v) => ("mismatched non-zeroness", v.display_str()),
                AI::Name(v) => ("mismatched type", v.display_str()),
                AI::Package(v) => ("mismatched package", v.display_str()),
                AI::PackageVersionParseError(v) => {
                    let expected = "a valid version string".to_string();
                    let found = format!("{:#?}", v);

                    (
                        "could not parse version string",
                        ExpectedFoundError { expected, found },
                    )
                }
                AI::PackageVersion(v) => ("incompatible package versions", v.display_str()),
                AI::Size(v) => ("incompatible type size", v.display_str()),
                AI::Alignment(v) => ("incompatible type alignment", v.display_str()),
                AI::GenericParamCount(v) => (
                    "incompatible ammount of generic parameters",
                    v.display_str(),
                ),

                AI::TLDataDiscriminant(v) => ("incompatible data ", v.debug_str()),
                AI::FieldCountMismatch(v) => ("too many fields", v.display_str()),
                AI::FieldLifetimeMismatch(v) => {
                    ("field references different lifetimes", v.debug_str())
                }
                AI::UnexpectedField(v) => ("unexpected field", v.debug_str()),
                AI::TooManyVariants(v) => ("too many variants", v.display_str()),
                AI::UnexpectedVariant(v) => ("unexpected variant", v.debug_str()),
            };

            writeln!(
                f,
                "\nError:{}\nExpected:\n{}\nFound:\n{}",
                error_msg,
                expected_err.expected.left_padder(4),
                expected_err.found.left_padder(4),
            )?;
        }
        Ok(())
    }
}

//////

/// Represents an error where a value was expected,but another value was found.
#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct ExpectedFoundError<T> {
    expected: T,
    found: T,
}

impl<T> ExpectedFoundError<T> {
    pub fn new<O, F>(this: O, other: O, mut field_getter: F) -> ExpectedFoundError<T>
    where
        F: FnMut(O) -> T,
    {
        ExpectedFoundError {
            expected: field_getter(this),
            found: field_getter(other),
        }
    }

    pub fn as_ref(&self) -> ExpectedFoundError<&T> {
        ExpectedFoundError {
            expected: &self.expected,
            found: &self.found,
        }
    }

    pub fn map<F, U>(self, mut f: F) -> ExpectedFoundError<U>
    where
        F: FnMut(T) -> U,
    {
        ExpectedFoundError {
            expected: f(self.expected),
            found: f(self.found),
        }
    }

    pub fn display_str(&self) -> ExpectedFoundError<String>
    where
        T: fmt::Display,
    {
        self.as_ref().map(|x| format!("{:#}", x))
    }

    pub fn debug_str(&self) -> ExpectedFoundError<String>
    where
        T: fmt::Debug,
    {
        self.as_ref().map(|x| format!("{:#?}", x))
    }
}

///////////////////////////////////////////////

struct AbiChecker {
    stack_trace: RVec<TLFieldAndType>,

    visited: HashSet<(*const AbiInfo,*const AbiInfo)>,
    errors: RVec<AbiInstabilityError>,

    error_index: usize,
}

///////////////////////////////////////////////

impl AbiChecker {
    fn new() -> Self {
        Self {
            stack_trace: RVec::new(),

            visited: HashSet::default(),
            errors: RVec::new(),
            error_index: 0,
        }
    }

    #[inline]
    fn check_fields(
        &mut self,
        errs: &mut RVec<AbiInstability>,
        this: &'static AbiInfo,
        _: &'static AbiInfo,
        t_fields: StaticSlice<TLField>,
        o_fields: StaticSlice<TLField>,
    ) {
        let t_fields = t_fields.as_slice();
        let o_fields = o_fields.as_slice();
        match (t_fields.len().cmp(&o_fields.len()), this.prefix_kind) {
            (Ordering::Greater, _) | (Ordering::Less, false) => {
                push_err(
                    errs,
                    t_fields,
                    o_fields,
                    |x| x.len(),
                    AI::FieldCountMismatch,
                );
            }
            (Ordering::Equal, _) | (Ordering::Less, true) => {}
        }

        let mut t_fields_iter=t_fields.iter().peekable();
        let mut o_fields_iter=o_fields.iter().peekable();
        while let (Some(&this_f),Some(&other_f))=(t_fields_iter.peek(),o_fields_iter.peek()) {
            if this_f.name != other_f.name {
                push_err(errs, this_f, other_f, |x| x, AI::UnexpectedField);
                // Skipping this field so that the error message does not 
                // list all the other fields that they have in common.
                if t_fields.len() < o_fields.len() {
                    o_fields_iter.next();
                }else{
                    t_fields_iter.next();
                }
                continue;
            }
            if this_f.lifetime_indices != other_f.lifetime_indices {
                push_err(errs, this_f, other_f, |x| x, AI::FieldLifetimeMismatch);
            }

            self.stack_trace.push(TLFieldAndType::new(this_f));
            self.check_inner(this_f.abi_info.get(), other_f.abi_info.get());
            self.stack_trace.pop();

            t_fields_iter.next();
            o_fields_iter.next();
        }
    }

    fn check_inner(&mut self, this: &'static AbiInfo, other: &'static AbiInfo) {
        if !self.visited.insert((this as *const _,other as *const _)) {
            return;
        }

        self.error_index += 1;
        let errs_index = self.error_index;
        let mut errs_ = RVec::<AbiInstability>::new();
        let t_lay = &this.layout;
        let o_lay = &other.layout;

        (|| {
            let errs = &mut errs_;
            if t_lay.name != o_lay.name {
                push_err(errs, t_lay, o_lay, |x| x.full_type(), AI::Name);
                return;
            }
            if t_lay.package != o_lay.package {
                push_err(errs, t_lay, o_lay, |x| x.package, AI::Package);
                return;
            }

            if this.prefix_kind != other.prefix_kind {
                push_err(errs, this, other, |x| x.prefix_kind, AI::IsPrefix);
            }
            if this.is_nonzero != other.is_nonzero {
                push_err(errs, this, other, |x| x.is_nonzero, AI::NonZeroness);
            }

            {
                let x = (|| {
                    let l = t_lay.package_version.parsed()?;
                    let r = o_lay.package_version.parsed()?;
                    Ok(l.is_compatible(r))
                })();
                match x {
                    Ok(false) => {
                        push_err(
                            errs,
                            t_lay,
                            o_lay,
                            |x| x.package_version,
                            AI::PackageVersion,
                        );
                    }
                    Ok(true) => {}
                    Err(parse_error) => {
                        errs.push(AI::PackageVersionParseError(parse_error));
                        return;
                    }
                }
            }
            {
                let t_gens = &t_lay.full_type.generics;
                let o_gens = &o_lay.full_type.generics;
                if t_gens.lifetime.len() != o_gens.lifetime.len()
                    || t_gens.type_.len() != o_gens.type_.len()
                    || t_gens.const_.len() != o_gens.const_.len()
                {
                    push_err(errs, t_lay, o_lay, |x| x.full_type, AI::GenericParamCount);
                }
            }
            self.check_fields(
                errs,
                this,
                other,
                this.layout.phantom_fields,
                other.layout.phantom_fields,
            );

            match (t_lay.size.cmp(&o_lay.size), this.prefix_kind) {
                (Ordering::Greater, _) | (Ordering::Less, false) => {
                    push_err(errs, t_lay, o_lay, |x| x.size, AI::Size);
                }
                (Ordering::Equal, _) | (Ordering::Less, true) => {}
            }
            if t_lay.alignment != o_lay.alignment {
                push_err(errs, t_lay, o_lay, |x| x.alignment, AI::Alignment);
            }

            let t_discr = t_lay.data.discriminant();
            let o_discr = o_lay.data.discriminant();
            if t_discr != o_discr {
                errs.push(AI::TLDataDiscriminant(ExpectedFoundError {
                    expected: t_discr,
                    found: o_discr,
                }));
            }

            match (t_lay.data, o_lay.data) {
                (TLData::Primitive, TLData::Primitive) => {}
                (TLData::Primitive, _) => {}
                (TLData::Struct { fields: t_fields }, TLData::Struct { fields: o_fields }) => {
                    self.check_fields(errs, this, other, t_fields, o_fields);
                }
                (TLData::Struct { .. }, _) => {}
                (TLData::Enum { variants: t_varis }, TLData::Enum { variants: o_varis }) => {
                    let t_varis = t_varis.as_slice();
                    let o_varis = o_varis.as_slice();
                    if t_varis.len() != o_varis.len() {
                        push_err(errs, t_varis, o_varis, |x| x.len(), AI::TooManyVariants);
                    }
                    for (t_vari, o_vari) in t_varis.iter().zip(o_varis) {
                        let t_name = t_vari.name.as_str();
                        let o_name = o_vari.name.as_str();
                        if t_name != o_name {
                            push_err(errs, *t_vari, *o_vari, |x| x, AI::UnexpectedVariant);
                            continue;
                        }
                        self.check_fields(errs, this, other, t_vari.fields, o_vari.fields);
                    }
                }
                (TLData::Enum { .. }, _) => {}
                (TLData::ReprTransparent(t_nested), TLData::ReprTransparent(o_nested)) => {
                    self.check_inner(t_nested, o_nested);
                }
                (TLData::ReprTransparent { .. }, _) => {}
            }
        })();

        if !errs_.is_empty() {
            self.errors.push(AbiInstabilityError {
                stack_trace: self.stack_trace.clone(),
                errs: errs_,
                index: errs_index,
                _priv:(),
            });
        }
    }
}

/**
Checks that the layout of `Interface` is compatible with `Impl`.

# Warning

This function is not symmetric,
the first parameter must be the expected layout,
and the second must be actual layout.

*/
pub fn check_abi_stability_for<Interface, Impl>() -> Result<(), AbiInstabilityErrors>
where
    Interface: StableAbi,
    Impl: StableAbi,
{
    check_abi_stability(Interface::ABI_INFO, Impl::ABI_INFO)
}

/**
Checks that the layout of `interface` is compatible with `implementation`.

# Warning

This function is not symmetric,
the first parameter must be the expected layout,
and the second must be actual layout.

*/
// Never inline this function because it will be called very infrequently and
// will take a long-ish time to run anyway.
#[inline(never)]
pub fn check_abi_stability(
    interface: &'static AbiInfoWrapper,
    implementation: &'static AbiInfoWrapper,
) -> Result<(), AbiInstabilityErrors> {
    let mut errors: RVec<AbiInstabilityError>;

    let interface = interface.get();
    let implementation = implementation.get();

    if interface.prefix_kind || implementation.prefix_kind {
        errors = vec![AbiInstabilityError {
            stack_trace: vec![].into(),
            errs: vec![AbiInstability::IsPrefix(ExpectedFoundError {
                expected: false,
                found: true,
            })]
            .into(),
            index: 0,
            _priv:(),
        }]
        .into();
    } else {
        let mut checker = AbiChecker::new();
        checker.check_inner(interface, implementation);
        errors = checker.errors;
    }

    if errors.is_empty() {
        Ok(())
    } else {
        errors.sort_by_key(|x| x.index);
        Err(AbiInstabilityErrors {
            interface: interface,
            implementation: implementation,
            errors,
            _priv:()
        })
    }
}

///////////////////////////////////////////////

fn push_err<O, U, FG, VC>(
    errs: &mut RVec<AbiInstability>,
    this: O,
    other: O,
    field_getter: FG,
    mut variant_constructor: VC,
) where
    FG: FnMut(O) -> U,
    VC: FnMut(ExpectedFoundError<U>) -> AbiInstability,
{
    let x = ExpectedFoundError::new(this, other, field_getter);
    let x = variant_constructor(x);
    errs.push(x);
}
