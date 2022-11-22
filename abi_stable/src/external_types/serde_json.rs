//! Ffi-safe equivalents of `serde_json` types.

use std::{
    convert::{TryFrom, TryInto},
    fmt::{self, Debug, Display},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{error::Error as JsonError, value::RawValue};

use crate::std_types::{RStr, RString};

/// An ffi-safe equivalent of `&serde_json::value::RawValue`
///
/// # Example
///
/// This defines a function that serializes a struct,
/// and deserializes the json into another one with `RawValueRef` fields.
///
/// ```
/// use abi_stable::{
///     external_types::RawValueRef,
///     sabi_extern_fn,
///     std_types::{RBoxError, RErr, ROk, RResult, RStr, RString},
/// };
///
/// use serde::{Deserialize, Serialize};
///
/// use std::collections::HashMap;
///
/// const JSON: &'static str = r##"{"hello":"world"}"##;
///
/// let value = RawValueRef::try_from_str(JSON).unwrap();
///
/// assert_eq!(serde_json::to_string(&value).unwrap().as_str(), JSON);
///
/// #[derive(Serialize)]
/// pub struct Pair {
///     pub first: Vec<u32>,
///     pub second: HashMap<RString, RString>,
/// }
///
/// #[derive(Debug, Deserialize)]
/// pub struct PairDeserialize<'a> {
///     #[serde(borrow)]
///     pub first: RawValueRef<'a>,
///
///     #[serde(borrow)]
///     pub second: RawValueRef<'a>,
/// }
///
/// #[sabi_extern_fn]
/// fn deserialize_data_structure<'de>(
///     input: RStr<'de>,
/// ) -> RResult<PairDeserialize<'de>, RBoxError> {
///     match serde_json::from_str::<PairDeserialize>(input.into()) {
///         Ok(x) => ROk(x),
///         Err(x) => RErr(RBoxError::new(x)),
///     }
/// }
///
/// # fn main(){
///
/// let json = serde_json::to_string(&Pair {
///     first: vec![0, 1, 2],
///     second: vec![(RString::from("hello"), "world".into())]
///         .into_iter()
///         .collect(),
/// })
/// .unwrap();
///
/// let pair = deserialize_data_structure(json.as_str().into()).unwrap();
///
/// assert_eq!(pair.first.get(), "[0,1,2]");
/// assert_eq!(pair.second.get(), r##"{"hello":"world"}"##);
///
/// # }
///
/// ```
#[repr(transparent)]
#[derive(StableAbi, Copy, Clone)]
pub struct RawValueRef<'a> {
    ref_: RStr<'a>,
}

impl<'a> RawValueRef<'a> {
    /// Converts a `&str` to a `RawValueRef<'a>` without checking whether it is valid JSON.
    ///
    /// # Safety
    ///
    /// `input` must be valid JSON and contain no leading or trailing whitespace.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RawValueRef;
    ///
    /// const JSON: &'static str = r##"{"huh":"that is interesting"}"##;
    ///
    /// const VALUE: RawValueRef<'_> = unsafe { RawValueRef::from_str_unchecked(JSON) };
    ///
    /// assert_eq!(serde_json::to_string(&VALUE).unwrap().as_str(), JSON);
    ///
    /// ```
    pub const unsafe fn from_str_unchecked(input: &'a str) -> RawValueRef<'a> {
        Self {
            ref_: RStr::from_str(input),
        }
    }

    /// Converts a `RStr<'a>` to a `RawValueRef<'a>` without checking whether it is valid JSON.
    ///
    /// # Safety
    ///
    /// `input` must be valid JSON and contain no leading or trailing whitespace.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{external_types::RawValueRef, std_types::RStr};
    ///
    /// const JSON: &'static str = r##"{"huh":"that is interesting"}"##;
    ///
    /// let json_rstr = RStr::from(JSON);
    /// let value = unsafe { RawValueRef::from_rstr_unchecked(json_rstr) };
    ///
    /// assert_eq!(serde_json::to_string(&value).unwrap().as_str(), JSON);
    /// ```
    ///
    ///
    pub const unsafe fn from_rstr_unchecked(input: RStr<'a>) -> RawValueRef<'a> {
        Self { ref_: input }
    }

    /// Attempts to convert a `&'a str` into a `RawValueRef<'a>`.
    ///
    /// Fails in the same cases as parsing a `&'a RawValue` from a string does.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{external_types::RawValueRef, std_types::RStr};
    ///
    /// const JSON: &'static str = r##"{"nope":"oof"}"##;
    ///
    /// let raw = RawValueRef::try_from_str(JSON).unwrap();
    ///
    /// assert_eq!(raw.get(), JSON);
    ///
    /// ```
    #[inline]
    pub fn try_from_str(input: &'a str) -> Result<Self, JsonError> {
        input.try_into()
    }

    /// Gets the json being serialized,as a `&str`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RawValueRef;
    ///
    /// const JSON: &'static str = r##"{"huh":1007}"##;
    ///
    /// let raw = serde_json::from_str::<RawValueRef<'static>>(JSON).unwrap();
    ///
    /// assert_eq!(raw.get(), JSON);
    ///
    /// ```
    #[inline]
    pub fn get(&self) -> &'a str {
        self.ref_.as_str()
    }

    /// Gets the json being serialized,as a `RStr<'a>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{external_types::RawValueRef, std_types::RStr};
    ///
    /// const JSON: &'static str = r##"{"bugs":"life"}"##;
    ///
    /// let raw = serde_json::from_str::<RawValueRef<'static>>(JSON).unwrap();
    ///
    /// assert_eq!(raw.get_rstr(), RStr::from(JSON));
    ///
    /// ```
    #[inline]
    pub const fn get_rstr(&self) -> RStr<'a> {
        self.ref_
    }
}

impl<'a> Debug for RawValueRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.ref_, f)
    }
}

impl<'a> Display for RawValueRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.ref_, f)
    }
}

impl<'a> From<&'a RawValue> for RawValueRef<'a> {
    fn from(v: &'a RawValue) -> Self {
        Self {
            ref_: v.get().into(),
        }
    }
}

impl<'a> TryFrom<&'a str> for RawValueRef<'a> {
    type Error = JsonError;
    fn try_from(v: &'a str) -> Result<Self, JsonError> {
        serde_json::from_str::<&'a RawValue>(v).map(Self::from)
    }
}

impl<'a> Serialize for RawValueRef<'a> {
    fn serialize<Z>(&self, serializer: Z) -> Result<Z::Ok, Z::Error>
    where
        Z: Serializer,
    {
        unsafe { into_ref_rawvalue(self.ref_.as_str()).serialize(serializer) }
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for RawValueRef<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <&'a RawValue>::deserialize(deserializer).map(Self::from)
    }
}

///////////////////////////////////////////////////////////////////////////////

/// An ffi-safe equivalent of `Box<serde_json::value::RawValue>`
///
///
/// # Example
///
/// This defines a function that serializes a struct,
/// and deserializes the json into another one with `RawValueBox` fields.
///
/// ```
/// use abi_stable::{
///     external_types::RawValueBox,
///     sabi_extern_fn,
///     std_types::{RBoxError, RErr, ROk, RResult, RStr, RString},
/// };
///
/// use serde::{Deserialize, Serialize};
///
/// const JSON: &'static str = r##"{"hello":"world"}"##;
///
/// let value = RawValueBox::try_from_string(JSON.to_string()).unwrap();
///
/// assert_eq!(serde_json::to_string(&value).unwrap().as_str(), JSON);
///
/// #[derive(Serialize)]
/// pub struct Pair {
///     pub first: u64,
///     pub second: RString,
/// }
///
/// #[derive(Debug, Deserialize)]
/// pub struct PairDeserialize {
///     pub first: RawValueBox,
///     pub second: RawValueBox,
/// }
///
/// #[sabi_extern_fn]
/// fn deserialize_data_structure(
///     input: RStr<'_>,
/// ) -> RResult<PairDeserialize, RBoxError> {
///     match serde_json::from_str::<PairDeserialize>(input.into()) {
///         Ok(x) => ROk(x),
///         Err(x) => RErr(RBoxError::new(x)),
///     }
/// }
///
/// # fn main(){
///
/// let json = serde_json::to_string(&Pair {
///     first: 99,
///     second: "How many apples?".into(),
/// })
/// .unwrap();
///
/// let pair = deserialize_data_structure(json.as_str().into()).unwrap();
///
/// assert_eq!(pair.first.get(), "99");
/// assert_eq!(pair.second.get(), r##""How many apples?"}"##);
///
/// # }
///
///
/// ```
///
#[repr(transparent)]
#[derive(StableAbi, Clone)]
pub struct RawValueBox {
    string: RString,
}

impl RawValueBox {
    /// Converts a `String` to an `RawValueBox` without checking whether it is valid JSON.
    ///
    /// # Safety
    ///
    /// `input` must be valid JSON and contain no leading or trailing whitespace.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RawValueBox;
    ///
    /// const JSON: &'static str = r##"{"huh":"that is interesting"}"##;
    ///
    /// let value = unsafe { RawValueBox::from_string_unchecked(JSON.to_string()) };
    ///
    /// assert_eq!(serde_json::to_string(&value).unwrap().as_str(), JSON);
    /// ```
    ///
    #[inline]
    pub unsafe fn from_string_unchecked(input: String) -> RawValueBox {
        Self {
            string: input.into(),
        }
    }

    /// Converts an `RString` to an `RawValueBox` without checking whether it is valid JSON.
    ///
    /// # Safety
    ///
    /// `input` must be valid JSON and contain no leading or trailing whitespace.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{external_types::RawValueBox, std_types::RString};
    ///
    /// const JSON: &'static str = r##"{"huh":"that is interesting"}"##;
    ///
    /// let json_rstring = RString::from(JSON);
    /// let value = unsafe { RawValueBox::from_rstring_unchecked(json_rstring) };
    ///
    /// assert_eq!(serde_json::to_string(&value).unwrap().as_str(), JSON);
    /// ```
    ///
    #[inline]
    pub const unsafe fn from_rstring_unchecked(input: RString) -> RawValueBox {
        Self { string: input }
    }

    /// Attempts to convert a `String` into a `RawValueBox`.
    ///
    /// Fails in the same cases as converting a String into a `Box<RawValue>` does.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{external_types::RawValueBox, std_types::RString};
    ///
    /// const JSON: &'static str = r##"{"nope":"oof"}"##;
    ///
    /// let raw = RawValueBox::try_from_string(JSON.to_string()).unwrap();
    ///
    /// assert_eq!(raw.get(), JSON);
    ///
    /// ```
    #[inline]
    pub fn try_from_string(input: String) -> Result<Self, JsonError> {
        input.try_into()
    }

    /// Gets the json being serialized,as a `&str`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RawValueBox;
    ///
    /// const JSON: &'static str = r##"{"huh":1007}"##;
    ///
    /// let raw = serde_json::from_str::<RawValueBox>(JSON).unwrap();
    ///
    /// assert_eq!(raw.get(), JSON);
    ///
    /// ```
    #[inline]
    pub fn get(&self) -> &str {
        self.string.as_str()
    }

    /// Gets the json being serialized,as a `RStr<'a>`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{external_types::RawValueBox, std_types::RStr};
    ///
    /// const JSON: &'static str = r##"{"bugs":"life"}"##;
    ///
    /// let raw = serde_json::from_str::<RawValueBox>(JSON).unwrap();
    ///
    /// assert_eq!(raw.get_rstr(), RStr::from(JSON));
    ///
    /// ```
    #[inline]
    pub fn get_rstr(&self) -> RStr<'_> {
        self.get().into()
    }

    /// Gets a RawValueRef<'_> borrowing from this RawValueBox.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::{RawValueBox, RawValueRef};
    ///
    /// const JSON: &'static str = r##"{"bugs":"life"}"##;
    ///
    /// let raw = serde_json::from_str::<RawValueBox>(JSON).unwrap();
    ///
    /// assert_eq!(raw.get(), RawValueRef::try_from_str(JSON).unwrap().get());
    ///
    /// ```
    #[inline]
    pub fn as_raw_value_ref(&self) -> RawValueRef<'_> {
        unsafe { RawValueRef::from_str_unchecked(self.get()) }
    }
}

impl Debug for RawValueBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.string, f)
    }
}

impl Display for RawValueBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.string, f)
    }
}

impl From<Box<RawValue>> for RawValueBox {
    fn from(v: Box<RawValue>) -> Self {
        let string: String = from_boxed_rawvalue(v).into();
        Self {
            string: string.into(),
        }
    }
}

impl TryFrom<String> for RawValueBox {
    type Error = JsonError;
    fn try_from(v: String) -> Result<Self, JsonError> {
        RawValue::from_string(v).map(Self::from)
    }
}

impl Serialize for RawValueBox {
    fn serialize<Z>(&self, serializer: Z) -> Result<Z::Ok, Z::Error>
    where
        Z: Serializer,
    {
        unsafe { into_ref_rawvalue(&self.string).serialize(serializer) }
    }
}

impl<'de> Deserialize<'de> for RawValueBox {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <Box<RawValue>>::deserialize(deserializer).map(Self::from)
    }
}

///////////////////////////////////////////////////////////////////////////////

fn from_boxed_rawvalue(x: Box<RawValue>) -> Box<str> {
    // This would become Undefined Behavior if
    // serde_json somehow changes RawValue to not be a transparent wrapper around `str`
    unsafe { Box::from_raw(Box::into_raw(x) as *mut str) }
}

const unsafe fn into_ref_rawvalue(x: &str) -> &RawValue {
    // This would become Undefined Behavior if
    // serde_json somehow changes RawValue to not be a transparent wrapper around `str`
    unsafe { &*(x as *const str as *const RawValue) }
}
