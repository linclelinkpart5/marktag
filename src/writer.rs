use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::metadata::{MetaBlock, MetaBlockList};

/// Helper function to write the block data that was used for input to files.
/// The files are written to a given output directory path.
fn write_block_files(output_dir: &Path, album_block: &MetaBlock, track_blocks: &MetaBlockList) {
    // Write out the album block, appending a newline at the end.
    let album_block_path = output_dir.join("album.json");
    let serialized = serde_json::to_string_pretty(album_block).unwrap();
    let mut file = File::create(album_block_path).unwrap();
    writeln!(&mut file, "{}", &serialized).unwrap();

    // Write out the track blocks, appending a newline at the end.
    let track_blocks_path = output_dir.join("track.json");
    let serialized = serde_json::to_string_pretty(track_blocks).unwrap();
    let mut file = File::create(track_blocks_path).unwrap();
    writeln!(&mut file, "{}", &serialized).unwrap();
}
