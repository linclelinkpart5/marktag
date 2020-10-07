
use std::path::PathBuf;

use clap::Clap;

#[derive(Debug, Clap)]
pub(crate) struct Opts {
    pub(crate) source_dir: PathBuf,
    #[clap(long, default_value = "/home/mark/album.json")]
    pub(crate) album_block_file: PathBuf,
    #[clap(long, default_value = "/home/mark/track.json")]
    pub(crate) track_blocks_file: PathBuf,
    #[clap(long)]
    pub(crate) emit_existing: bool,
    #[clap(long)]
    pub(crate) output_dir: Option<PathBuf>,
}
