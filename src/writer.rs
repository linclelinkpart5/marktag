use std::fs::File;
use std::io::Write;
use std::path::Path;

use metaflac::{BlockType, Tag};

use crate::{
    helpers::Track,
    metadata::{MetaBlock, MetaBlockList, Metadata},
};

/// Helper function to write the block data that was used for input to files.
/// The files are written to a given output directory path.
pub(crate) fn write_block_files(
    output_dir: &Path,
    album_block: &MetaBlock,
    track_blocks: &MetaBlockList,
) {
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

/// Helper method to write the combined metadata file into the final output
/// directory, alongside the newly-tagged tracks.
pub(crate) fn write_output_metadata_file(output_dir: &Path, metadata: &Metadata) {
    let metadata_fp = output_dir.join("meta.json");
    let serialized = serde_json::to_string_pretty(metadata).unwrap();
    let mut file = File::create(metadata_fp).unwrap();
    writeln!(&mut file, "{}", &serialized).unwrap();
}

pub(crate) fn write_meta_blocks_to_tag(
    track: &Track,
    total_num_tracks: usize,
    new_album_block: &MetaBlock,
    track_block: MetaBlock,
) -> Tag {
    println!("Writing new tags to file: {}", track.path.display());
    let mut flac_tag = Tag::read_from_path(&track.path).unwrap();

    // Remove all tags and pictures.
    flac_tag.remove_blocks(BlockType::VorbisComment);
    flac_tag.remove_blocks(BlockType::Picture);

    // Add in album block fields.
    for (k, v) in new_album_block {
        flac_tag.set_vorbis(k.clone(), v.as_slice().to_vec());
    }

    // Add in track block fields.
    for (k, v) in track_block {
        flac_tag.set_vorbis(k, v.into_vec());
    }

    // Add track index/count fields.
    flac_tag.set_vorbis(String::from("tracknumber"), vec![track.index.to_string()]);
    flac_tag.set_vorbis(
        String::from("totaltracks"),
        vec![total_num_tracks.to_string()],
    );

    flac_tag.save().unwrap();

    flac_tag
}
