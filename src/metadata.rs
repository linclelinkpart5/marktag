use std::collections::BTreeMap;

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
}
