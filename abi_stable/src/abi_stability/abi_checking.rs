/*!
Functions and types related to the layout checking.
*/

use std::{cmp::Ordering, fmt,mem};

#[allow(unused_imports)]
use core_extensions::{prelude::*,matches};

use std::{
    borrow::Borrow,
    cell::Cell,
    collections::HashSet,
};
// use std::collections::HashSet;

use crate::{
    abi_stability::{ExtraChecksRef,TypeChecker,TypeChecker_TO,ExtraChecksError},
    sabi_types::{ParseVersionError, VersionStrings},
    prefix_type::{FieldAccessibility,IsConditional},
    sabi_types::CmpIgnored,
    std_types::{RArc,RVec, StaticSlice, StaticStr,UTypeId,RBoxError,RResult,ROk,RErr},
    traits::IntoReprC,
    type_layout::{
        TypeLayout, TLData, TLDataDiscriminant, TLField, 
        FullType, ReprAttr, TLDiscriminant,TLPrimitive,
        TLEnum,IsExhaustive,IncompatibleWithNonExhaustive,TLNonExhaustive,
        TLFieldOrFunction, TLFunction,
        tagging::TagErrors,
    },
    type_level::unerasability::TU_Opaque,
    utils::{max_by,min_max_by},
};

/// All the errors from checking the layout of every nested type in TypeLayout.
#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct AbiInstabilityErrors {
    pub interface: &'static TypeLayout,
    pub implementation: &'static TypeLayout,
    pub errors: RVec<AbiInstabilityError>,
    _priv:(),
}

/// All the shallow errors from checking an individual type.
///
/// Error that happen lower or higher on the stack are stored in separate
///  `AbiInstabilityError`s.
#[derive(Debug,Clone, PartialEq)]
#[repr(C)]
pub struct AbiInstabilityError {
    pub stack_trace: RVec<ExpectedFound<TLFieldOrFunction>>,
    pub errs: RVec<AbiInstability>,
    pub index: usize,
    _priv:(),
}

/// An individual error from checking the layout of some type.
#[derive(Debug, PartialEq,Clone)]
pub enum AbiInstability {
    ReentrantLayoutCheckingCall,
    NonZeroness(ExpectedFound<bool>),
    Name(ExpectedFound<FullType>),
    Package(ExpectedFound<StaticStr>),
    PackageVersionParseError(ParseVersionError),
    PackageVersion(ExpectedFound<VersionStrings>),
    MismatchedPrefixSize(ExpectedFound<usize>),
    Size(ExpectedFound<usize>),
    Alignment(ExpectedFound<usize>),
    GenericParamCount(ExpectedFound<FullType>),
    TLDataDiscriminant(ExpectedFound<TLDataDiscriminant>),
    MismatchedPrimitive(ExpectedFound<TLPrimitive>),
    FieldCountMismatch(ExpectedFound<usize>),
    FieldLifetimeMismatch(ExpectedFound<TLField>),
    FnLifetimeMismatch(ExpectedFound<TLFunction>),
    UnexpectedField(ExpectedFound<TLField>),
    TooManyVariants(ExpectedFound<usize>),
    MismatchedPrefixConditionality(ExpectedFound<StaticSlice<IsConditional>>),
    MismatchedExhaustiveness(ExpectedFound<IsExhaustive>),
    UnexpectedVariant(ExpectedFound<StaticStr>),
    ReprAttr(ExpectedFound<ReprAttr>),
    EnumDiscriminant(ExpectedFound<TLDiscriminant>),
    IncompatibleWithNonExhaustive(IncompatibleWithNonExhaustive),
    NoneExtraChecks,
    ExtraCheckError(CmpIgnored<ExtraCheckError>),
    TagError{
        err:TagErrors,
    },
}

#[derive(Debug,Clone)]
pub struct ExtraCheckError{
    pub err:RArc<RBoxError>,
    pub expected_err:ExpectedFound<ExtraChecksRef>,
}


use self::AbiInstability as AI;

#[allow(dead_code)]
impl AbiInstabilityErrors {
    #[cfg(test)]
    pub fn flatten_errors(&self) -> RVec<AbiInstability> {
        self.flattened_errors()
            .collect::<RVec<AbiInstability>>()
    }

    #[cfg(test)]
    pub fn flattened_errors<'a>(&'a self) -> impl Iterator<Item=AbiInstability>+'a {
        self.errors
            .iter()
            .flat_map(|x| &x.errs )
            .cloned()
    }
}

impl fmt::Debug for AbiInstabilityErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self,f)
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
        let mut extra_err=None::<String>;

        write!(f, "{} error(s)", self.errs.len())?;
        if self.stack_trace.is_empty() {
            writeln!(f,".")?;
        }else{
            writeln!(f,"inside:\n    <other>\n")?;
        }
        for field in &self.stack_trace {
            writeln!(f, "{}\n", field.found.to_string().left_padder(4))?;
        }
        if let Some(ExpectedFound{expected,found}) = self.stack_trace.last() {
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
                AI::ReentrantLayoutCheckingCall => ("mismatched non-zeroness", None),
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
                AI::MismatchedPrefixSize(v) => 
                    (
                        "prefix-types have a different prefix", 
                        v.display_str()
                    ),
                AI::Size(v) => ("incompatible type size", v.display_str()),
                AI::Alignment(v) => ("incompatible type alignment", v.display_str()),
                AI::GenericParamCount(v) => (
                    "incompatible ammount of generic parameters",
                    v.display_str(),
                ),

                AI::TLDataDiscriminant(v) => ("incompatible data ", v.debug_str()),
                AI::MismatchedPrimitive(v) => ("incompatible primitive", v.debug_str()),
                AI::FieldCountMismatch(v) => ("too many fields", v.display_str()),
                AI::FnLifetimeMismatch(v) => {
                    ("function pointers reference different lifetimes", v.display_str())
                }
                AI::FieldLifetimeMismatch(v) => {
                    ("field references different lifetimes", v.display_str())
                }
                AI::UnexpectedField(v) => ("unexpected field", v.display_str()),
                AI::TooManyVariants(v) => ("too many variants", v.display_str()),
                AI::MismatchedPrefixConditionality(v)=>(
                    "prefix fields differ in whether they are conditional",
                    v.debug_str()
                ),
                AI::MismatchedExhaustiveness(v)=>(
                    "enums differ in whether they are exhaustive",
                    v.debug_str()
                ),
                AI::UnexpectedVariant(v) => ("unexpected variant", v.debug_str()),
                AI::ReprAttr(v)=>("incompatible repr attributes",v.debug_str()),
                AI::EnumDiscriminant(v)=>("different discriminants",v.debug_str()),
                AI::IncompatibleWithNonExhaustive(e)=>{
                    extra_err=Some(e.to_string());

                    ("",None)
                }
                AI::NoneExtraChecks=>{
                    let msg="\
                        Interface contains a value in `extra_checks` \
                        while the implementation does not.\
                    ";
                    (msg, None)
                }
                AI::ExtraCheckError(ec_error) => {
                    let ExtraCheckError{err,expected_err}=&**ec_error;
                    extra_err=Some((**err).to_string());

                    ("", expected_err.display_str())
                },
                AI::TagError{err} => {
                    extra_err=Some(err.to_string());

                    ("", None)
                },
            };

            let (error_msg, expected_err):(&'static str, Option<ExpectedFound<String>>)=pair;

            if let Some(expected_err)=expected_err{
                writeln!(
                    f,
                    "\nError:{}\nExpected:\n{}\nFound:\n{}",
                    error_msg,
                    expected_err.expected.left_padder(4),
                    expected_err.found   .left_padder(4),
                )?;
            }
            if let Some(extra_err)=&extra_err {
                writeln!(f,"\nExtra:\n{}\n",extra_err.left_padder(4))?;
            }
        }
        Ok(())
    }
}

//////


/// What is AbiChecker::check_fields being called with.
#[derive(Debug,Copy,Clone, PartialEq,Eq,Ord,PartialOrd,Hash)]
#[repr(C)]
enum FieldContext{
    Fields,
    Subfields,
    PhantomFields,
}


//////


#[derive(Debug, PartialEq,Eq,Ord,PartialOrd,Hash)]
#[repr(C)]
struct CheckingUTypeId{
    type_id:UTypeId,
    name:StaticStr,
    package:StaticStr,
}

impl CheckingUTypeId{
    fn new(this: &'static TypeLayout)->Self{
        let layout=this;
        Self{
            type_id:this.get_utypeid(),
            name:layout.name,
            package:layout.package(),
        }
    }
}


//////

/// Represents an error where a value was expected,but another value was found.
#[derive(Debug, PartialEq,Clone)]
#[repr(C)]
pub struct ExpectedFound<T> {
    pub expected: T,
    pub found: T,
}

impl<T> ExpectedFound<T> {
    pub fn new<O, F>(this: O, other: O, mut field_getter: F) -> ExpectedFound<T>
    where
        F: FnMut(O) -> T,
    {
        ExpectedFound {
            expected: field_getter(this),
            found: field_getter(other),
        }
    }

    pub fn as_ref(&self) -> ExpectedFound<&T> {
        ExpectedFound {
            expected: &self.expected,
            found: &self.found,
        }
    }

    pub fn map<F, U>(self, mut f: F) -> ExpectedFound<U>
    where
        F: FnMut(T) -> U,
    {
        ExpectedFound {
            expected: f(self.expected),
            found: f(self.found),
        }
    }

    pub fn display_str(&self) -> Option<ExpectedFound<String>>
    where
        T: fmt::Display,
    {
        Some(self.as_ref().map(|x| format!("{:#}", x)))
    }

    pub fn debug_str(&self) -> Option<ExpectedFound<String>>
    where
        T: fmt::Debug,
    {
        Some(self.as_ref().map(|x| format!("{:#?}", x)))
    }
}

///////////////////////////////////////////////





#[derive(Debug)]
#[repr(C)]
pub struct CheckedPrefixTypes{
    this:&'static TypeLayout,
    this_prefix:PrefixTypeMetadata,
    other:&'static TypeLayout,
    other_prefix:PrefixTypeMetadata,
}


#[derive(Debug,Copy,Clone)]
#[repr(C)]
pub struct NonExhaustiveEnumWithContext{
    layout:&'static TypeLayout,
    enum_:&'static TLEnum,
    nonexhaustive:&'static TLNonExhaustive,
}


#[derive(Debug,Copy,Clone)]
#[repr(C)]
pub struct CheckedNonExhaustiveEnums{
    this:NonExhaustiveEnumWithContext,
    other:NonExhaustiveEnumWithContext,
}


///////////////////////////////////////////////

struct AbiChecker {
    stack_trace: RVec<ExpectedFound<TLFieldOrFunction>>,
    checked_prefix_types:RVec<CheckedPrefixTypes>,
    checked_nonexhaustive_enums:RVec<CheckedNonExhaustiveEnums>,

    visited: HashSet<(CheckingUTypeId,CheckingUTypeId)>,
    errors: RVec<AbiInstabilityError>,

    error_index: usize,
}

///////////////////////////////////////////////

impl AbiChecker {
    fn new() -> Self {
        Self {
            stack_trace: RVec::new(),
            checked_prefix_types:RVec::new(),
            checked_nonexhaustive_enums:RVec::new(),

            visited: HashSet::default(),
            errors: RVec::new(),
            error_index: 0,
        }
    }

    #[inline]
    fn check_fields<I,F>(
        &mut self,
        errs: &mut RVec<AbiInstability>,
        t_lay: &'static TypeLayout,
        o_lay: &'static TypeLayout,
        ctx:FieldContext,
        t_fields: I,
        o_fields: I,
    ) 
    where
        I:ExactSizeIterator<Item=F>,
        F:Borrow<TLField>,
    {
        if t_fields.len()==0&&o_fields.len()==0 {
            return;
        }
        
        let is_prefix= match &t_lay.data {
            TLData::PrefixType{..}=>true,
            TLData::Enum(enum_)=>!enum_.exhaustiveness.is_exhaustive(),
            _=>false,
        };
        match (t_fields.len().cmp(&o_fields.len()), is_prefix) {
            (Ordering::Greater, _) | (Ordering::Less, false) => {
                push_err(
                    errs,
                    &t_fields,
                    &o_fields,
                    |x| x.len(),
                    AI::FieldCountMismatch,
                );
            }
            (Ordering::Equal, _) | (Ordering::Less, true) => {}
        }
        
        let acc_fields:Option<(FieldAccessibility,FieldAccessibility)>=
            match (&t_lay.data,&o_lay.data) {
                (TLData::PrefixType(t_prefix), TLData::PrefixType(o_prefix))=>
                    Some((t_prefix.accessible_fields, o_prefix.accessible_fields)),
                _=>None,
            };


        for (field_i,(this_f,other_f)) in t_fields.into_iter().zip(o_fields).enumerate() {
            let this_f=this_f.borrow();
            let other_f=other_f.borrow();
            if this_f.name != other_f.name {
                push_err(errs, this_f, other_f, |x| *x, AI::UnexpectedField);
                continue;
            }

            let t_field_abi=this_f.layout.get();
            let o_field_abi=other_f.layout.get();

            let is_accessible=match (ctx,acc_fields) {
                (FieldContext::Fields,Some((l,r))) => {
                    l.is_accessible(field_i)&&r.is_accessible(field_i)
                },
                _ => true,
            };

            if is_accessible {

                if this_f.lifetime_indices != other_f.lifetime_indices {
                    push_err(errs, this_f, other_f, |x| *x, AI::FieldLifetimeMismatch);
                }

                self.stack_trace.push(ExpectedFound{
                    expected:(*this_f).into(),
                    found:(*other_f).into(),
                });

                let sf_ctx=FieldContext::Subfields;

                for (t_func,o_func) in this_f.function_range.iter().zip(other_f.function_range) {
                    self.error_index += 1;
                    let errs_index = self.error_index;
                    let mut errs_ = RVec::<AbiInstability>::new();
                    let errs=&mut errs_;

                    self.stack_trace.push(ExpectedFound{
                        expected:t_func.into(),
                        found:o_func.into(),
                    });

                    if t_func.paramret_lifetime_indices != o_func.paramret_lifetime_indices {
                        push_err(errs, t_func, o_func, |x| x, AI::FnLifetimeMismatch);
                    }

                    self.check_fields(
                        errs,
                        t_lay,
                        o_lay,
                        sf_ctx,
                        t_func.get_params_ret_iter(),
                        o_func.get_params_ret_iter(),
                    );

                    if !errs_.is_empty() {
                        self.errors.push(AbiInstabilityError {
                            stack_trace: self.stack_trace.clone(),
                            errs: errs_,
                            index: errs_index,
                            _priv:(),
                        });
                    }

                    self.stack_trace.pop();
                }


                self.check_inner(t_field_abi, o_field_abi);
                self.stack_trace.pop();
            }else{
                self.stack_trace.push(ExpectedFound{
                    expected:(*this_f).into(),
                    found:(*other_f).into(),
                });
                
                let t_field_layout=&t_field_abi;
                let o_field_layout=&o_field_abi;
                if  t_field_layout.size!=o_field_layout.size {
                    push_err(errs, t_field_layout, o_field_layout, |x| x.size, AI::Size);
                }
                if t_field_layout.alignment != o_field_layout.alignment {
                    push_err(
                        errs, 
                        t_field_layout, 
                        o_field_layout, 
                        |x| x.alignment, 
                        AI::Alignment
                    );
                }
                self.stack_trace.pop();
            }

        }
    }

    fn check_inner(&mut self, this: &'static TypeLayout, other: &'static TypeLayout) {
        let t_cuti=CheckingUTypeId::new(this );
        let o_cuti=CheckingUTypeId::new(other);
        if !self.visited.insert((t_cuti,o_cuti)) {
            return;
        }

        self.error_index += 1;
        let errs_index = self.error_index;
        let mut errs_ = RVec::<AbiInstability>::new();
        let t_lay = &this;
        let o_lay = &other;

        (|| {
            let errs = &mut errs_;
            if t_lay.name != o_lay.name {
                push_err(errs, t_lay, o_lay, |x| x.full_type(), AI::Name);
                return;
            }
            let (t_package,t_ver_str)=t_lay.package_and_version();
            let (o_package,o_ver_str)=o_lay.package_and_version();
            if t_package != o_package {
                push_err(errs, t_lay, o_lay, |x| x.package(), AI::Package);
                return;
            }

            if this.is_nonzero() != other.is_nonzero() {
                push_err(errs, this, other, |x| x.is_nonzero(), AI::NonZeroness);
            }

            if t_lay.repr_attr != o_lay.repr_attr {
                push_err(errs, t_lay, o_lay, |x| x.repr_attr, AI::ReprAttr);
            }

            {
                let x = (|| {
                    let l = t_ver_str.parsed()?;
                    let r = o_ver_str.parsed()?;
                    Ok(l.is_compatible(r))
                })();
                match x {
                    Ok(false) => {
                        push_err(
                            errs,
                            t_lay,
                            o_lay,
                            |x| x.package_version(),
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
                    // || t_gens.type_.len() != o_gens.type_.len()
                    // || t_gens.const_.len() != o_gens.const_.len()
                {
                    push_err(errs, t_lay, o_lay, |x| x.full_type, AI::GenericParamCount);
                }
            }

            // Checking phantom fields
            self.check_fields(
                errs,
                this,
                other,
                FieldContext::PhantomFields,
                this.phantom_fields.as_slice().iter(),
                other.phantom_fields.as_slice().iter(),
            );

            match (t_lay.size.cmp(&o_lay.size), this.is_prefix_kind()) {
                (Ordering::Greater, _) | (Ordering::Less, false) => {
                    push_err(errs, t_lay, o_lay, |x| x.size, AI::Size);
                }
                (Ordering::Equal, _) | (Ordering::Less, true) => {}
            }
            if t_lay.alignment != o_lay.alignment {
                push_err(errs, t_lay, o_lay, |x| x.alignment, AI::Alignment);
            }

            let t_discr = t_lay.data.as_discriminant();
            let o_discr = o_lay.data.as_discriminant();
            if t_discr != o_discr {
                errs.push(AI::TLDataDiscriminant(ExpectedFound {
                    expected: t_discr,
                    found: o_discr,
                }));
            }

            let t_tag=t_lay.tag.to_checkable();
            let o_tag=o_lay.tag.to_checkable();
            if let Err(tag_err)=t_tag.check_compatible(&o_tag) {
                errs.push(AI::TagError{
                    err:tag_err,
                });
            }

            match (t_lay.extra_checks(),o_lay.extra_checks()) {
                (None,_)=>{}
                (Some(_),None)=>{
                    errs.push(AI::NoneExtraChecks);
                }
                (Some(t_extra_checks),Some(o_extra_checks))=>{
                    let checker=TypeChecker_TO::from_ptr(&mut *self,TU_Opaque);

                    match t_extra_checks.check_compatibility(t_lay,o_lay,checker) {
                        ROk(())|RErr(ExtraChecksError::TypeChecker)=>{}
                        RErr(ExtraChecksError::NoneExtraChecks)=>{
                            errs.push(AI::NoneExtraChecks);
                        }
                        RErr(ExtraChecksError::ExtraChecks(et_err))=>{
                            let e=ExtraCheckError{
                                    err: RArc::new(et_err),
                                    expected_err: ExpectedFound{
                                        expected:t_extra_checks,
                                        found:o_extra_checks,
                                    }
                                }.piped(CmpIgnored::new)
                                 .piped(AI::ExtraCheckError);

                            errs.push(e);
                        }
                    }
                }
            }

            match (t_lay.data, o_lay.data) {
                (TLData::Opaque{..}, _) => {
                    // No checks are necessary
                }

                (TLData::Primitive(t_prim), TLData::Primitive(o_prim)) => {
                    if t_prim != o_prim {
                        errs.push(AI::MismatchedPrimitive(ExpectedFound {
                            expected: t_prim,
                            found: o_prim,
                        }));
                    }
                }
                (TLData::Primitive{..}, _) => {}
                
                (TLData::Struct { fields: t_fields }, TLData::Struct { fields: o_fields }) => {
                    self.check_fields(
                        errs, 
                        this,
                        other,
                        FieldContext::Fields, 
                        t_fields.get_fields(), 
                        o_fields.get_fields()
                    );
                }
                (TLData::Struct { .. }, _) => {}
                
                (TLData::Union { fields: t_fields }, TLData::Union { fields: o_fields }) => {
                    self.check_fields(
                        errs, 
                        this,
                        other,
                        FieldContext::Fields, 
                        t_fields.get_fields(), 
                        o_fields.get_fields()
                    );
                }
                (TLData::Union { .. }, _) => {}
                
                ( TLData::Enum(t_enum),TLData::Enum(o_enum)  ) => {
                    self.check_enum(errs,this,other,t_enum,o_enum);
                    let t_as_ne=t_enum.exhaustiveness.as_nonexhaustive();
                    let o_as_ne=o_enum.exhaustiveness.as_nonexhaustive();
                    if let (Some(this_ne),Some(other_ne))=(t_as_ne,o_as_ne) {
                        self.checked_nonexhaustive_enums.push(CheckedNonExhaustiveEnums{
                            this:NonExhaustiveEnumWithContext{
                                layout:this,
                                enum_:t_enum,
                                nonexhaustive:this_ne,
                            },
                            other:NonExhaustiveEnumWithContext{
                                layout:other,
                                enum_:o_enum,
                                nonexhaustive:other_ne,
                            },
                        });
                    }
                }
                (TLData::Enum { .. }, _) => {}
                
                (
                    TLData::PrefixType (t_prefix),
                    TLData::PrefixType (o_prefix),
                ) => {
                    let this_prefix =PrefixTypeMetadata::with_prefix_layout(t_prefix,t_lay);
                    let other_prefix=PrefixTypeMetadata::with_prefix_layout(o_prefix,o_lay);

                    self.check_prefix_types(errs,&this_prefix,&other_prefix);

                    self.checked_prefix_types.push(
                        CheckedPrefixTypes{this,this_prefix,other,other_prefix}
                    )
                }
                ( TLData::PrefixType {..}, _ ) => {}
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


    fn check_enum(
        &mut self,
        errs: &mut RVec<AbiInstability>,
        this: &'static TypeLayout,other: &'static TypeLayout,
        t_enum:&'static TLEnum,o_enum:&'static TLEnum,
    ){
        let TLEnum{ fields: t_fields,.. }=t_enum;
        let TLEnum{ fields: o_fields,.. }=o_enum;

        let t_fcount = t_enum.field_count.as_slice();
        let o_fcount = o_enum.field_count.as_slice();

        let t_exhaus=t_enum.exhaustiveness;
        let o_exhaus=o_enum.exhaustiveness;

        match (t_exhaus.as_nonexhaustive(),o_exhaus.as_nonexhaustive()) {
            (Some(this_ne),Some(other_ne))=>{
                if let Err(e)=this_ne.check_compatible(this){
                    errs.push(AI::IncompatibleWithNonExhaustive(e))
                }
                if let Err(e)=other_ne.check_compatible(other){
                    errs.push(AI::IncompatibleWithNonExhaustive(e))
                }
            }
            (Some(_),None)|(None,Some(_))=>{
                push_err(
                    errs,
                    t_enum,
                    o_enum, 
                    |x| x.exhaustiveness, 
                    AI::MismatchedExhaustiveness
                );
            }
            (None,None)=>{}
        }

        if t_exhaus.is_exhaustive()&&t_fcount.len()!=o_fcount.len() ||
           t_exhaus.is_nonexhaustive()&&t_fcount.len() >o_fcount.len()
        {
            push_err(errs, t_fcount, o_fcount, |x| x.len(), AI::TooManyVariants);
        }

        
        if let Err(d_errs)=t_enum.discriminants.compare(&o_enum.discriminants) {
            errs.extend(d_errs);
        }

        let mut t_names=t_enum.variant_names.as_str().split(';');
        let mut o_names=o_enum.variant_names.as_str().split(';');
        let mut total_field_count=0;
        for (t_field_count, o_field_count) in t_fcount.iter().zip(o_fcount) {
            let t_name = t_names.next().unwrap_or("<this unavailable>");
            let o_name = o_names.next().unwrap_or("<other unavailable>");
            
            total_field_count+=usize::from(*t_field_count);

            if t_field_count!=o_field_count {
                push_err(
                    errs, 
                    *t_field_count, 
                    *o_field_count, 
                    |x| x as usize, 
                    AI::FieldCountMismatch
                );
            }

            if t_name != o_name {
                push_err(errs, t_name, o_name,StaticStr::new, AI::UnexpectedVariant);
                continue;
            }
        }

        let min_field_count=t_fields.len().min(o_fields.len());
        if total_field_count!=min_field_count {
            push_err(errs, total_field_count, min_field_count,|x|x, AI::FieldCountMismatch);
        }

        self.check_fields(
            errs, 
            this, 
            other, 
            FieldContext::Fields,
            t_fields.get_fields(), 
            o_fields.get_fields()
        );
    }


    fn check_prefix_types(
        &mut self,
        errs: &mut RVec<AbiInstability>,
        this: &PrefixTypeMetadata,
        other: &PrefixTypeMetadata,
    ){
        if this.prefix_field_count != other.prefix_field_count {
            push_err(
                errs,
                this,
                other,
                |x| x.prefix_field_count ,
                AI::MismatchedPrefixSize
            );
        }

        if this.conditional_prefix_fields != other.conditional_prefix_fields {
            push_err(
                errs,
                this,
                other,
                |x| StaticSlice::new(x.conditional_prefix_fields) ,
                AI::MismatchedPrefixConditionality
            );
        }


        self.check_fields(
            errs,
            this.layout,
            other.layout,
            FieldContext::Fields,
            this.fields.get_fields(),
            other.fields.get_fields()
        );
    }


    /// Combines the prefix types into a global map of prefix types.
    fn final_prefix_type_checks(
        &mut self,
        globals:&CheckingGlobals
    )->Result<(),AbiInstabilityError>{
        self.error_index += 1;
        let mut errs_ = RVec::<AbiInstability>::new();
        let errs =&mut errs_;

        let mut prefix_type_map=globals.prefix_type_map.lock().unwrap();

        for pair in mem::replace(&mut self.checked_prefix_types,Default::default()) {
            // let t_lay=pair.this_prefix;
            let errors_before=self.errors.len();
            let t_utid=pair.this .get_utypeid();
            let o_utid=pair.other.get_utypeid();
            // let t_fields=pair.this_prefix.fields;
            // let o_fields=pair.other_prefix.fields;

            let t_index=prefix_type_map.get_index(&t_utid);
            let mut o_index=prefix_type_map.get_index(&o_utid);

            if t_index==o_index{
                o_index=None;
            }

            let (min_prefix,mut max_prefix)=pair.this_prefix.min_max(pair.other_prefix);

            match (t_index,o_index) {
                (None,None)=>{
                    max_prefix.combine_fields_from(&min_prefix);

                    let i=prefix_type_map
                        .get_or_insert(t_utid,max_prefix)
                        .into_inner()
                        .index;
                    prefix_type_map.associate_key(o_utid,i);
                }
                (Some(im_index),None)|(None,Some(im_index))=>{
                    max_prefix.combine_fields_from(&min_prefix);

                    let im_prefix=prefix_type_map.get_mut_with_index(im_index).unwrap();
                    let im_prefix_addr=im_prefix as *const _ as usize;

                    let (min_prefix,max_prefix)=
                        min_max_by(im_prefix,&mut max_prefix,|x|x.fields.len());

                    self.check_prefix_types(errs,min_prefix,max_prefix);
                    if !errs.is_empty() || errors_before!=self.errors.len() { break; }

                    max_prefix.combine_fields_from(&*min_prefix);
                    
                    if im_prefix_addr != (max_prefix as *mut _ as usize) {
                        mem::swap(min_prefix,max_prefix);
                    }
                    
                    prefix_type_map.associate_key(t_utid,im_index);
                    prefix_type_map.associate_key(o_utid,im_index);

                }
                (Some(l_index),Some(r_index))=>{
                    let (l_prefix,r_prefix)=
                        prefix_type_map.get2_mut_with_index(l_index,r_index);
                    let l_prefix=l_prefix.unwrap();
                    let r_prefix=r_prefix.unwrap();

                    let (replace,with)=if l_prefix.fields.len() < r_prefix.fields.len() {
                        (l_index,r_index)
                    }else{
                        (r_index,l_index)
                    };

                    let (min_prefix,max_prefix)=min_max_by(l_prefix,r_prefix,|x|x.fields.len());
                    self.check_prefix_types(errs,min_prefix,max_prefix);
                    if !errs.is_empty() || errors_before!=self.errors.len() { break; }

                    max_prefix.combine_fields_from(&*min_prefix);

                    prefix_type_map.replace_with_index(replace,with);

                }
            }
        }

        if errs_.is_empty() {
            Ok(())
        }else{
            Err(AbiInstabilityError {
                stack_trace: self.stack_trace.clone(),
                errs: errs_,
                index: self.error_index,
                _priv:(),
            })
        }
    }

    /// Combines the nonexhaustive enums into a global map of nonexhaustive enums.
    fn final_non_exhaustive_enum_checks(
        &mut self,
        globals:&CheckingGlobals
    )->Result<(),AbiInstabilityError>{
        self.error_index += 1;
        let mut errs_ = RVec::<AbiInstability>::new();
        let errs =&mut errs_;

        let mut nonexhaustive_map=globals.nonexhaustive_map.lock().unwrap();


        for pair in mem::replace(&mut self.checked_nonexhaustive_enums,Default::default()) {
            let CheckedNonExhaustiveEnums{this,other}=pair;
            let errors_before=self.errors.len();
            
            let t_utid=this .layout.get_utypeid();
            let o_utid=other.layout.get_utypeid();

            let t_index=nonexhaustive_map.get_index(&t_utid);
            let mut o_index=nonexhaustive_map.get_index(&o_utid);

            if t_index==o_index{
                o_index=None;
            }

            let mut max_=max_by(this,other,|x|x.enum_.variant_count());
            
            match (t_index,o_index) {
                (None,None)=>{
                    let i=nonexhaustive_map
                        .get_or_insert(t_utid,max_)
                        .into_inner()
                        .index;
                    
                    nonexhaustive_map.associate_key(o_utid,i);
                }
                (Some(im_index),None)|(None,Some(im_index))=>{
                    let im_nonexh=nonexhaustive_map.get_mut_with_index(im_index).unwrap();
                    let im_nonexh_addr=im_nonexh as *const _ as usize;

                    let (min_nonexh,max_nonexh)=
                        min_max_by(im_nonexh,&mut max_,|x|x.enum_.variant_count());

                    self.check_enum(
                        errs,
                        min_nonexh.layout,max_nonexh.layout,
                        min_nonexh.enum_   ,max_nonexh.enum_   ,
                    );

                    if !errs.is_empty() || errors_before!=self.errors.len() { break; }

                    if im_nonexh_addr != (max_nonexh as *mut _ as usize) {
                        mem::swap(min_nonexh,max_nonexh);
                    }

                    nonexhaustive_map.associate_key(t_utid,im_index);
                    nonexhaustive_map.associate_key(o_utid,im_index);
                }
                (Some(l_index),Some(r_index))=>{
                    let (l_nonexh,r_nonexh)=
                        nonexhaustive_map.get2_mut_with_index(l_index,r_index);
                    let l_nonexh=l_nonexh.unwrap();
                    let r_nonexh=r_nonexh.unwrap();

                    let (replace,with)=
                        if l_nonexh.enum_.variant_count() < r_nonexh.enum_.variant_count() {
                            (l_index,r_index)
                        }else{
                            (r_index,l_index)
                        };

                    let (min_nonexh,max_nonexh)=
                        min_max_by(l_nonexh,r_nonexh,|x|x.enum_.variant_count());
                    
                    self.check_enum(
                        errs,
                        min_nonexh.layout,max_nonexh.layout,
                        min_nonexh.enum_   ,max_nonexh.enum_   ,
                    );

                    if !errs.is_empty() || errors_before!=self.errors.len() { break; }

                    nonexhaustive_map.replace_with_index(replace,with);
                }
            }
        }

        if errs_.is_empty() {
            Ok(())
        }else{
            Err(AbiInstabilityError {
                stack_trace: self.stack_trace.clone(),
                errs: errs_,
                index: self.error_index,
                _priv:(),
            })
        }
    }

    

}


/**
Checks that the layout of `interface` is compatible with `implementation`.

# Warning

This function is not symmetric,
the first parameter must be the expected layout,
and the second must be actual layout.

*/
pub(super) fn check_layout_compatibility(
    interface: &'static TypeLayout,
    implementation: &'static TypeLayout,
) -> Result<(), AbiInstabilityErrors> {
    check_layout_compatibility_with_globals(
        interface,
        implementation,
        get_checking_globals(),
    )
}


#[inline(never)]
pub(super) fn check_layout_compatibility_with_globals(
    interface: &'static TypeLayout,
    implementation: &'static TypeLayout,
    globals:&CheckingGlobals,
) -> Result<(), AbiInstabilityErrors> {
    let mut errors: RVec<AbiInstabilityError>;

    if interface.is_prefix_kind() || implementation.is_prefix_kind() {
        let mut errs=RVec::with_capacity(1);
        push_err(
            &mut errs,
            interface,
            implementation,
            |x| x.data.as_discriminant() ,
            AI::TLDataDiscriminant
        );
        errors = vec![AbiInstabilityError {
            stack_trace: vec![].into(),
            errs,
            index: 0,
            _priv:(),
        }]
        .into();
    } else {
        let mut checker = AbiChecker::new();
        checker.check_inner(interface, implementation);
        if checker.errors.is_empty() {
            if let Err(e)=checker.final_prefix_type_checks(globals) {
                checker.errors.push(e);
            }
            if let Err(e)=checker.final_non_exhaustive_enum_checks(globals) {
                checker.errors.push(e);
            }
        }
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

/**
Checks that the layout of `interface` is compatible with `implementation`,
*/
pub(crate) extern fn check_layout_compatibility_for_ffi(
    interface: &'static TypeLayout,
    implementation: &'static TypeLayout,
) -> RResult<(), RBoxError> {
    extern_fn_panic_handling!{
        let mut is_already_inside=false;
        INSIDE_LAYOUT_CHECKER.with(|inside|{
            is_already_inside=inside.get();
            inside.set(true);
        });
        let _guard=LayoutCheckerGuard;

        if is_already_inside {
            let errors = 
                vec![AbiInstabilityError {
                    stack_trace: vec![].into(),
                    errs:vec![AbiInstability::ReentrantLayoutCheckingCall].into(),
                    index: 0,
                    _priv:(),
                }].into_c();

            Err(AbiInstabilityErrors{ interface, implementation, errors, _priv:() })
        }else{
            check_layout_compatibility(interface,implementation)
        }.map_err(RBoxError::from_fmt)
         .into_c()
    }
}


/**
Checks that the layout of `interface` is compatible with `implementation`,

If this function is called within a dynamic library,
it must be called at or after the function that exports its root module is called.

**DO NOT** call this in the static initializer of a dynamic library,
since this library relies on setting up its global state before
calling the root module loader.

# Warning

This function is not symmetric,
the first parameter must be the expected layout,
and the second must be actual layout.


*/
pub extern fn exported_check_layout_compatibility(
    interface: &'static TypeLayout,
    implementation: &'static TypeLayout,
) -> RResult<(), RBoxError> {
    extern_fn_panic_handling!{
        (crate::globals::initialized_globals().layout_checking)
            (interface,implementation)
    }
}



impl TypeChecker for AbiChecker{
    fn check_compatibility(
        &mut self,
        interface:&'static TypeLayout,
        implementation:&'static TypeLayout,
    )->RResult<(), ExtraChecksError> {
        let error_count_before=self.errors.len();
        
        self.check_inner(interface,implementation);

        if error_count_before==self.errors.len() {
            ROk(())
        }else{
            RErr(ExtraChecksError::TypeChecker)
        }
    }
}


///////////////////////////////////////////////


thread_local!{
    static INSIDE_LAYOUT_CHECKER:Cell<bool>=Cell::new(false);
}


struct LayoutCheckerGuard;

impl Drop for LayoutCheckerGuard{
    fn drop(&mut self){
        INSIDE_LAYOUT_CHECKER.with(|inside|{
            inside.set(false);
        });
    }
}


///////////////////////////////////////////////

use std::sync::Mutex;

use crate::{
    sabi_types::LateStaticRef,
    multikey_map::MultiKeyMap,
    prefix_type::PrefixTypeMetadata,
    utils::leak_value,
};

#[derive(Debug)]
pub struct CheckingGlobals{
    pub(crate) prefix_type_map:Mutex<MultiKeyMap<UTypeId,PrefixTypeMetadata>>,
    pub(crate) nonexhaustive_map:Mutex<MultiKeyMap<UTypeId,NonExhaustiveEnumWithContext>>,
}

impl CheckingGlobals{
    pub fn new()->Self{
        CheckingGlobals{
            prefix_type_map:MultiKeyMap::new().piped(Mutex::new),
            nonexhaustive_map:MultiKeyMap::new().piped(Mutex::new),
        }
    }
}

static CHECKING_GLOBALS:LateStaticRef<CheckingGlobals>=LateStaticRef::new();

pub fn get_checking_globals()->&'static CheckingGlobals{
    CHECKING_GLOBALS.init(||{
        CheckingGlobals::new().piped(leak_value)
    })
}


///////////////////////////////////////////////

pub(crate) fn push_err<O, U, FG, VC>(
    errs: &mut RVec<AbiInstability>,
    this: O,
    other: O,
    field_getter: FG,
    mut variant_constructor: VC,
) where
    FG: FnMut(O) -> U,
    VC: FnMut(ExpectedFound<U>) -> AbiInstability,
{
    let x = ExpectedFound::new(this, other, field_getter);
    let x = variant_constructor(x);
    errs.push(x);
}
