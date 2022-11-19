//! Tag is a dynamically typed data structure used to encode extra properties
//! about a type in its layout constant.
//!
//! # Comparison semantics
//!
//! Tags don't use strict equality when doing layout checking ,
//! here is an exhaustive list on what is considered compatible
//! for each variant **in the interface**:
//!
//! - Null:
//!     A Tag which is compatible with any other one.
//!     Note that Nulls are stripped from arrays,set,and map keys.
//!
//! - Integers/bools/strings:
//!     They must be strictly equal.
//!
//! - Arrays:
//!     They must have the same length, and have elements that compare equal.
//!
//! - Sets/Maps:
//!     The set/map in the interface must be a subset of the implementation,
//!
//! # Examples
//!
//!
//! ###  Declaring a unit type with a tag.
//!
#![cfg_attr(not(feature = "no_tagging_doctest"), doc = "```rust")]
#![cfg_attr(feature = "no_tagging_doctest", doc = "```ignore")]
//!
//! use abi_stable::{tag,StableAbi};
//!
//! #[repr(C)]
//! #[derive(StableAbi)]
//! #[sabi( tag = tag!("WAT"))]
//! struct UnitType;
//!
//!
//! # fn main(){}
//!
//!
//! ```
//!
//! ###  Emulating const generics for strings
//!
//! This emulates a `const NAME:&'static str` parameter,
//! which is checked as being the same between the interface and implementation.
//!
//!
#![cfg_attr(not(feature = "no_tagging_doctest"), doc = "```rust")]
#![cfg_attr(feature = "no_tagging_doctest", doc = "```ignore")]
//! use abi_stable::{tag,StableAbi,marker_type::UnsafeIgnoredType};
//!
//!
//! trait Name{
//!     const NAME:&'static str;
//! }
//!
//! ///
//! /// The layout of `StringParameterized<S>` is determined by `<S as Name>::NAME`,
//! /// allowing the interface crate to have a different `S`
//! /// type parameter than the implementation crate,
//! /// so long as they have the same associated `&'static str`.
//! ///
//! /// StringParameterized<Foo> has the "same" layout as StringParameterized<Bar>.
//! ///
//! /// StringParameterized<Foo> has a "different" layout to StringParameterized<Boor>.
//! ///
//! #[repr(C)]
//! #[derive(StableAbi)]
//! #[sabi(
//!     bound(S:Name),
//!     tag = tag!( S::NAME ),
//! )]
//! struct StringParameterized<S>{
//!     _marker:UnsafeIgnoredType<S>
//! }
//!
//! #[repr(C)]
//! #[derive(StableAbi)]
//! struct Foo;
//!
//! impl Name for Foo{
//!     const NAME:&'static str="Hello, World!";
//! }
//!
//!
//! #[repr(C)]
//! #[derive(StableAbi)]
//! struct Bar;
//!
//! impl Name for Bar{
//!     const NAME:&'static str="Hello, Helloooooo!";
//! }
//!
//!
//! #[repr(C)]
//! #[derive(StableAbi)]
//! struct Boor;
//!
//! impl Name for Boor{
//!     const NAME:&'static str="This is a different string!";
//! }
//!
//! # fn main(){}
//!
//! ```
//!
//! ###  Declaring each variant.
//!
#![cfg_attr(not(feature = "no_tagging_doctest"), doc = "```rust")]
#![cfg_attr(feature = "no_tagging_doctest", doc = "```ignore")]
//! use abi_stable::{
//!     rslice,tag,
//!     type_layout::Tag,
//! };
//!
//! const NULL:Tag=Tag::null();
//!
//!
//! const BOOL_MACRO:Tag=tag!( false );
//! const BOOL_FN   :Tag=Tag::bool_(false);
//!
//!
//! const INT_MACRO_0:Tag=tag!(  100 );
//! const INT_FN_0   :Tag=Tag::int(100);
//!
//! const INT_MACRO_1:Tag=tag!( -100 );
//! const INT_FN_1   :Tag=Tag::int(-100);
//!
//!
//! // This can only be declared using the function for now.
//! const UINT:Tag=Tag::uint(100);
//!
//!
//! const STR_0_MACRO:Tag=tag!("Hello,World!");
//! const STR_0_FN:Tag=Tag::str("Hello,World!");
//!
//! const ARR_0_MACRO:Tag=tag![[ 0,1,2,3 ]];
//! const ARR_0_FN:Tag=Tag::arr(rslice![
//!     Tag::int(0),
//!     Tag::int(1),
//!     Tag::int(2),
//!     Tag::int(3),
//! ]);
//!
//!
//! const SET_0_MACRO:Tag=tag!{{ 0,1,2,3 }};
//! const SET_0_FN:Tag=Tag::set(rslice![
//!     Tag::int(0),
//!     Tag::int(1),
//!     Tag::int(2),
//!     Tag::int(3),
//! ]);
//!
//!
//! const MAP_0_MACRO:Tag=tag!{{
//!     0=>"a",
//!     1=>"b",
//!     2=>false,
//!     3=>100,
//! }};
//! const MAP_0_FN:Tag=Tag::map(rslice![
//!     Tag::kv( Tag::int(0), Tag::str("a")),
//!     Tag::kv( Tag::int(1), Tag::str("b")),
//!     Tag::kv( Tag::int(2), Tag::bool_(false)),
//!     Tag::kv( Tag::int(3), Tag::int(100)),
//! ]);
//!
//! # fn main(){}
//!
//! ```
//!
//! ###  Creating a complex data structure.
//!
//!
#![cfg_attr(not(feature = "no_tagging_doctest"), doc = "```rust")]
#![cfg_attr(feature = "no_tagging_doctest", doc = "```ignore")]
//! use abi_stable::{
//!     tag,
//!     type_layout::Tag,
//! };
//!
//! const TAG:Tag=tag!{{
//!     // This must match exactly,
//!     // adding required traits on the interface or the implementation
//!     // would be a breaking change.
//!     "required"=>tag![[
//!         "Copy",
//!     ]],
//!     
//!     "requires at least"=>tag!{{
//!         "Debug",
//!         "Display",
//!     }},
//!
//!
//!     "maps"=>tag!{{
//!         0=>"Zero",
//!         1=>"One",
//!     }}
//! }};
//!
//!
//! ```

use std::{
    collections::BTreeMap,
    fmt::{self, Display},
    mem,
};

use core_extensions::{matches, SelfOps};

use crate::{
    abi_stability::extra_checks::{
        ExtraChecks, ExtraChecksError, ForExtraChecksImplementor, TypeCheckerMut,
    },
    std_types::{RBox, RCowSlice, RNone, ROption, RResult, RSlice, RSome, RStr, RVec},
    traits::IntoReprC,
    type_layout::TypeLayout,
    utils::FmtPadding,
    StableAbi,
};

/// Tag is a dynamically typed data structure used to encode extra properties
/// about a type in its layout constant.
///
/// For more information [look at the module-level documentation](./index.html)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct Tag {
    variant: TagVariant,
}

/// All the Tag variants.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub enum TagVariant {
    ///
    Primitive(Primitive),
    /// A Tag that's considered compatible with any other
    Ignored(&'static Tag),
    ///
    Array(RSlice<'static, Tag>),
    ///
    Set(RSlice<'static, Tag>),
    ///
    Map(RSlice<'static, KeyValue<Tag>>),
}

/// The primitive types of a variant,which do not contain other nested tags.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub enum Primitive {
    ///
    Null,
    ///
    Bool(bool),
    ///
    Int(i64),
    ///
    UInt(u64),
    ///
    String_(RStr<'static>),
}

/// A tag that can be checked for compatibility with another tag.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct CheckableTag {
    variant: CTVariant,
}

/// The possible variants of CheckableTag.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub enum CTVariant {
    ///
    Primitive(Primitive),
    /// A Tag that's considered compatible with any other
    Ignored(RBox<CheckableTag>),
    ///
    Array(RVec<CheckableTag>),
    ///
    Set(RVec<KeyValue<CheckableTag>>),
    ///
    Map(RVec<KeyValue<CheckableTag>>),
}

/// A key-value pair,used when constructing a map.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, StableAbi)]
pub struct KeyValue<T> {
    ///
    pub key: T,
    ///
    pub value: T,
}

#[doc(hidden)]
pub trait TagTrait {
    fn is_null(&self) -> bool;
}

impl TagTrait for Tag {
    fn is_null(&self) -> bool {
        self.variant == TagVariant::Primitive(Primitive::Null)
    }
}

impl<'a> TagTrait for &'a Tag {
    fn is_null(&self) -> bool {
        self.variant == TagVariant::Primitive(Primitive::Null)
    }
}

impl TagTrait for CheckableTag {
    fn is_null(&self) -> bool {
        self.variant == CTVariant::Primitive(Primitive::Null)
    }
}

impl<KV> TagTrait for KeyValue<KV>
where
    KV: TagTrait,
{
    fn is_null(&self) -> bool {
        self.key.is_null()
    }
}

impl<'a, KV> TagTrait for &'a KeyValue<KV>
where
    KV: TagTrait,
{
    fn is_null(&self) -> bool {
        self.key.is_null()
    }
}

impl<'a> TagTrait for &'a CheckableTag {
    fn is_null(&self) -> bool {
        *self == &Tag::null().to_checkable()
    }
}

impl Tag {
    const fn new(variant: TagVariant) -> Self {
        Self { variant }
    }

    /// Constructs the Null variant.
    pub const NULL: &'static Tag = &Tag::null();

    /// Constructs the Null variant.
    pub const fn null() -> Self {
        Self::new(TagVariant::Primitive(Primitive::Null))
    }
    /// Constructs the Bool variant.
    pub const fn bool_(b: bool) -> Self {
        Self::new(TagVariant::Primitive(Primitive::Bool(b)))
    }

    /// Constructs the Int variant.
    pub const fn int(n: i64) -> Self {
        Self::new(TagVariant::Primitive(Primitive::Int(n)))
    }

    /// Constructs the UInt variant.
    pub const fn uint(n: u64) -> Self {
        Self::new(TagVariant::Primitive(Primitive::UInt(n)))
    }

    /// Constructs the String_ variant.
    pub const fn str(s: &'static str) -> Self {
        Self::new(TagVariant::Primitive(Primitive::String_(RStr::from_str(s))))
    }

    /// Constructs the String_ variant.
    pub const fn rstr(s: RStr<'static>) -> Self {
        Self::new(TagVariant::Primitive(Primitive::String_(s)))
    }

    /// Constructs the Ignored variant.
    pub const fn ignored(ignored: &'static Tag) -> Self {
        Self::new(TagVariant::Ignored(ignored))
    }

    /// Constructs the Array variant.
    pub const fn arr(s: RSlice<'static, Tag>) -> Self {
        Self::new(TagVariant::Array(s))
    }

    /// Constructs the Set variant.
    pub const fn set(s: RSlice<'static, Tag>) -> Self {
        Self::new(TagVariant::Set(s))
    }

    /// Constructs a KeyValue.
    pub const fn kv(key: Tag, value: Tag) -> KeyValue<Tag> {
        KeyValue { key, value }
    }

    /// Constructs the Map variant.
    pub const fn map(s: RSlice<'static, KeyValue<Tag>>) -> Self {
        Self::new(TagVariant::Map(s))
    }
}

impl Tag {
    /// Converts the `Tag` into a `CheckableTag`,
    /// so as to check `Tag`s for compatibility.
    pub fn to_checkable(self) -> CheckableTag {
        let variant = match self.variant {
            TagVariant::Primitive(prim) => CTVariant::Primitive(prim),
            TagVariant::Ignored(ignored) => (*ignored)
                .to_checkable()
                .piped(RBox::new)
                .piped(CTVariant::Ignored),
            TagVariant::Array(arr) => arr
                .iter()
                .cloned()
                .filter(|x| *x != Tag::null())
                .map(Self::to_checkable)
                .collect::<RVec<CheckableTag>>()
                .piped(CTVariant::Array),
            TagVariant::Set(arr) => arr
                .iter()
                .cloned()
                .filter(|x| !x.is_null())
                .map(|x| (x.to_checkable(), Tag::null().to_checkable()))
                .piped(sorted_ct_vec_from_iter)
                .piped(CTVariant::Set),
            TagVariant::Map(arr) => arr
                .iter()
                .cloned()
                .filter(|kv| !kv.key.is_null())
                .map(|x| x.map(|y| y.to_checkable()).into_pair())
                .piped(sorted_ct_vec_from_iter)
                .piped(CTVariant::Map),
        };

        CheckableTag { variant }
    }
}

fn sorted_ct_vec_from_iter<I>(iter: I) -> RVec<KeyValue<CheckableTag>>
where
    I: IntoIterator<Item = (CheckableTag, CheckableTag)>,
{
    iter.into_iter()
        .collect::<BTreeMap<CheckableTag, CheckableTag>>()
        .into_iter()
        .map(KeyValue::from_pair)
        .collect::<RVec<KeyValue<CheckableTag>>>()
}

impl CheckableTag {
    /// Checks that this `CheckableTag` is compatible with another one,
    /// returning `Ok` if it is compatible, `Err` if it was not.
    pub fn check_compatible(&self, other: &Self) -> Result<(), TagErrors> {
        use self::CTVariant as CTV;

        let err_with_variant = |vari: TagErrorVariant| TagErrors {
            expected: self.clone(),
            found: other.clone(),
            backtrace: vec![].into(),
            errors: vec![vari].into(),
        };

        let mismatched_val_err = |cond: bool| {
            if cond {
                Ok(())
            } else {
                Err(err_with_variant(TagErrorVariant::MismatchedValue))
            }
        };

        let same_variant = match (&self.variant, &other.variant) {
            (CTV::Primitive(Primitive::Null), _) => return Ok(()),
            (CTV::Primitive(l), CTV::Primitive(r)) => mem::discriminant(l) == mem::discriminant(r),
            (l, r) => mem::discriminant(l) == mem::discriminant(r),
        };

        if !same_variant {
            return Err(err_with_variant(TagErrorVariant::MismatchedDiscriminant));
        }

        let is_map = matches!(self.variant, CTV::Map { .. });

        match (&self.variant, &other.variant) {
            (CTV::Primitive(l), CTV::Primitive(r)) => match (l, r) {
                (Primitive::Null, Primitive::Null) => (),
                (Primitive::Null, _) => (),

                (Primitive::Bool(l_cond), Primitive::Bool(r_cond)) => {
                    mismatched_val_err(l_cond == r_cond)?
                }
                (Primitive::Bool(_), _) => {}

                (Primitive::Int(l_num), Primitive::Int(r_num)) => {
                    mismatched_val_err(l_num == r_num)?
                }
                (Primitive::Int(_), _) => {}

                (Primitive::UInt(l_num), Primitive::UInt(r_num)) => {
                    mismatched_val_err(l_num == r_num)?
                }
                (Primitive::UInt(_), _) => {}

                (Primitive::String_(l_str), Primitive::String_(r_str)) => {
                    mismatched_val_err(l_str.as_str() == r_str.as_str())?
                }
                (Primitive::String_(_), _) => {}
            },
            (CTV::Primitive(_), _) => {}

            (CTV::Ignored(_), _) => {}

            (CTV::Array(l_arr), CTV::Array(r_arr)) => {
                let l_arr = l_arr.as_slice();
                let r_arr = r_arr.as_slice();

                if l_arr.len() != r_arr.len() {
                    let e = TagErrorVariant::MismatchedArrayLength {
                        expected: l_arr.len(),
                        found: r_arr.len(),
                    };
                    return Err(err_with_variant(e));
                }

                for (l_elem, r_elem) in l_arr.iter().zip(r_arr.iter()) {
                    l_elem
                        .check_compatible(r_elem)
                        .map_err(|errs| errs.context(l_elem.clone()))?;
                }
            }
            (CTV::Array(_), _) => {}

            (CTV::Set(l_map), CTV::Set(r_map)) | (CTV::Map(l_map), CTV::Map(r_map)) => {
                if l_map.len() > r_map.len() {
                    let e = TagErrorVariant::MismatchedAssocLength {
                        expected: l_map.len(),
                        found: r_map.len(),
                    };
                    return Err(err_with_variant(e));
                }

                let mut r_iter = r_map.iter().map(KeyValue::as_pair);

                'outer: for (l_key, l_elem) in l_map.iter().map(KeyValue::as_pair) {
                    let mut first_err = None::<KeyValue<&CheckableTag>>;

                    'inner: loop {
                        let (r_key, r_elem) = match r_iter.next() {
                            Some(x) => x,
                            None => break 'inner,
                        };

                        match l_key
                            .check_compatible(r_key)
                            .and_then(|_| l_elem.check_compatible(r_elem))
                        {
                            Ok(_) => continue 'outer,
                            Err(_) => {
                                first_err.get_or_insert(KeyValue::new(r_key, r_elem));
                            }
                        }
                    }

                    let e = if is_map {
                        TagErrorVariant::MismatchedMapEntry {
                            expected: KeyValue::new(l_key.clone(), l_elem.clone()),
                            found: first_err.map(|x| x.map(Clone::clone)).into_c(),
                        }
                    } else {
                        TagErrorVariant::MissingSetValue {
                            expected: l_key.clone(),
                            found: first_err.map(|x| x.key).cloned().into_c(),
                        }
                    };
                    return Err(err_with_variant(e));
                }
            }
            (CTV::Set(_), _) => {}
            (CTV::Map(_), _) => {}
        }
        Ok(())
    }
}

/////////////////////////////////////////////////////////////////

#[allow(clippy::missing_const_for_fn)]
impl<T> KeyValue<T> {
    /// Constructs a KeyValue with `key`,`value`
    pub const fn new(key: T, value: T) -> Self {
        Self { key, value }
    }
    /// Transforms the `KeyValue<T>` to `KeyValue<U>`,
    /// using `f` to convert `T` to `U`.
    pub fn map<F, U>(self, mut f: F) -> KeyValue<U>
    where
        F: FnMut(T) -> U,
    {
        KeyValue {
            key: f(self.key),
            value: f(self.value),
        }
    }

    /// Converts the KeyValue into a `(key, value)` pair.
    pub fn into_pair(self) -> (T, T) {
        (self.key, self.value)
    }

    /// Casts a &KeyValue into a `(key, value)` pair of references.
    pub const fn as_pair(&self) -> (&T, &T) {
        (&self.key, &self.value)
    }

    /// Converts a `(key, value)` pair into a KeyValue.
    pub fn from_pair((key, value): (T, T)) -> Self {
        Self { key, value }
    }
}

impl<T> Display for KeyValue<T>
where
    T: Display + TagTrait,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key)?;
        if !self.value.is_null() {
            write!(f, "=>{}", self.value)?;
        }
        Ok(())
    }
}

/////////////////////////////////////////////////////////////////

/// Used to convert many types to `Tag`.
pub struct FromLiteral<T>(pub T);

#[allow(clippy::wrong_self_convention)]
impl FromLiteral<bool> {
    /// Converts the wrapped `bool` into a Tag.
    pub const fn to_tag(self) -> Tag {
        Tag::bool_(self.0)
    }
}

#[allow(clippy::wrong_self_convention)]
impl FromLiteral<&'static str> {
    /// Converts the wrapped `&'static str` into a Tag.
    pub const fn to_tag(self) -> Tag {
        Tag::str(self.0)
    }
}

#[allow(clippy::wrong_self_convention)]
impl FromLiteral<RStr<'static>> {
    /// Converts the wrapped `RStr<'static>` into a Tag.
    pub const fn to_tag(self) -> Tag {
        Tag::rstr(self.0)
    }
}

#[allow(clippy::wrong_self_convention)]
impl FromLiteral<i64> {
    /// Converts the wrapped `i64` into a Tag.
    pub const fn to_tag(self) -> Tag {
        Tag::int(self.0)
    }
}

#[allow(clippy::wrong_self_convention)]
impl FromLiteral<Tag> {
    /// Converts the wrapped `Tag` into a Tag.
    pub const fn to_tag(self) -> Tag {
        self.0
    }
}

/////////////////////////////////////////////////////////////////

fn display_iter<I>(iter: I, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result
where
    I: IntoIterator,
    I::Item: Display + TagTrait,
{
    let mut buffer = String::new();
    for elem in iter.into_iter().filter(|x| !x.is_null()) {
        Display::fmt(&buffer.display_pad(indent, &elem)?, f)?;
        writeln!(f, ",")?;
    }
    Ok(())
}

impl Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Primitive::Null => {
                write!(f, "null")?;
            }
            Primitive::Bool(cond) => {
                write!(f, "{}", cond)?;
            }
            Primitive::Int(num) => {
                write!(f, "{}", num)?;
            }
            Primitive::UInt(num) => {
                write!(f, "{}", num)?;
            }
            Primitive::String_(s) => {
                write!(f, "'{}'", s)?;
            }
        }
        Ok(())
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.variant {
            TagVariant::Primitive(prim) => {
                Display::fmt(prim, f)?;
            }
            TagVariant::Ignored(ignored) => {
                Display::fmt(ignored, f)?;
            }
            TagVariant::Array(arr) => {
                writeln!(f, "[")?;
                display_iter(&**arr, f, 4)?;
                write!(f, "]")?;
            }
            TagVariant::Set(map) => {
                writeln!(f, "{{")?;
                display_iter(map.iter(), f, 4)?;
                write!(f, "}}")?;
            }
            TagVariant::Map(map) => {
                writeln!(f, "{{")?;
                display_iter(map.iter(), f, 4)?;
                write!(f, "}}")?;
            }
        }
        Ok(())
    }
}

impl Display for CheckableTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.variant {
            CTVariant::Primitive(prim) => {
                Display::fmt(prim, f)?;
            }
            CTVariant::Ignored(ignored) => {
                Display::fmt(ignored, f)?;
            }
            CTVariant::Array(arr) => {
                writeln!(f, "[")?;
                display_iter(arr, f, 4)?;
                write!(f, "]")?;
            }
            CTVariant::Set(map) | CTVariant::Map(map) => {
                writeln!(f, "{{")?;
                display_iter(map.iter(), f, 4)?;
                write!(f, "}}")?;
            }
        }
        Ok(())
    }
}

/////////////////////////////////////////////////////////////////

/////////////////////////////////////////////////////////////////

/// The error produced when checking `CheckableTag`s.
#[derive(Debug, Clone, PartialEq)]
pub struct TagErrors {
    expected: CheckableTag,
    found: CheckableTag,
    backtrace: RVec<CheckableTag>,
    errors: RVec<TagErrorVariant>,
}

impl TagErrors {
    fn context(mut self, current: CheckableTag) -> Self {
        self.backtrace.push(current);
        self
    }
}

impl Display for TagErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = String::new();

        writeln!(f, "Stacktrace:")?;
        if self.backtrace.is_empty() {
            writeln!(f, "    Empty.")?;
        } else {
            for stack in self.backtrace.iter().rev() {
                writeln!(f, "    Inside:\n{},", buffer.display_pad(8, stack)?)?;
            }
        }
        writeln!(f, "Expected:\n{}", buffer.display_pad(4, &self.expected)?)?;
        writeln!(f, "Found:\n{}", buffer.display_pad(4, &self.found)?)?;
        writeln!(f, "Errors:\n")?;
        for err in self.errors.iter().rev() {
            writeln!(f, "\n{},", buffer.display_pad(4, err)?)?;
        }
        Ok(())
    }
}

impl std::error::Error for TagErrors {}

/////////////////////////////////////////////////////////////////

unsafe impl ExtraChecks for Tag {
    fn type_layout(&self) -> &'static TypeLayout {
        Self::LAYOUT
    }

    fn check_compatibility(
        &self,
        _layout_containing_self: &'static TypeLayout,
        layout_containing_other: &'static TypeLayout,
        checker: TypeCheckerMut<'_>,
    ) -> RResult<(), ExtraChecksError> {
        Self::downcast_with_layout(layout_containing_other, checker, |other, _| {
            let t_tag = self.to_checkable();
            let o_tag = other.to_checkable();
            t_tag.check_compatible(&o_tag)
        })
    }

    fn nested_type_layouts(&self) -> RCowSlice<'_, &'static TypeLayout> {
        RCowSlice::from_slice(&[])
    }
}

/////////////////////////////////////////////////////////////////

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, StableAbi)]
pub(crate) enum TagErrorVariant {
    MismatchedDiscriminant,
    MismatchedValue,
    MismatchedArrayLength {
        expected: usize,
        found: usize,
    },
    MismatchedAssocLength {
        expected: usize,
        found: usize,
    },
    MissingSetValue {
        expected: CheckableTag,
        found: ROption<CheckableTag>,
    },
    MismatchedMapEntry {
        expected: KeyValue<CheckableTag>,
        found: ROption<KeyValue<CheckableTag>>,
    },
}

impl Display for TagErrorVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TagErrorVariant::MismatchedDiscriminant => {
                writeln!(f, "Mismatched Tag variant.")?;
            }
            TagErrorVariant::MismatchedValue => {
                writeln!(f, "Mitmatched Value.")?;
            }
            TagErrorVariant::MismatchedArrayLength { expected, found } => {
                writeln!(
                    f,
                    "Mismatched length  expected:{}  found:{}",
                    expected, found
                )?;
            }
            TagErrorVariant::MismatchedAssocLength { expected, found } => {
                writeln!(
                    f,
                    "Mismatched length  expected at least:{}  found:{}",
                    expected, found,
                )?;
            }
            TagErrorVariant::MissingSetValue { expected, found } => {
                let mut buffer = String::new();
                writeln!(
                    f,
                    "Mismatched value in set\nExpected:\n{}",
                    buffer.display_pad(4, &expected)?
                )?;
                match found {
                    RSome(found) => writeln!(f, "Found:\n{}", buffer.display_pad(4, &found)?),
                    RNone => writeln!(f, "Found:\n    Nothing",),
                }?;
            }
            TagErrorVariant::MismatchedMapEntry { expected, found } => {
                let mut buffer = String::new();
                writeln!(
                    f,
                    "Mismatched entry in map\nExpected:\n{}",
                    buffer.display_pad(4, &expected)?
                )?;
                match found {
                    RSome(found) => writeln!(f, "Found:\n{}", buffer.display_pad(4, &found)?),
                    RNone => writeln!(f, "Found:\n    Nothing",),
                }?;
            }
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(all(
    test,
    not(feature = "only_new_tests"),
    not(feature = "no_fn_promotion")
))]
mod test;
