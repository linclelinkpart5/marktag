
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};

use clap::Clap;

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
        .collect::<Vec<_>>()
    ;

    let mut expected_track_nums = (0..flac_files.len()).collect::<HashSet<_>>();

    for flac_file in flac_files {
    }

    unimplemented!()
}

fn main() {
    let opts = Opts::parse();
    println!("{:?}", opts);
}
