/*!
Functions and types related to the layout checking.
*/

use std::{cmp::Ordering, fmt,mem};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use std::collections::HashSet;
// use std::collections::HashSet;

use super::{
    AbiInfo, AbiInfoWrapper, StableAbi,
    stable_abi_trait::TypeKind,
    type_layout::{
        TypeLayout, TLData, TLDataDiscriminant, TLEnumVariant, TLField,TLFieldAndType, 
        FullType,
    },
};
use crate::{
    version::{ParseVersionError, VersionStrings},
    std_types::{RVec, StaticSlice, StaticStr,utypeid::UTypeId,RBoxError,RResult},
    traits::IntoReprC,
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
    MismatchedPrefixSize(ExpectedFoundError<usize>),
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


#[derive(Debug, PartialEq,Eq,Ord,PartialOrd,Hash)]
#[repr(C)]
struct CheckingUTypeId{
    type_id:UTypeId,
    name:StaticStr,
    package:StaticStr,
}

impl CheckingUTypeId{
    fn new(this: &'static AbiInfo)->Self{
        let layout=this.layout;
        Self{
            type_id:(this.type_id.function)(),
            name:layout.name,
            package:layout.package,
        }
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





#[derive(Debug)]
#[repr(C)]
pub struct CheckedPrefixTypes{
    this:&'static AbiInfo,
    this_prefix:PrefixTypeMetadata,
    other:&'static AbiInfo,
    other_prefix:PrefixTypeMetadata,
}


///////////////////////////////////////////////

struct AbiChecker {
    stack_trace: RVec<TLFieldAndType>,
    checked_prefix_types:RVec<CheckedPrefixTypes>,

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

            visited: HashSet::default(),
            errors: RVec::new(),
            error_index: 0,
        }
    }

    #[inline]
    fn check_fields(
        &mut self,
        errs: &mut RVec<AbiInstability>,
        t_lay: &'static TypeLayout,
        t_fields: StaticSlice<TLField>,
        o_fields: StaticSlice<TLField>,
    ) {
        let t_fields = t_fields.as_slice();
        let o_fields = o_fields.as_slice();
        let is_prefix= t_lay.data.discriminant() == TLDataDiscriminant::PrefixType;
        match (t_fields.len().cmp(&o_fields.len()), is_prefix) {
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
            
            self.check_fields(
                errs,
                t_lay,
                this_f.subfields,
                other_f.subfields,
            );

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
        let t_cuti=CheckingUTypeId::new(this );
        let o_cuti=CheckingUTypeId::new(other);
        if !self.visited.insert((t_cuti,o_cuti)) {
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
                this.layout,
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
                    self.check_fields(errs, this.layout, t_fields, o_fields);
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
                        self.check_fields(errs, this.layout, t_vari.fields, o_vari.fields);
                    }
                }
                (TLData::Enum { .. }, _) => {}
                (
                    TLData::PrefixType {
                        first_suffix_field:t_first_suffix_field,
                        fields:t_fields,
                    },
                    TLData::PrefixType {
                        first_suffix_field:o_first_suffix_field,
                        fields:o_fields
                    },
                ) => {
                    let this_prefix=PrefixTypeMetadata::new(t_lay);
                    let other_prefix=PrefixTypeMetadata::new(o_lay);

                    self.check_prefix_types(errs,this_prefix,other_prefix);

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


    fn check_prefix_types(
        &mut self,
        errs: &mut RVec<AbiInstability>,
        this: PrefixTypeMetadata,
        other: PrefixTypeMetadata,
    ){
        if this.prefix_field_count != other.prefix_field_count {
            push_err(
                errs,
                this ,
                other,
                |x| x.prefix_field_count ,
                AI::MismatchedPrefixSize
            );
        }


        self.check_fields(
            errs,
            this.layout,
            this.fields,
            other.fields
        );
    }


    fn final_prefix_type_checks(
        &mut self,
        globals:&CheckingGlobals
    )->Result<(),AbiInstabilityError>{
        self.error_index += 1;
        let mut errs_ = RVec::<AbiInstability>::new();
        let errs =&mut errs_;

        let mut prefix_type_map=globals.prefix_type_map.lock().unwrap();

        for pair in mem::replace(&mut self.checked_prefix_types,Default::default()) {
            let t_lay=pair.this_prefix.layout;
            let t_utid=pair.this .get_utypeid();
            let o_utid=pair.other.get_utypeid();
            let t_fields=pair.this_prefix.fields;
            let o_fields=pair.other_prefix.fields;

            let t_index=prefix_type_map.get_index(&t_utid);
            let o_index=prefix_type_map.get_index(&o_utid);

            let mut max_prefix=if t_fields.len() < o_fields.len() { 
                pair.this_prefix
            }else{
                pair.other_prefix
            };

            match (t_index,o_index) {
                (None,None)=>{
                    let i=prefix_type_map
                        .get_or_insert(t_utid,max_prefix)
                        .into_inner()
                        .index;
                    prefix_type_map.associate_key(o_utid,i);
                }
                (Some(im_index),None)|(None,Some(im_index))=>{
                    let im_prefix=prefix_type_map.get_mut_with_index(im_index).unwrap();
                    
                    self.check_prefix_types(errs,*im_prefix,max_prefix);
                    if !errs.is_empty() { break; }
                    
                    *im_prefix=im_prefix.max(max_prefix);
                    drop(im_prefix);
                    prefix_type_map.associate_key(t_utid,im_index);
                    prefix_type_map.associate_key(o_utid,im_index);
                }
                (Some(l_index),Some(r_index))=>{
                    let l_prefix=*prefix_type_map.get_with_index(l_index).unwrap();
                    let r_prefix=*prefix_type_map.get_with_index(r_index).unwrap();

                    self.check_prefix_types(errs,l_prefix,r_prefix);
                    if !errs.is_empty() { break; }

                    let (replace,with)=if l_prefix.fields.len() < r_prefix.fields.len() {
                        (l_index,r_index)
                    }else{
                        (r_index,l_index)
                    };
                    if l_prefix.fields.len() != r_prefix.fields.len() {
                        prefix_type_map.replace_with_index(replace,with);
                    }
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
pub fn check_abi_stability(
    interface: &'static AbiInfoWrapper,
    implementation: &'static AbiInfoWrapper,
) -> Result<(), AbiInstabilityErrors> {
    check_abi_stability_with_globals(
        interface,
        implementation,
        get_checking_globals(),
    )
}


/**
Checks that the layout of `interface` is compatible with `implementation`,
passing in the globals updated every time this is called.

# Warning

This function is not symmetric,
the first parameter must be the expected layout,
and the second must be actual layout.

*/
// Never inline this function because it will be called very infrequently and
// will take a long-ish time to run anyway.
#[inline(never)]
pub fn check_abi_stability_with_globals(
    interface: &'static AbiInfoWrapper,
    implementation: &'static AbiInfoWrapper,
    globals:&CheckingGlobals,
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
        if checker.errors.is_empty() {
            if let Err(e)=checker.final_prefix_type_checks(globals) {
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


pub extern fn check_abi_stability_for_ffi(
    interface: &'static AbiInfoWrapper,
    implementation: &'static AbiInfoWrapper,
) -> RResult<(), RBoxError> {
    check_abi_stability(interface,implementation)
        .map_err(RBoxError::from_fmt)
        .into_c()
}



///////////////////////////////////////////////

use std::sync::Mutex;

use crate::{
    lazy_static_ref::LazyStaticRef,
    multikey_map::MultiKeyMap,
    prefix_type::PrefixTypeMetadata,
    utils::leak_value,
};

pub struct CheckingGlobals{
    prefix_type_map:Mutex<MultiKeyMap<UTypeId,PrefixTypeMetadata>>,
}

impl CheckingGlobals{
    pub fn new()->Self{
        CheckingGlobals{
            prefix_type_map:MultiKeyMap::new().piped(Mutex::new),
        }
    }
}

static CHECKING_GLOBALS:LazyStaticRef<CheckingGlobals>=LazyStaticRef::new();

pub fn get_checking_globals()->&'static CheckingGlobals{
    CHECKING_GLOBALS.init(||{
        CheckingGlobals::new().piped(leak_value)
    })
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
