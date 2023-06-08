use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Represents a metadata value. Metadata values can be either a bare string,
/// or a list of strings.
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum MetaVal {
    One(String),
    Many(Vec<String>),
}

pub type MetaBlock = BTreeMap<String, MetaVal>;
pub type MetaBlockList = Vec<MetaBlock>;

/// The combined representation of an album's metadata. This includes metadata
/// about the album itself, as well as its contained tracks.
#[derive(Deserialize, Serialize)]
pub struct Metadata {
    album: MetaBlock,
    tracks: MetaBlockList,
}
