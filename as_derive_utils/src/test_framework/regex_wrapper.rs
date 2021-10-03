use std::{
    cmp::{Eq, PartialEq},
    hash::{Hash, Hasher},
    ops::Deref,
};

use core_extensions::SelfOps;
use regex::Regex;
use serde::de::{Deserialize, Deserializer, Error as deError};

#[derive(Clone, Debug)]
pub struct RegexWrapper(pub Regex);

impl From<Regex> for RegexWrapper {
    fn from(t: Regex) -> Self {
        RegexWrapper(t)
    }
}

impl Hash for RegexWrapper {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_str().hash(state)
    }
}

impl Eq for RegexWrapper {}
impl PartialEq for RegexWrapper {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}

impl Deref for RegexWrapper {
    type Target = Regex;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for RegexWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <String>::deserialize(deserializer)?
            .piped(|v| Regex::new(&v))
            .map(RegexWrapper)
            .map_err(|e| format!("{}", e))
            .map_err(D::Error::custom)
    }
}
