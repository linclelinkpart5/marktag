
use std::path::{Path, PathBuf};
use std::collections::HashMap;

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
    track_num: u32,
    block: HashMap<String, Vec<String>>,
}

fn collect_entries(source_dir: &Path) -> Vec<Entry> {
    unimplemented!()
}

fn main() {
    let opts = Opts::parse();
    println!("{:?}", opts);
}
