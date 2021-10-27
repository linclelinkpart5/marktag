
use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct Opts {
    pub(crate) source_dir: PathBuf,
    #[clap(long)]
    pub(crate) album_block_file: Option<PathBuf>,
    #[clap(long)]
    pub(crate) track_blocks_file: Option<PathBuf>,
    #[clap(long)]
    pub(crate) emit_existing: bool,
    #[clap(long)]
    pub(crate) emit_existing_to: Option<PathBuf>,
    #[clap(long)]
    pub(crate) output_dir: Option<PathBuf>,
}
