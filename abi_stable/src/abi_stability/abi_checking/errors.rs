use super::*;

use core_extensions::StringExt;

/// An individual error from checking the layout of some type.
#[derive(Debug, PartialEq, Clone)]
pub enum AbiInstability {
    ReentrantLayoutCheckingCall,
    CyclicTypeChecking {
        interface: &'static TypeLayout,
        implementation: &'static TypeLayout,
    },
    NonZeroness(ExpectedFound<bool>),
    Name(ExpectedFound<FmtFullType>),
    Package(ExpectedFound<RStr<'static>>),
    PackageVersionParseError(ParseVersionError),
    PackageVersion(ExpectedFound<VersionStrings>),
    MismatchedPrefixSize(ExpectedFound<u8>),
    Size(ExpectedFound<usize>),
    Alignment(ExpectedFound<usize>),
    GenericParamCount(ExpectedFound<FmtFullType>),
    TLDataDiscriminant(ExpectedFound<TLDataDiscriminant>),
    MismatchedPrimitive(ExpectedFound<TLPrimitive>),
    FieldCountMismatch(ExpectedFound<usize>),
    FieldLifetimeMismatch(ExpectedFound<TLField>),
    FnLifetimeMismatch(ExpectedFound<TLFunction>),
    FnQualifierMismatch(ExpectedFound<TLFunction>),
    UnexpectedField(ExpectedFound<TLField>),
    TooManyVariants(ExpectedFound<usize>),
    MismatchedPrefixConditionality(ExpectedFound<FieldConditionality>),
    MismatchedExhaustiveness(ExpectedFound<IsExhaustive>),
    MismatchedConstParam(ExpectedFound<ConstGeneric>),
    UnexpectedVariant(ExpectedFound<RStr<'static>>),
    ReprAttr(ExpectedFound<ReprAttr>),
    EnumDiscriminant(ExpectedFound<TLDiscriminant>),
    IncompatibleWithNonExhaustive(IncompatibleWithNonExhaustive),
    NoneExtraChecks,
    ExtraCheckError(CmpIgnored<ExtraCheckError>),
    TagError {
        err: TagErrors,
    },
}

#[derive(Debug, Clone)]
pub struct ExtraCheckError {
    pub err: RArc<RBoxError>,
    pub expected_err: ExpectedFound<RArc<RBoxError>>,
}

use self::AbiInstability as AI;

#[allow(dead_code)]
impl AbiInstabilityErrors {
    #[cfg(feature = "testing")]
    pub fn flatten_errors(&self) -> RVec<AbiInstability> {
        self.flattened_errors().collect::<RVec<AbiInstability>>()
    }

    #[cfg(feature = "testing")]
    pub fn flattened_errors(&self) -> impl Iterator<Item = AbiInstability> + '_ {
        self.errors.iter().flat_map(|x| &x.errs).cloned()
    }
}

impl std::error::Error for AbiInstabilityErrors {}

impl fmt::Debug for AbiInstabilityErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl fmt::Display for AbiInstabilityErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Compared <this>:\n{}\nTo <other>:\n{}\n",
            self.interface.to_string().left_padder(4),
            self.implementation.to_string().left_padder(4),
        )?;
        for err in &self.errors {
            fmt::Display::fmt(err, f)?;
        }
        Ok(())
    }
}

impl fmt::Display for AbiInstabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut extra_err = None::<String>;

        write!(f, "{} error(s)", self.errs.len())?;
        if self.stack_trace.is_empty() {
            writeln!(f, ".")?;
        } else {
            writeln!(f, "inside:\n    <other>\n")?;
        }
        for field in &self.stack_trace {
            writeln!(f, "{}\n", field.found.to_string().left_padder(4))?;
        }
        if let Some(ExpectedFound { expected, found }) = self.stack_trace.last() {
            writeln!(
                f,
                "Layout of expected type:\n{}\n\n\
                 Layout of found type:\n{}\n",
                expected.formatted_layout().left_padder(4),
                found.formatted_layout().left_padder(4),
            )?;
        }
        writeln!(f)?;

        for err in &self.errs {
            let pair = match err {
                AI::ReentrantLayoutCheckingCall => ("reentrant layout checking call", None),
                AI::CyclicTypeChecking { interface, .. } => {
                    extra_err = Some(format!("The type:\n{}", interface));

                    (
                        "Attempted to check the layout of a type while checking the layout \
                         of one of it's const parameters/extra_checks\
                         (not necessarily a direct one).",
                        None,
                    )
                }
                AI::NonZeroness(v) => ("mismatched non-zeroness", v.display_str()),
                AI::Name(v) => ("mismatched type", v.display_str()),
                AI::Package(v) => ("mismatched package", v.display_str()),
                AI::PackageVersionParseError(v) => {
                    let expected = "a valid version string".to_string();
                    let found = format!("{:#?}", v);

                    (
                        "could not parse version string",
                        Some(ExpectedFound { expected, found }),
                    )
                }
                AI::PackageVersion(v) => ("incompatible package versions", v.display_str()),
                AI::MismatchedPrefixSize(v) => {
                    ("prefix-types have a different prefix", v.display_str())
                }
                AI::Size(v) => ("incompatible type size", v.display_str()),
                AI::Alignment(v) => ("incompatible type alignment", v.display_str()),
                AI::GenericParamCount(v) => {
                    ("incompatible amount of generic parameters", v.display_str())
                }

                AI::TLDataDiscriminant(v) => ("incompatible data ", v.debug_str()),
                AI::MismatchedPrimitive(v) => ("incompatible primitive", v.debug_str()),
                AI::FieldCountMismatch(v) => ("too many fields", v.display_str()),
                AI::FnLifetimeMismatch(v) => (
                    "function pointers reference different lifetimes",
                    v.display_str(),
                ),
                AI::FnQualifierMismatch(v) => (
                    "function pointers have different qualifiers (`unsafe`, etc.)",
                    v.display_str(),
                ),
                AI::FieldLifetimeMismatch(v) => {
                    ("field references different lifetimes", v.display_str())
                }
                AI::UnexpectedField(v) => ("unexpected field", v.display_str()),
                AI::TooManyVariants(v) => ("too many variants", v.display_str()),
                AI::MismatchedPrefixConditionality(v) => (
                    "prefix fields differ in whether they are conditional",
                    v.debug_str(),
                ),
                AI::MismatchedExhaustiveness(v) => {
                    ("enums differ in whether they are exhaustive", v.debug_str())
                }
                AI::MismatchedConstParam(v) => {
                    ("The cconst parameters are different", v.debug_str())
                }
                AI::UnexpectedVariant(v) => ("unexpected variant", v.debug_str()),
                AI::ReprAttr(v) => ("incompatible repr attributes", v.debug_str()),
                AI::EnumDiscriminant(v) => ("different discriminants", v.debug_str()),
                AI::IncompatibleWithNonExhaustive(e) => {
                    extra_err = Some(e.to_string());

                    ("", None)
                }
                AI::NoneExtraChecks => {
                    let msg = "\
                        Interface contains a value in `extra_checks` \
                        while the implementation does not.\
                    ";
                    (msg, None)
                }
                AI::ExtraCheckError(ec_error) => {
                    let ExtraCheckError { err, expected_err } = &**ec_error;
                    extra_err = Some((**err).to_string());

                    ("", expected_err.display_str())
                }
                AI::TagError { err } => {
                    extra_err = Some(err.to_string());

                    ("", None)
                }
            };

            let (error_msg, expected_err): (&'static str, Option<ExpectedFound<String>>) = pair;

            if let Some(expected_err) = expected_err {
                writeln!(
                    f,
                    "\nError:{}\nExpected:\n{}\nFound:\n{}",
                    error_msg,
                    expected_err.expected.left_padder(4),
                    expected_err.found.left_padder(4),
                )?;
            }
            if let Some(extra_err) = &extra_err {
                writeln!(f, "\nExtra:\n{}\n", extra_err.left_padder(4))?;
            }
        }
        Ok(())
    }
}

/// All the errors from checking the layout of every nested type in TypeLayout.
#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct AbiInstabilityErrors {
    pub interface: &'static TypeLayout,
    pub implementation: &'static TypeLayout,
    pub errors: RVec<AbiInstabilityError>,
    pub(super) _priv: (),
}

/// All the shallow errors from checking an individual type.
///
/// Error that happen lower or higher on the stack are stored in separate
///  `AbiInstabilityError`s.
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct AbiInstabilityError {
    pub stack_trace: RVec<ExpectedFound<TLFieldOrFunction>>,
    pub errs: RVec<AbiInstability>,
    pub index: usize,
    pub(super) _priv: (),
}
