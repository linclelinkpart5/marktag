
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};

use clap::Clap;
use metaflac::Tag;
use metaflac::block::VorbisComment;

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
    Block::new()
}

fn load_track_blocks(path: &Path) -> Vec<Block> {
    Vec::new()
}

fn main() {
    let opts = Opts::parse();
    println!("{:?}", opts);

    collect_entries(&opts.source_dir);
}
