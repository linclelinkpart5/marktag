use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use metaflac::Tag;

use crate::helpers::{self, Track};
use crate::metadata::{MetaBlock, MetaBlockList, MetaVal, Metadata};

const SKIPPED_TAGS: &[&str] = &[
    "album",
    "albumartist",
    "comment",
    "copyright",
    "date",
    "description",
    "discnumber",
    "disctotal",
    "encoder",
    "genre",
    "replaygain_album_gain",
    "replaygain_album_peak",
    "replaygain_album_range",
    "replaygain_algorithm",
    "replaygain_reference_loudness",
    "replaygain_track_gain",
    "replaygain_track_peak",
    "replaygain_track_range",
    "totaltracks",
    "tracknumber",
    "tracktotal",
    "year",
];

pub(crate) fn load_metadata(path: &Path) -> Metadata {
    println!("Loading incoming metadata file: {}", path.display());

    let contents = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&contents).unwrap()
}

pub(crate) fn load_split_metadata(album_path: &Path, track_path: &Path) -> Metadata {
    println!(
        "Loading incoming metadata files (album, track): ({}, {})",
        album_path.display(),
        track_path.display(),
    );

    let contents = std::fs::read_to_string(album_path).unwrap();
    let album_block: MetaBlock = serde_json::from_str(&contents).unwrap();

    let contents = std::fs::read_to_string(track_path).unwrap();
    let track_blocks: MetaBlockList = serde_json::from_str(&contents).unwrap();

    Metadata {
        album: album_block,
        tracks: track_blocks,
    }
}

pub(crate) fn emit_existing_tags<'a>(
    tags: impl Iterator<Item = &'a Tag>,
    emit_stdout: bool,
    emit_fp: Option<&Path>,
) {
    let mut pe_blocks = Vec::new();
    let mut count = 0usize;

    for tag in tags {
        count += 1;

        let mut pe_block = MetaBlock::new();

        let keys = tag.vorbis_comments().unwrap().comments.keys();

        for key in keys {
            let key = key.to_ascii_lowercase();
            if !SKIPPED_TAGS.contains(&key.as_str()) {
                tag.get_vorbis(&key).map(|v| {
                    let mut vals = v.map(String::from).collect::<Vec<_>>();

                    let meta_val = if vals.len() == 1 {
                        MetaVal::One(vals.swap_remove(0))
                    } else {
                        MetaVal::Many(vals)
                    };

                    pe_block.insert(key, meta_val);
                });
            }
        }

        pe_blocks.push(pe_block);
    }

    // Serialize existing blocks to a string.
    let json_str = serde_json::to_string_pretty(&pe_blocks).unwrap();

    if emit_stdout {
        println!(
            "Emitting existing tags for {} input file(s) below this line...",
            count
        );
        println!("----------------------------------------------------------------");
        println!("{}", json_str);
        println!("");
        println!("----------------------------------------------------------------");
    }

    // Emit the existing blocks to a file, if provided.
    emit_fp.map(|fp| std::fs::write(fp, &json_str).unwrap());

    // Pause for user input.
    helpers::pause();
}

pub(crate) fn collect_tracks(
    source_dir: &Path,
    emit_existing: bool,
    emit_existing_to: Option<PathBuf>,
) -> Vec<Track> {
    let track_paths = source_dir
        .read_dir()
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension() == Some(OsStr::new("flac")))
        .collect::<Vec<_>>();

    let mut expected_track_nums = (1..=track_paths.len()).collect::<HashSet<_>>();

    let mut tracks = Vec::with_capacity(track_paths.len());

    for track_path in track_paths {
        println!("Found input file: {}", track_path.display());
        let track_tag = Tag::read_from_path(&track_path).unwrap();

        let track_num_str = helpers::expect_one(track_tag.get_vorbis("tracknumber").unwrap());
        let track_num = track_num_str.parse::<usize>().unwrap();

        assert!(
            expected_track_nums.remove(&track_num),
            "unexpected track number"
        );

        let track = Track {
            index: track_num,
            path: track_path,
            // tag: track_tag,
        };

        tracks.push(track);
    }

    // Ensure that all expected track numbers were covered.
    assert!(expected_track_nums.is_empty());

    // Sort the tracks by track number.
    tracks.sort_by_key(|t| t.index);

    // Emit existing tags, if requested.
    if emit_existing || emit_existing_to.is_some() {
        // emit_existing_tags(
        //     tracks.iter().map(|t| &t.tag),
        //     emit_existing,
        //     emit_existing_to,
        // );
    }

    tracks
}
