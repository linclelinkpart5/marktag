
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};

use clap::Clap;
use metaflac::Tag;
use metaflac::block::VorbisComment;
use serde::Deserialize;

#[derive(Debug, Clap)]
struct Opts {
    source_dir: PathBuf,
    album_block_file: PathBuf,
    track_blocks_file: PathBuf,
    #[clap(long)]
    emit_existing: bool,
    #[clap(long)]
    output_dir: Option<PathBuf>,
}

struct Entry {
    path: PathBuf,
    track_num: usize,
    block: VorbisComment,
}

#[derive(Deserialize)]
#[serde(from = "BlockRepr")]
struct Block(HashMap<String, Vec<String>>);

#[derive(Deserialize)]
#[serde(untagged)]
enum BlockReprVal {
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

#[derive(Deserialize)]
#[serde(transparent)]
struct BlockRepr(HashMap<String, BlockReprVal>);

impl From<BlockRepr> for Block {
    fn from(br: BlockRepr) -> Block {
        Block(
            br.0.into_iter()
            .map(|(k, v)| (k, v.into_many()))
            .collect()
        )
    }
}

fn expect_one<T>(it: impl Iterator<Item = T>) -> T {
    let mut it = it.into_iter();
    let first = it.next();
    let second = it.next();

    match (first, second) {
        (Some(e), None) => e,
        _ => panic!("did not find exactly one value"),
    }
}

fn collect_entries(source_dir: &Path) -> Vec<Entry> {
    let flac_files =
        source_dir
        .read_dir()
        .unwrap()
        .map(Result::unwrap)
        .map(|e| e.path())
        .filter(|p| p.extension() == Some(OsStr::new("flac")))
        .collect::<Vec<_>>()
    ;

    let mut expected_track_nums = (1..=flac_files.len()).collect::<HashSet<_>>();
    let mut entries = Vec::with_capacity(flac_files.len());

    for flac_file in flac_files {
        println!("{}", flac_file.display());
        let flac_tag = Tag::read_from_path(&flac_file).unwrap();

        let track_num_str = expect_one(flac_tag.get_vorbis("tracknumber").unwrap());
        let track_num = track_num_str.parse::<usize>().unwrap();

        println!("Track #{}", track_num);

        assert!(expected_track_nums.remove(&track_num), "unexpected track number");

        let block = flac_tag.vorbis_comments().cloned().unwrap();

        let entry = Entry {
            path: flac_file,
            track_num,
            block,
        };

        entries.push(entry);
    }

    // Ensure that all expected track numbers were covered.
    assert!(expected_track_nums.is_empty());

    // Sort the entries by track number.
    entries.sort_by_key(|e| e.track_num);

    entries
}

fn load_album_block(path: &Path) -> Block {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    serde_json::from_str(&contents).unwrap()
}

fn load_track_blocks(path: &Path) -> Vec<Block> {
    Vec::new()
}

fn main() {
    let opts = Opts::parse();
    println!("{:?}", opts);

    collect_entries(&opts.source_dir);
}
