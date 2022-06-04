use crate::PluginId;

use std::{
    fmt::{self, Display},
    str::FromStr,
};

use arrayvec::ArrayVec;

use abi_stable::{
    std_types::{cow::BorrowingRCowStr, RCowStr, RString, RVec},
    StableAbi,
};

use core_extensions::{SelfOps, StringExt};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A way to choose to which plugins one refers to when sending commands,and other operations.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub enum WhichPlugin {
    Id(PluginId),
    First { named: RCowStr<'static> },
    Last { named: RCowStr<'static> },
    Every { named: RCowStr<'static> },
    Many(RVec<WhichPlugin>),
}

impl WhichPlugin {
    /// Converts this `WhichPlugin` to its json representation,
    /// generally used as a key in a json object.
    pub fn to_key(&self) -> RString {
        let mut buffer = RString::new();
        self.write_key(&mut buffer);
        buffer
    }

    /// Writes the value of this as a key usable in the application config.
    pub fn write_key(&self, buf: &mut RString) {
        use std::fmt::Write;
        match self {
            WhichPlugin::Id(id) => write!(buf, "{}:{}", id.named, id.instance).drop_(),
            WhichPlugin::First { named } => write!(buf, "{}:first", named).drop_(),
            WhichPlugin::Last { named } => write!(buf, "{}:last", named).drop_(),
            WhichPlugin::Every { named } => write!(buf, "{}:every", named).drop_(),
            WhichPlugin::Many(list) => {
                for elem in list {
                    elem.write_key(buf);
                    buf.push(',');
                }
            }
        }
    }
}

impl FromStr for WhichPlugin {
    type Err = WhichPluginError;

    fn from_str(full_str: &str) -> Result<Self, WhichPluginError> {
        let mut comma_sep = full_str.split(',').peekable();
        let first = comma_sep
            .next()
            .unwrap_or("")
            .piped(|s| Self::parse_single(s, full_str))?;

        if comma_sep.peek().is_some() {
            let mut list: RVec<WhichPlugin> = vec![first].into();
            for s in comma_sep.filter(|s| !s.is_empty()) {
                list.push(Self::parse_single(s, full_str)?);
            }
            WhichPlugin::Many(list)
        } else {
            first
        }
        .piped(Ok)
    }
}

impl WhichPlugin {
    fn parse_single(s: &str, full_str: &str) -> Result<Self, WhichPluginError> {
        let splitted = s
            .splitn(2, ':')
            .map(|s| s.trim())
            .collect::<ArrayVec<&str, 2>>();
        let named = splitted
            .get(0)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| WhichPluginError(full_str.into()))?
            .to_string()
            .into_::<RCowStr<'static>>();
        let selector = splitted.get(1).map_or("", |x| *x);

        match selector {
            "first" => return Ok(WhichPlugin::First { named }),
            "" | "last" => return Ok(WhichPlugin::Last { named }),
            "all" | "every" => return Ok(WhichPlugin::Every { named }),
            _ => (),
        }

        let instance = selector
            .parse::<u64>()
            .map_err(|_| WhichPluginError(full_str.into()))?;
        Ok(WhichPlugin::Id(PluginId { named, instance }))
    }

    pub const FMT_MSG: &'static str = r##"

"plugin name":
    refers to the last plugin named "plugin name".

"plugin name:10":
    refers to the 10th instance of the plugin named "plugin name".

"plugin name:first":
    refers to the first instance of the plugin named "plugin name".

"plugin name:last":
    refers to the last instance of the plugin named "plugin name".

"plugin name:every":
    refers to all the instances of the plugin named "plugin name".

"plugin name 1,plugin name 2:first,plugin name 3:every":
    refers to the last instance of the plugin named "plugin name 1".
    refers to the first instance of the plugin named "plugin name 2".
    refers to the all the instances of the plugin named "plugin name 3".

Plugin names:

- Are trimmed,so you can add spaces at the start and the end.

- Cannot contain commas,since they will be interpreted as a list of plugins.


    "##;
}

impl<'de> Deserialize<'de> for WhichPlugin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de;
        BorrowingRCowStr::deserialize(deserializer)?
            .cow
            .parse::<Self>()
            .map_err(de::Error::custom)
    }
}

impl Serialize for WhichPlugin {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_key().serialize(serializer)
    }
}

///////////////////////////////////////

#[repr(transparent)]
#[derive(Debug, Clone, StableAbi)]
pub struct WhichPluginError(RString);

impl Display for WhichPluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Could not parse this as a `WhichPlugin`:\n\t'{}'\nExpected format:\n{}\n",
            self.0,
            WhichPlugin::FMT_MSG.left_padder(4),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_str_expected() -> Vec<(&'static str, WhichPlugin)> {
        vec![
            (
                "plugin name",
                WhichPlugin::Last {
                    named: "plugin name".into(),
                },
            ),
            (
                "plugin name:10",
                WhichPlugin::Id(PluginId {
                    named: "plugin name".into(),
                    instance: 10,
                }),
            ),
            (
                "plugin name:first",
                WhichPlugin::First {
                    named: "plugin name".into(),
                },
            ),
            (
                "plugin name:last",
                WhichPlugin::Last {
                    named: "plugin name".into(),
                },
            ),
            (
                "plugin name:every",
                WhichPlugin::Every {
                    named: "plugin name".into(),
                },
            ),
            (
                "plugin name 1,plugin name 2:first,plugin name 3:every",
                WhichPlugin::Many(
                    vec![
                        WhichPlugin::Last {
                            named: "plugin name 1".into(),
                        },
                        WhichPlugin::First {
                            named: "plugin name 2".into(),
                        },
                        WhichPlugin::Every {
                            named: "plugin name 3".into(),
                        },
                    ]
                    .into(),
                ),
            ),
        ]
    }

    #[test]
    fn parses_correctly() {
        let str_expected = new_str_expected();

        for (str_, expected) in str_expected {
            let parsed = str_.parse::<WhichPlugin>().unwrap();
            assert_eq!(parsed, expected);

            assert_eq!(parsed.to_key().parse::<WhichPlugin>().unwrap(), expected,);
        }
    }

    #[test]
    fn serde_() {
        let str_expected = new_str_expected();

        for (_, elem) in str_expected {
            let str_ = serde_json::to_string(&elem).unwrap();
            let other: WhichPlugin =
                serde_json::from_str(&str_).unwrap_or_else(|e| panic!("{}", e));
            assert_eq!(other, elem);
        }
    }

    #[test]
    fn parses_incorrectly() {
        let list = vec![
            // An empty plugin name is invalid
            "",
            ":",
            ":first",
            ":last",
            ",",
            ",,,:first,:last",
        ];

        for str_ in list {
            str_.parse::<WhichPlugin>().unwrap_err();
        }
    }
}
