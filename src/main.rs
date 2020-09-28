
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Clap;
use metaflac::Tag;
use metaflac::block::BlockType;
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
}

type Block = HashMap<String, Vec<String>>;

#[derive(Deserialize)]
#[serde(from = "BlockRepr")]
struct BlockWrapper(Block);

#[derive(Deserialize)]
#[serde(from = "BlockListRepr")]
struct BlockListWrapper(Vec<Block>);

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

type BlockRepr = HashMap<String, BlockReprVal>;

impl From<BlockRepr> for BlockWrapper {
    fn from(br: BlockRepr) -> BlockWrapper {
        let mut br = br;
        BlockWrapper(
            br.drain()
            .map(|(k, v)| (k, v.into_many()))
            .collect()
        )
    }
}

type BlockListRepr = Vec<BlockRepr>;

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
        println!("Found input file: {}", flac_file.display());
        let flac_tag = Tag::read_from_path(&flac_file).unwrap();

        let track_num_str = expect_one(flac_tag.get_vorbis("tracknumber").unwrap());
        let track_num = track_num_str.parse::<usize>().unwrap();

        println!("Track #{}", track_num);

        assert!(expected_track_nums.remove(&track_num), "unexpected track number");

        let entry = Entry {
            path: flac_file,
            track_num,
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
    println!("Loading album block file: {}", path.display());
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    serde_json::from_str::<BlockWrapper>(&contents).unwrap().0
}

fn load_track_blocks(path: &Path) -> Vec<Block> {
    println!("Loading track blocks file: {}", path.display());
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    serde_json::from_str::<BlockListWrapper>(&contents).unwrap().0
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

        println!("Created temp dir: {}", temp_dir_path.display());

        for (entry, track_block) in entries.into_iter().zip(track_blocks) {
            println!("Processing input file: {}", entry.path.display());
            let mut flac_tag = Tag::read_from_path(&entry.path).unwrap();

            // Remove all tags and pictures.
            flac_tag.remove_blocks(BlockType::VorbisComment);
            flac_tag.remove_blocks(BlockType::Picture);

            // Add in album block fields.
            for (k, v) in &album_block {
                flac_tag.set_vorbis(k.clone(), v.clone());
            }

            // Add in track block fields.
            for (k, v) in track_block {
                flac_tag.set_vorbis(k, v);
            }

            // Add track index/count fields.
            flac_tag.set_vorbis(
                String::from("tracknumber"),
                vec![entry.track_num.to_string()],
            );
            flac_tag.set_vorbis(
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

            println!("Moving {} to temp directory", entry.path.file_name().and_then(|f| f.to_str()).unwrap());
            std::fs::rename(&entry.path, &interim_path).unwrap();
        }

        // Run `bs1770gain` as an external command.
        // This should also copy the files to their final destination.
        println!("Running bs1770gain");
        let status =
            Command::new("bs1770gain")
            .arg("--replaygain")
            .arg("-irt")
            .arg("--output")
            .arg(output_dir.as_os_str())
            .arg(temp_dir_path.as_os_str())
            .status()
            .unwrap()
        ;

        assert!(status.success());
    }
}

fn main() {
    let opts = Opts::parse();

    let entries = collect_entries(&opts.source_dir);

    // If no output directory is given, use the source directory.
    let output_dir = opts.output_dir.unwrap_or(opts.source_dir);

    let album_block = load_album_block(&opts.album_block_file);
    let track_blocks = load_track_blocks(&opts.track_blocks_file);

    process_entries(entries, album_block, track_blocks, &output_dir);
}
