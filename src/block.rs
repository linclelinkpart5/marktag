
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub type Block = BTreeMap<String, Vec<String>>;
pub type BlockList = Vec<Block>;

#[derive(Clone, Deserialize, Serialize)]
#[serde(from = "BlockRepr")]
#[serde(into = "BlockRepr")]
pub struct BlockWrapper(pub Block);

#[derive(Clone, Deserialize, Serialize)]
#[serde(from = "BlockListRepr")]
#[serde(into = "BlockListRepr")]
pub struct BlockListWrapper(pub BlockList);

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum BlockReprVal {
    One(String),
    Many(Vec<String>),
}

impl BlockReprVal {
    fn into_many(self) -> Vec<String> {
        match self {
            Self::One(s) => vec![s],
            Self::Many(v) => v,
        }
    }
}

pub type BlockRepr = BTreeMap<String, BlockReprVal>;

impl From<BlockRepr> for BlockWrapper {
    fn from(br: BlockRepr) -> BlockWrapper {
        BlockWrapper(
            br.into_iter()
            .map(|(k, v)| (k, v.into_many()))
            .collect()
        )
    }
}

impl From<BlockWrapper> for BlockRepr {
    fn from(bw: BlockWrapper) -> BlockRepr {
        bw.0.into_iter()
        .map(|(k, v)| {
            let mut v = v;
            let br_val =
                if v.len() == 1 { BlockReprVal::One(v.swap_remove(0)) }
                else { BlockReprVal::Many(v) }
            ;

            (k, br_val)
        })
        .collect()
    }
}

pub type BlockListRepr = Vec<BlockRepr>;

impl From<BlockListRepr> for BlockListWrapper {
    fn from(blr: BlockListRepr) -> BlockListWrapper {
        let mut blr = blr;
        BlockListWrapper(
            blr.drain(..)
            .map(|br| { BlockWrapper::from(br).0 })
            .collect()
        )
    }
}

impl From<BlockListWrapper> for BlockListRepr {
    fn from(blw: BlockListWrapper) -> BlockListRepr {
        let mut blw = blw;

        blw.0.drain(..)
        .map(|b| BlockRepr::from(BlockWrapper(b)))
        .collect()
    }
}
