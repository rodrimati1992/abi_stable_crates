/*!
Functions and types related to the layout checking.
*/

use std::{cmp::Ordering, fmt,mem};

#[allow(unused_imports)]
use core_extensions::prelude::*;

use std::{
    collections::HashSet,
    slice,
};
// use std::collections::HashSet;

use super::{
    AbiInfo, AbiInfoWrapper,
    type_layout::{
        TypeLayout, TLData, TLDataDiscriminant, TLEnumVariant, TLField,TLFieldAndType, 
        FullType, ReprAttr, TLDiscriminant,TLPrimitive,
    },
    tagging::{CheckableTag,TagErrors},
};
use crate::{
    version::{ParseVersionError, VersionStrings},
    prefix_type::{FieldAccessibility,IsConditional},
    std_types::{RVec, StaticSlice, StaticStr,utypeid::UTypeId,RBoxError,RResult},
    traits::IntoReprC,
    utils::min_max_by,
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
#[derive(Debug, PartialEq,Clone)]
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
    MismatchedPrimitive(ExpectedFoundError<TLPrimitive>),
    FieldCountMismatch(ExpectedFoundError<usize>),
    FieldLifetimeMismatch(ExpectedFoundError<TLField>),
    UnexpectedField(ExpectedFoundError<TLField>),
    TooManyVariants(ExpectedFoundError<usize>),
    MismatchedPrefixConditionality(ExpectedFoundError<StaticSlice<IsConditional>>),
    UnexpectedVariant(ExpectedFoundError<TLEnumVariant>),
    ReprAttr(ExpectedFoundError<ReprAttr>),
    EnumDiscriminant(ExpectedFoundError<TLDiscriminant>),
    TagError{
        expected_found:ExpectedFoundError<CheckableTag>,
        err:TagErrors,
    },
}



use self::AbiInstability as AI;

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
        let mut extra_err=None::<String>;

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
                AI::MismatchedPrimitive(v) => ("incompatible primitive", v.debug_str()),
                AI::FieldCountMismatch(v) => ("too many fields", v.display_str()),
                AI::FieldLifetimeMismatch(v) => {
                    ("field references different lifetimes", v.debug_str())
                }
                AI::UnexpectedField(v) => ("unexpected field", v.debug_str()),
                AI::TooManyVariants(v) => ("too many variants", v.display_str()),
                AI::MismatchedPrefixConditionality(v)=>(
                    "prefix fields differ in whether they are conditional",
                    v.debug_str()
                ),
                AI::UnexpectedVariant(v) => ("unexpected variant", v.debug_str()),
                AI::ReprAttr(v)=>("incompatible repr attributes",v.debug_str()),
                AI::EnumDiscriminant(v)=>("different discriminants",v.debug_str()),
                AI::TagError{expected_found,err} => {
                    extra_err=Some(err.to_string());

                    ("incompatible tag", expected_found.display_str())
                },
            };

            writeln!(
                f,
                "\nError:{}\nExpected:\n{}\nFound:\n{}",
                error_msg,
                expected_err.expected.left_padder(4),
                expected_err.found   .left_padder(4),
            )?;
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
    fn new(this: &'static AbiInfo)->Self{
        let layout=this.layout;
        Self{
            type_id:(this.type_id.function)(),
            name:layout.name,
            package:layout.package(),
        }
    }
}


//////

/// Represents an error where a value was expected,but another value was found.
#[derive(Debug, PartialEq,Clone)]
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
        o_lay: &'static TypeLayout,
        ctx:FieldContext,
        t_fields: &[TLField],
        o_fields: &[TLField],
    ) {
        if t_fields.is_empty()&&o_fields.is_empty() {
            return;
        }

        let is_prefix= t_lay.data.as_discriminant() == TLDataDiscriminant::PrefixType;
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
        
        let acc_fields:Option<(FieldAccessibility,FieldAccessibility)>=
            match (&t_lay.data,&o_lay.data) {
                (TLData::PrefixType(t_prefix), TLData::PrefixType(o_prefix))=>
                    Some((t_prefix.accessible_fields, o_prefix.accessible_fields)),
                _=>None,
            };


        for (field_i,(this_f,other_f)) in t_fields.iter().zip(o_fields).enumerate() {
            if this_f.name != other_f.name {
                push_err(errs, this_f, other_f, |x| *x, AI::UnexpectedField);
                continue;
            }

            let t_field_abi=this_f.abi_info.get();
            let o_field_abi=other_f.abi_info.get();

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

                self.stack_trace.push(TLFieldAndType::new(*this_f));
                    
                let sf_ctx=FieldContext::Subfields;
                
                for (t_func,o_func) in this_f.functions.iter().zip(&*other_f.functions) {
                    self.check_fields(errs,t_lay,o_lay,sf_ctx,&t_func.params,&o_func.params);

                    let t_returns=t_func.returns.as_ref().map_or(&[][..],slice::from_ref);
                    let o_returns=o_func.returns.as_ref().map_or(&[][..],slice::from_ref);
                    self.check_fields(errs,t_lay,o_lay,sf_ctx,t_returns,o_returns);
                }


                self.check_inner(t_field_abi, o_field_abi);
            }else{
                self.stack_trace.push(TLFieldAndType::new(*this_f));
                
                let t_field_layout=&t_field_abi.layout;
                let o_field_layout=&o_field_abi.layout;
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
            }

            self.stack_trace.pop();
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
            if t_lay.package() != o_lay.package() {
                push_err(errs, t_lay, o_lay, |x| x.package(), AI::Package);
                return;
            }

            if this.prefix_kind != other.prefix_kind {
                push_err(errs, this, other, |x| x.prefix_kind, AI::IsPrefix);
            }
            if this.is_nonzero != other.is_nonzero {
                push_err(errs, this, other, |x| x.is_nonzero, AI::NonZeroness);
            }

            if t_lay.repr_attr != o_lay.repr_attr {
                push_err(errs, t_lay, o_lay, |x| x.repr_attr, AI::ReprAttr);
            }

            {
                let x = (|| {
                    let l = t_lay.package_version().parsed()?;
                    let r = o_lay.package_version().parsed()?;
                    Ok(l.is_compatible(r))
                })();
                match x {
                    Ok(false) => {
                        push_err(
                            errs,
                            t_lay,
                            o_lay,
                            |x| *x.package_version(),
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
                this.layout,
                other.layout,
                FieldContext::PhantomFields,
                &this.layout.phantom_fields,
                &other.layout.phantom_fields,
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

            let t_discr = t_lay.data.as_discriminant();
            let o_discr = o_lay.data.as_discriminant();
            if t_discr != o_discr {
                errs.push(AI::TLDataDiscriminant(ExpectedFoundError {
                    expected: t_discr,
                    found: o_discr,
                }));
            }

            let t_tag=t_lay.tag.to_checkable();
            let o_tag=o_lay.tag.to_checkable();
            if let Err(tag_err)=t_tag.check_compatible(&o_tag) {
                errs.push(AI::TagError{
                    expected_found:ExpectedFoundError{
                        expected:t_tag,
                        found:o_tag,
                    },
                    err:tag_err,
                });
            }

            match (t_lay.data, o_lay.data) {
                (TLData::Opaque{..}, _) => {
                    // No checks are necessary
                }

                (TLData::Primitive(t_prim), TLData::Primitive(o_prim)) => {
                    if t_prim != o_prim {
                        errs.push(AI::MismatchedPrimitive(ExpectedFoundError {
                            expected: t_prim,
                            found: o_prim,
                        }));
                    }
                }
                (TLData::Primitive{..}, _) => {}
                
                (TLData::Struct { fields: t_fields }, TLData::Struct { fields: o_fields }) => {
                    self.check_fields(
                        errs, 
                        this.layout,
                        other.layout,
                        FieldContext::Fields, 
                        &t_fields, 
                        &o_fields
                    );
                }
                (TLData::Struct { .. }, _) => {}
                
                (TLData::Union { fields: t_fields }, TLData::Union { fields: o_fields }) => {
                    self.check_fields(
                        errs, 
                        this.layout,
                        other.layout,
                        FieldContext::Fields, 
                        &t_fields, 
                        &o_fields
                    );
                }
                (TLData::Union { .. }, _) => {}
                
                (TLData::Enum { variants: t_varis }, TLData::Enum { variants: o_varis }) => {
                    let t_varis = t_varis.as_slice();
                    let o_varis = o_varis.as_slice();
                    if t_varis.len() != o_varis.len() {
                        push_err(errs, t_varis, o_varis, |x| x.len(), AI::TooManyVariants);
                    }
                    for (t_vari, o_vari) in t_varis.iter().zip(o_varis) {
                        let t_name = t_vari.name.as_str();
                        let o_name = o_vari.name.as_str();
                        
                        if t_vari.discriminant!=o_vari.discriminant {
                            push_err(
                                errs, 
                                *t_vari, 
                                *o_vari, 
                                |x| x.discriminant, 
                                AI::EnumDiscriminant
                            );
                        }

                        if t_name != o_name {
                            push_err(errs, *t_vari, *o_vari, |x| x, AI::UnexpectedVariant);
                            continue;
                        }
                        self.check_fields(
                            errs, 
                            this.layout, 
                            other.layout, 
                            FieldContext::Fields,
                            &t_vari.fields, 
                            &o_vari.fields
                        );
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
            &this.fields,
            &other.fields
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
            // let t_lay=pair.this_prefix.layout;
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
                    if !errs.is_empty() { break; }

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
                    if !errs.is_empty() { break; }

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
}


/**
Checks that the layout of `interface` is compatible with `implementation`.

# Warning

This function is not symmetric,
the first parameter must be the expected layout,
and the second must be actual layout.

*/
pub(super) fn check_layout_compatibility(
    interface: &'static AbiInfoWrapper,
    implementation: &'static AbiInfoWrapper,
) -> Result<(), AbiInstabilityErrors> {
    check_layout_compatibility_with_globals(
        interface,
        implementation,
        get_checking_globals(),
    )
}


#[inline(never)]
pub(super) fn check_layout_compatibility_with_globals(
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

/**
Checks that the layout of `interface` is compatible with `implementation`,
*/
pub(crate) extern fn check_layout_compatibility_for_ffi(
    interface: &'static AbiInfoWrapper,
    implementation: &'static AbiInfoWrapper,
) -> RResult<(), RBoxError> {
    check_layout_compatibility(interface,implementation)
        .map_err(RBoxError::from_fmt)
        .into_c()
}


/**
Checks that the layout of `interface` is compatible with `implementation`,

# Warning

This function is not symmetric,
the first parameter must be the expected layout,
and the second must be actual layout.

# Safety 

If this function is called within a dynamic library,
it must be called at or after the function that exports its root module is called.

**DO NOT** call this in the static initializer of a dynamic library,
since this library relies on setting up its global state before
calling the root module loader.

*/
pub unsafe extern fn exported_check_layout_compatibility(
    interface: &'static AbiInfoWrapper,
    implementation: &'static AbiInfoWrapper,
) -> RResult<(), RBoxError> {
    extern_fn_panic_handling!{
        (crate::globals::initialized_globals().layout_checking)
            (interface,implementation)
    }
}



///////////////////////////////////////////////

use std::sync::Mutex;

use crate::{
    late_static_ref::LateStaticRef,
    multikey_map::MultiKeyMap,
    prefix_type::PrefixTypeMetadata,
    utils::leak_value,
};

#[derive(Debug)]
pub struct CheckingGlobals{
    pub(crate) prefix_type_map:Mutex<MultiKeyMap<UTypeId,PrefixTypeMetadata>>,
}

impl CheckingGlobals{
    pub fn new()->Self{
        CheckingGlobals{
            prefix_type_map:MultiKeyMap::new().piped(Mutex::new),
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
