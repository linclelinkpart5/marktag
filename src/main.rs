mod helpers;
mod metadata;
mod opts;
mod reader;
mod writer;

use std::path::Path;

use clap::Parser;
use metaflac::block::BlockType;
use metaflac::Tag;

use crate::helpers::Track;
use crate::metadata::{MetaBlock, MetaBlockList};
use crate::opts::Opts;

fn process_tracks(
    tracks: Vec<Track>,
    album_block: MetaBlock,
    track_blocks: MetaBlockList,
    output_dir: &Path,
) {
    // Ensure equal numbers of tracks and track blocks.
    assert_eq!(tracks.len(), track_blocks.len());

    let total_tracks = tracks.len();
    let num_digits = format!("{}", total_tracks).len();

    {
        let temp_dir = tempfile::tempdir().expect("unable to create temp dir");
        let temp_dir_path = temp_dir.path();

        println!("Created temp dir: {}", temp_dir_path.display());

        for (track, track_block) in tracks.into_iter().zip(track_blocks) {
            println!("Processing input file: {}", track.path.display());
            let mut flac_tag = Tag::read_from_path(&track.path).unwrap();

            // Remove all tags and pictures.
            flac_tag.remove_blocks(BlockType::VorbisComment);
            flac_tag.remove_blocks(BlockType::Picture);

            // Add in album block fields.
            for (k, v) in &album_block {
                flac_tag.set_vorbis(k.clone(), v.as_slice().to_vec());
            }

            // Add in track block fields.
            for (k, v) in track_block {
                flac_tag.set_vorbis(k, v.into_vec());
            }

            // Add track index/count fields.
            flac_tag.set_vorbis(String::from("tracknumber"), vec![track.index.to_string()]);
            flac_tag.set_vorbis(String::from("totaltracks"), vec![total_tracks.to_string()]);

            flac_tag.save().unwrap();

            // Create temporary interim file path.
            let tno = format!("{:01$}", track.index, num_digits);

            let ars = flac_tag
                .get_vorbis("artist")
                .unwrap()
                .collect::<Vec<_>>()
                .join(", ");

            let ttl = helpers::expect_one(flac_tag.get_vorbis("title").unwrap());

            let ext = track.path.extension().unwrap().to_string_lossy();

            let mut interim_file_name = format!("{}. {} - {}.{}", tno, ars, ttl, ext);

            // Fixing bug with fields that have path separators embedded in them.
            interim_file_name.retain(|c| c != '/');

            let interim_path = temp_dir_path.join(&interim_file_name);

            println!("Moving file to temp dir: {}", interim_file_name);
            std::fs::rename(&track.path, &interim_path).unwrap();
        }

        println!("Running bs1770gain");
        helpers::calculate_gain(&output_dir, &temp_dir_path);
    }
}

fn main() {
    let opts = Opts::parse();

    let tracks =
        reader::collect_tracks(&opts.source_dir, opts.emit_existing, opts.emit_existing_to);

    let source_dir = opts.source_dir;

    let album_block_file = opts
        .album_block_file
        .unwrap_or_else(|| source_dir.join("album.json"));
    let track_blocks_file = opts
        .track_blocks_file
        .unwrap_or_else(|| source_dir.join("track.json"));

    // If no output directory is given, use the source directory.
    let output_dir = opts.output_dir.unwrap_or(source_dir);

    let album_block = reader::load_album_block(&album_block_file);
    let track_blocks = reader::load_track_blocks(&track_blocks_file);

    // Write out the input blocks to the output directory.
    writer::write_block_files(&output_dir, &album_block, &track_blocks);

    process_tracks(tracks, album_block, track_blocks, &output_dir);
}
