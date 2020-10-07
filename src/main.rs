
mod block;
mod opts;

use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Clap;
use metaflac::Tag;
use metaflac::block::BlockType;

use crate::block::*;
use crate::opts::Opts;

const SKIPPED_TAGS: &[&str] = &[
    "tracknumber",
    "comment",
    "totaltracks",
    "year",
    // "date",
    "albumartist",
    "album",
    "encoder",
    "replaygain_album_gain",
    "replaygain_album_peak",
    "replaygain_album_range",
    "replaygain_algorithm",
    "replaygain_reference_loudness",
    "replaygain_track_gain",
    "replaygain_track_peak",
    "replaygain_track_range",
];

struct Entry {
    path: PathBuf,
    track_num: usize,
}

fn expect_one<T, I: IntoIterator<Item = T>>(it: I) -> T {
    let mut it = it.into_iter();
    let first = it.next();
    let second = it.next();

    match (first, second) {
        (Some(e), None) => e,
        _ => panic!("did not find exactly one value"),
    }
}

fn collect_entries(source_dir: &Path, emit_existing: bool) -> Vec<Entry> {
    let flac_files =
        source_dir
        .read_dir()
        .unwrap()
        .map(Result::unwrap)
        .map(|e| e.path())
        .filter(|p| p.extension() == Some(OsStr::new("flac")))
        .collect::<Vec<_>>()
    ;

    let mut expected_track_nums = (1..=flac_files.len()).collect::<HashSet<_>>();
    let mut entries = Vec::with_capacity(flac_files.len());
    let mut emitted_tag_blocks = None;

    if emit_existing {
        emitted_tag_blocks = Some(Vec::with_capacity(flac_files.len()));
    }

    for flac_file in flac_files {
        println!("Found input file: {}", flac_file.display());
        let flac_tag = Tag::read_from_path(&flac_file).unwrap();

        let track_num_str = expect_one(flac_tag.get_vorbis("tracknumber").unwrap());
        let track_num = track_num_str.parse::<usize>().unwrap();

        println!("Track #{}", track_num);

        emitted_tag_blocks.as_mut().map(|etbs| {
            etbs.push((track_num, flac_tag));
        });

        assert!(expected_track_nums.remove(&track_num), "unexpected track number");

        let entry = Entry {
            path: flac_file,
            track_num,
        };

        entries.push(entry);
    }

    // Ensure that all expected track numbers were covered.
    assert!(expected_track_nums.is_empty());

    // Sort the entries by track number.
    entries.sort_by_key(|e| e.track_num);

    // Sort and emit source blocks, if any.
    emitted_tag_blocks.as_mut().map(|etbs| {
        etbs.sort_by_key(|(tn, _)| *tn);

        let src_blocks =
            etbs.drain(..)
            .map(|(_, tag)| {
                let mut block_repr = BlockRepr::new();

                let keys = tag.vorbis_comments().unwrap().comments.keys();

                for key in keys {
                    let key = key.to_ascii_lowercase();
                    if !SKIPPED_TAGS.contains(&key.as_str()) {
                        let lookup =
                            tag.get_vorbis(&key)
                            .map(|v| {
                                v.map(String::from)
                                .collect::<Vec<_>>()
                            })
                        ;

                        if let Some(mut vals) = lookup {
                            let block_repr_val =
                                if vals.len() == 1 { BlockReprVal::One(vals.swap_remove(0)) }
                                else { BlockReprVal::Many(vals) }
                            ;

                            block_repr.insert(key, block_repr_val);
                        }
                    }
                }

                block_repr
            })
            .collect::<BlockListRepr>()
        ;

        // Serialize source blocks and print to stdout.
        serde_json::to_writer_pretty(std::io::stdout(), &src_blocks).unwrap();
        println!("");

        // Pause for user input.
        let mut stdout = std::io::stdout();
        let mut stdin = std::io::stdin();
        write!(stdout, "Press any key to continue...").unwrap();
        stdout.flush().unwrap();
        stdin.read(&mut [0u8]).unwrap();
    });

    entries
}

fn load_album_block(path: &Path) -> Block {
    println!("Loading album block file: {}", path.display());
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    serde_json::from_str::<BlockWrapper>(&contents).unwrap().0
}

fn load_track_blocks(path: &Path) -> BlockList {
    println!("Loading track blocks file: {}", path.display());
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    serde_json::from_str::<BlockListWrapper>(&contents).unwrap().0
}

/// Helper function to write the block data that was used for input to files.
/// The files are written to a given output directory path.
fn write_block_files(output_dir: &Path, album_block: &BlockWrapper, track_blocks: &BlockListWrapper) {
    // Write out the album block, appending a newline at the end.
    let album_block_path = output_dir.join("album.json");
    let mut serialized = serde_json::to_string_pretty(album_block).unwrap();
    serialized.push('\n');
    std::fs::write(album_block_path, &serialized).unwrap();

    // Write out the track blocks, appending a newline at the end.
    let track_blocks_path = output_dir.join("track.json");
    let mut serialized = serde_json::to_string_pretty(track_blocks).unwrap();
    serialized.push('\n');
    std::fs::write(track_blocks_path, &serialized).unwrap();
}

fn process_entries(
    entries: Vec<Entry>,
    album_block: Block,
    track_blocks: BlockList,
    output_dir: &Path,
)
{
    // Ensure equal numbers of entries and track blocks.
    assert_eq!(entries.len(), track_blocks.len());

    let total_tracks = entries.len();
    let num_digits = format!("{}", total_tracks).len();

    {
        let temp_dir = tempfile::tempdir().expect("unable to create temp dir");
        let temp_dir_path = temp_dir.path();

        println!("Created temp dir: {}", temp_dir_path.display());

        for (entry, track_block) in entries.into_iter().zip(track_blocks) {
            println!("Processing input file: {}", entry.path.display());
            let mut flac_tag = Tag::read_from_path(&entry.path).unwrap();

            // Remove all tags and pictures.
            flac_tag.remove_blocks(BlockType::VorbisComment);
            flac_tag.remove_blocks(BlockType::Picture);

            // Add in album block fields.
            for (k, v) in &album_block {
                flac_tag.set_vorbis(k.clone(), v.clone());
            }

            // Add in track block fields.
            for (k, v) in track_block {
                flac_tag.set_vorbis(k, v);
            }

            // Add track index/count fields.
            flac_tag.set_vorbis(
                String::from("tracknumber"),
                vec![entry.track_num.to_string()],
            );
            flac_tag.set_vorbis(
                String::from("totaltracks"),
                vec![total_tracks.to_string()],
            );

            flac_tag.save().unwrap();

            // Create temporary interim file path.
            let tno = format!("{:01$}", entry.track_num, num_digits);

            let ars = flac_tag.get_vorbis("artist").unwrap().collect::<Vec<_>>().join(", ");

            let ttl = expect_one(flac_tag.get_vorbis("title").unwrap());

            let ext = entry.path.extension().unwrap().to_string_lossy();

            let mut interim_file_name = format!("{}. {} - {}.{}", tno, ars, ttl, ext);

            // Fixing bug with fields that have path separators embedded in them.
            interim_file_name.retain(|c| c != '/');

            let interim_path = temp_dir_path.join(interim_file_name);

            println!("Moving {} to temp directory", entry.path.file_name().and_then(|f| f.to_str()).unwrap());
            std::fs::rename(&entry.path, &interim_path).unwrap();
        }

        // Run `bs1770gain` as an external command.
        // This should also copy the files to their final destination.
        println!("Running bs1770gain");
        let status =
            Command::new("bs1770gain")
            .arg("--replaygain")
            .arg("-irt")
            .arg("--output")
            .arg(output_dir.as_os_str())
            .arg(temp_dir_path.as_os_str())
            .status()
            .unwrap()
        ;

        assert!(status.success());
    }
}

fn main() {
    let opts = Opts::parse();

    let entries = collect_entries(&opts.source_dir, opts.emit_existing);

    // If no output directory is given, use the source directory.
    let output_dir = opts.output_dir.unwrap_or(opts.source_dir);

    let album_block = load_album_block(&opts.album_block_file);
    let track_blocks = load_track_blocks(&opts.track_blocks_file);

    // Write out the input blocks to the output directory.
    // This involves wrapping up the block data just for now, for serialization
    // purposes.
    let abw = BlockWrapper(album_block);
    let tblw = BlockListWrapper(track_blocks);
    write_block_files(&output_dir, &abw, &tblw);

    // Unpackage the block data.
    let album_block = abw.0;
    let track_blocks = tblw.0;

    process_entries(entries, album_block, track_blocks, &output_dir);
}
