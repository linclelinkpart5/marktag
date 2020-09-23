
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};

use clap::Clap;
use metaflac::Tag;

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
    block: HashMap<String, Vec<String>>,
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

    let mut expected_track_nums = (0..flac_files.len()).collect::<HashSet<_>>();

    for flac_file in flac_files {
        println!("{}", flac_file.display());
        let flac_tag = Tag::read_from_path(&flac_file).unwrap();
    }

    Vec::new()
}

fn main() {
    let opts = Opts::parse();
    println!("{:?}", opts);

    collect_entries(&opts.source_dir);
}
