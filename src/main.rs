mod helpers;
mod loudness;
mod metadata;
mod opts;
mod reader;
mod writer;

use std::path::Path;

use clap::Parser;

use crate::helpers::Track;
use crate::metadata::Metadata;
use crate::opts::Opts;

fn process_tracks(tracks: Vec<Track>, incoming_metadata: Metadata, output_dir: &Path) {
    let Metadata {
        album: incoming_album_block,
        tracks: incoming_track_blocks,
    } = incoming_metadata;

    // Ensure equal numbers of tracks and track blocks.
    assert_eq!(tracks.len(), incoming_track_blocks.len());

    let total_tracks = tracks.len();
    let num_digits = format!("{}", total_tracks).len();

    {
        let temp_dir = tempfile::tempdir().expect("unable to create temp dir");
        let temp_dir_path = temp_dir.path();

        println!("Created temp dir: {}", temp_dir_path.display());

        for (track, incoming_track_block) in tracks.into_iter().zip(incoming_track_blocks) {
            let display_artist = incoming_track_block
                .get("artist")
                .expect("track block did not have a 'artist' key")
                .to_string();
            let display_title = incoming_track_block
                .get("title")
                .expect("track block did not have a 'title' key")
                .to_string();

            println!("Processing input file: {}", track.path.display());
            writer::write_tags_to_track(
                &track,
                total_tracks,
                incoming_album_block.clone(),
                incoming_track_block,
            );

            let ext = track.path.extension().unwrap().to_string_lossy();
            let output_track_file_name = helpers::generate_output_file_name(
                track.index,
                num_digits,
                &display_artist,
                &display_title,
                &ext,
            );

            let interim_path = temp_dir_path.join(&output_track_file_name);

            println!("Moving file to temp dir: {}", output_track_file_name);
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

    // Load the incoming metadata (the metadata the user has configured to be
    // written to the tags).
    let incoming_metadata = reader::load_split_metadata(&album_block_file, &track_blocks_file);

    // Write out the incoming metadata to the output directory.
    writer::write_output_metadata_file(&output_dir, &incoming_metadata);

    process_tracks(tracks, incoming_metadata, &output_dir);
}
