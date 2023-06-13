use std::fs::File;
use std::io::Write;
use std::path::Path;

use metaflac::{BlockType, Tag};

use crate::{
    helpers::Track,
    metadata::{MetaBlock, Metadata},
};

/// Helper method to write the combined metadata file into the final output
/// directory, alongside the newly-tagged tracks.
pub(crate) fn write_output_metadata_file(output_dir: &Path, metadata: &Metadata) {
    let metadata_fp = output_dir.join("meta.json");
    let serialized = serde_json::to_string_pretty(metadata).unwrap();
    let mut file = File::create(metadata_fp).unwrap();
    writeln!(&mut file, "{}", &serialized).unwrap();
}

pub(crate) fn write_tags_to_track(
    track: &Track,
    total_num_tracks: usize,
    incoming_album_block: MetaBlock,
    incoming_track_block: MetaBlock,
) {
    println!("Writing new tags to file: {}", track.path.display());
    let mut flac_tag = Tag::read_from_path(&track.path).unwrap();

    // Remove all tags and pictures.
    flac_tag.remove_blocks(BlockType::VorbisComment);
    flac_tag.remove_blocks(BlockType::Picture);

    // Add in album block fields.
    for (k, v) in incoming_album_block {
        flac_tag.set_vorbis(k, v.into_vec());
    }

    // Add in track block fields.
    for (k, v) in incoming_track_block {
        flac_tag.set_vorbis(k, v.into_vec());
    }

    // Add track index/count fields.
    flac_tag.set_vorbis(String::from("tracknumber"), vec![track.index.to_string()]);
    flac_tag.set_vorbis(
        String::from("totaltracks"),
        vec![total_num_tracks.to_string()],
    );

    flac_tag.save().unwrap();
}
