
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use clap::Clap;
use metaflac::Tag;
use metaflac::block::{BlockType, VorbisComment};
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

type Block = HashMap<String, Vec<String>>;

#[derive(Deserialize)]
#[serde(from = "BlockRepr")]
struct BlockWrapper(Block);

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

impl From<BlockRepr> for BlockWrapper {
    fn from(br: BlockRepr) -> BlockWrapper {
        BlockWrapper(
            br.0.into_iter()
            .map(|(k, v)| (k, v.into_many()))
            .collect()
        )
    }
}

fn expect_one<T, I: IntoIterator<Item = T>>(it: I) -> T {
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

    serde_json::from_str::<BlockWrapper>(&contents).unwrap().0
}

fn load_track_blocks(path: &Path) -> Vec<Block> {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    serde_json::from_str(&contents).unwrap()
}

fn process_entries(
    entries: Vec<Entry>,
    album_block: Block,
    track_blocks: Vec<Block>,
    output_dir: &Path,
)
{
    // Ensure equal numbers of entries and track blocks.
    assert_eq!(entries.len(), track_blocks.len());

    let total_tracks = entries.len();
    let num_digits = format!("{}", total_tracks).len();

    {
        let temp_dir = tempfile::tempdir().expect("unable to create temp dir");
        let temp_dir_path = temp_dir.path();

        for (entry, track_block) in entries.into_iter().zip(track_blocks) {
            let mut flac_tag = Tag::read_from_path(&entry.path).unwrap();

            // Remove all tags and pictures.
            flac_tag.remove_blocks(BlockType::VorbisComment);
            flac_tag.remove_blocks(BlockType::Picture);

            let comments = &mut flac_tag.vorbis_comments_mut().comments;

            // Add in album block fields.
            for (k, v) in &album_block {
                comments.insert(k.clone(), v.clone());
            }

            // Add in track block fields.
            for (k, v) in track_block {
                comments.insert(k, v);
            }

            // Add track index/count fields.
            comments.insert(
                String::from("tracknumber"),
                vec![entry.track_num.to_string()],
            );
            comments.insert(
                String::from("totaltracks"),
                vec![total_tracks.to_string()],
            );

            flac_tag.save().unwrap();

            // Create temporary interim file path.
            let tno = format!("{:01$}", entry.track_num, num_digits);

            let ars = flac_tag.get_vorbis("artist").unwrap().collect::<Vec<_>>().join(", ");

            let ttl = expect_one(flac_tag.get_vorbis("title").unwrap());

            let ext = entry.path.extension().unwrap().to_string_lossy();

            let interim_file_name = format!("{}. {} - {}.{}", tno, ars, ttl, ext);
            let interim_path = temp_dir_path.join(interim_file_name);
        }
    }
}

fn main() {
    let opts = Opts::parse();
    println!("{:?}", opts);

    let entries = collect_entries(&opts.source_dir);

    let album_block = load_album_block(&opts.album_block_file);
    let track_blocks = load_track_blocks(&opts.track_blocks_file);
}
