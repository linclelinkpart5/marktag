
use std::path::PathBuf;

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

fn main() {
    let opts = Opts::parse();
    println!("{:?}", opts);
}
