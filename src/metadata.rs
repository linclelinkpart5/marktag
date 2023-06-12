use std::collections::BTreeMap;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Represents a metadata value. Metadata values can be either a bare string,
/// or a list of strings.
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(untagged)]
pub enum MetaVal {
    One(String),
    Many(Vec<String>),
}

impl MetaVal {
    pub fn as_slice(&self) -> &[String] {
        match self {
            Self::One(v) => core::slice::from_ref(v),
            Self::Many(vs) => vs.as_slice(),
        }
    }

    pub fn into_vec(self) -> Vec<String> {
        match self {
            Self::One(v) => vec![v],
            Self::Many(vs) => vs,
        }
    }
}

impl Display for MetaVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::One(v) => write!(f, "{}", v),
            Self::Many(vs) => {
                // TODO: Use `iter_intersperse` instead once it is stabilized.
                let mut is_first = true;
                for v in vs {
                    if is_first {
                        is_first = false;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }

                Ok(())
            }
        }
    }
}

pub type MetaBlock = BTreeMap<String, MetaVal>;
pub type MetaBlockList = Vec<MetaBlock>;

/// The combined representation of an album's metadata. This includes metadata
/// about the album itself, as well as its contained tracks.
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Metadata {
    album: MetaBlock,
    tracks: MetaBlockList,
}

#[cfg(test)]
mod tests {
    use super::MetaVal::{Many, One};
    use super::*;

    use big_s::S;
    use maplit::btreemap;
    use serde_json;

    #[test]
    fn test_deserialize() {
        let serialized: &'static str = r#"
            {
                "album": {
                    "album": "Villano",
                    "albumartist": "Dani J",
                    "date": "2023-05-30",
                    "vendor": "Qobuz"
                },
                "tracks": [
                    {
                        "artist": "Dani J",
                        "title": "Villano"
                    },
                    {
                        "artist": "Dani J",
                        "title": "7 Pecados"
                    },
                    {
                        "artist": "Dani J",
                        "title": "Caprichito"
                    },
                    {
                        "artist": [
                            "Dani J",
                            "Caluu C."
                        ],
                        "title": "Peón"
                    },
                    {
                        "artist": "Dani J",
                        "title": "Voy a Robarte"
                    }
                ]
            }
        "#;

        let deserialized: Metadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            deserialized,
            Metadata {
                album: btreemap! {
                    S("album") => One(S("Villano")),
                    S("albumartist") => One(S("Dani J")),
                    S("date") => One(S("2023-05-30")),
                    S("vendor") => One(S("Qobuz")),
                },
                tracks: vec![
                    btreemap! {
                        S("artist") => One(S("Dani J")),
                        S("title") => One(S("Villano")),
                    },
                    btreemap! {
                        S("artist") => One(S("Dani J")),
                        S("title") => One(S("7 Pecados")),
                    },
                    btreemap! {
                        S("artist") => One(S("Dani J")),
                        S("title") => One(S("Caprichito")),
                    },
                    btreemap! {
                        S("artist") => Many(vec![S("Dani J"), S("Caluu C.")]),
                        S("title") => One(S("Peón")),
                    },
                    btreemap! {
                        S("artist") => One(S("Dani J")),
                        S("title") => One(S("Voy a Robarte")),
                    },
                ],
            }
        );
    }

    #[test]
    fn test_round_trip() {
        let metadata = Metadata {
            album: btreemap! {
                S("album") => One(S("Villano")),
                S("albumartist") => One(S("Dani J")),
                S("date") => One(S("2023-05-30")),
                S("vendor") => One(S("Qobuz")),
            },
            tracks: vec![
                btreemap! {
                    S("artist") => One(S("Dani J")),
                    S("title") => One(S("Villano")),
                },
                btreemap! {
                    S("artist") => One(S("Dani J")),
                    S("title") => One(S("7 Pecados")),
                },
                btreemap! {
                    S("artist") => One(S("Dani J")),
                    S("title") => One(S("Caprichito")),
                },
                btreemap! {
                    S("artist") => Many(vec![S("Dani J"), S("Caluu C.")]),
                    S("title") => One(S("Peón")),
                },
                btreemap! {
                    S("artist") => One(S("Dani J")),
                    S("title") => One(S("Voy a Robarte")),
                },
            ],
        };

        let serialized = serde_json::to_string(&metadata).unwrap();

        let deserialized: Metadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, metadata);
    }

    #[test]
    fn test_display() {
        let meta_val = MetaVal::One(S("VALUE"));
        assert_eq!(meta_val.to_string(), "VALUE");

        let meta_val = MetaVal::Many(vec![S("VALUE_A"), S("VALUE_B"), S("VALUE_C")]);
        assert_eq!(meta_val.to_string(), "VALUE_A, VALUE_B, VALUE_C");
    }
}
