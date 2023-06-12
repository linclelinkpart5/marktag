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

pub(crate) enum IncomingMetadataSource<'a> {
    Unified(Option<&'a Path>),
    AlbumTrack(Option<&'a Path>, Option<&'a Path>),
}

impl<'a> IncomingMetadataSource<'a> {
    pub fn load_metadata(&self) -> Metadata {
        match self {
            Self::Unified(p) => {
                let path = p.unwrap_or(Path::new("meta.json"));

                println!("Loading incoming metadata (unified): {}", path.display());

                let contents = std::fs::read_to_string(path).unwrap();
                serde_json::from_str(&contents).unwrap()
            }
            Self::AlbumTrack(ap, tp) => {
                let album_path = ap.unwrap_or(Path::new("album.json"));
                let track_path = tp.unwrap_or(Path::new("track.json"));

                println!(
                    "Loading incoming metadata (album, track): ({}, {})",
                    album_path.display(),
                    track_path.display()
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
        }
    }
}

pub(crate) fn emit_preexisting_tags(
    tags: impl Iterator<Item = Tag>,
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

    // Serialize preexisting blocks to a string.
    let json_str = serde_json::to_string_pretty(&pe_blocks).unwrap();

    if emit_stdout {
        println!(
            "Emitting preexisting tags for {} input file(s) below this line...",
            count
        );
        println!("----------------------------------------------------------------");
        println!("{}", json_str);
        println!("");
        println!("----------------------------------------------------------------");
    }

    // Emit the preexisting blocks to a file, if provided.
    emit_fp.map(|fp| std::fs::write(fp, &json_str).unwrap());

    // Pause for user input.
    helpers::pause();
}

pub(crate) fn collect_tracks(
    source_dir: &Path,
    emit_existing: bool,
    emit_existing_to: Option<PathBuf>,
) -> Vec<Track> {
    let flac_files = source_dir
        .read_dir()
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension() == Some(OsStr::new("flac")))
        .collect::<Vec<_>>();

    let mut expected_track_nums = (1..=flac_files.len()).collect::<HashSet<_>>();
    let mut tracks = Vec::with_capacity(flac_files.len());
    let mut tags_to_emit = None;

    if emit_existing || emit_existing_to.is_some() {
        tags_to_emit = Some(Vec::with_capacity(flac_files.len()));
    }

    for flac_file in flac_files {
        println!("Found input file: {}", flac_file.display());
        let flac_tag = Tag::read_from_path(&flac_file).unwrap();

        let track_num_str = helpers::expect_one(flac_tag.get_vorbis("tracknumber").unwrap());
        let track_num = track_num_str.parse::<usize>().unwrap();

        tags_to_emit.as_mut().map(|tte| {
            tte.push((track_num, flac_tag));
        });

        assert!(
            expected_track_nums.remove(&track_num),
            "unexpected track number"
        );

        let track = Track {
            index: track_num,
            path: flac_file,
        };

        tracks.push(track);
    }

    // Ensure that all expected track numbers were covered.
    assert!(expected_track_nums.is_empty());

    // Sort the tracks by track number.
    tracks.sort_by_key(|e| e.index);

    // Sort and emit preexisting tags, if requested.
    tags_to_emit.as_mut().map(|tte| {
        tte.sort_by_key(|(tn, _)| *tn);

        let tags = tte.drain(..).map(|(_, tag)| tag);

        emit_preexisting_tags(
            tags,
            emit_existing,
            emit_existing_to.as_ref().map(|p| p.as_path()),
        );
    });

    tracks
}

pub(crate) fn load_album_block(path: &Path) -> MetaBlock {
    println!("Loading album file: {}", path.display());
    let contents = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&contents).unwrap()
}

pub(crate) fn load_track_blocks(path: &Path) -> MetaBlockList {
    println!("Loading track file: {}", path.display());
    let contents = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&contents).unwrap()
}
