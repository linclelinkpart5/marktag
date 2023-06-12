use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use metaflac::Tag;

use crate::helpers::{self, Entry};
use crate::metadata::{MetaBlock, MetaBlockList, MetaVal};

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

pub(crate) fn emit_source_tags(
    tags: impl Iterator<Item = Tag>,
    emit_stdout: bool,
    emit_fp: Option<&Path>,
) {
    let mut src_blocks = Vec::new();
    let mut count = 0usize;

    for tag in tags {
        count += 1;

        let mut block = MetaBlock::new();

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

                    block.insert(key, meta_val);
                });
            }
        }

        src_blocks.push(block);
    }

    // Serialize source blocks to a string.
    let json_str = serde_json::to_string_pretty(&src_blocks).unwrap();

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

    // Emit the source blocks to a file, if provided.
    emit_fp.map(|fp| std::fs::write(fp, &json_str).unwrap());

    // Pause for user input.
    helpers::pause();
}

pub(crate) fn collect_entries(
    source_dir: &Path,
    emit_existing: bool,
    emit_existing_to: Option<PathBuf>,
) -> Vec<Entry> {
    let flac_files = source_dir
        .read_dir()
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension() == Some(OsStr::new("flac")))
        .collect::<Vec<_>>();

    let mut expected_track_nums = (1..=flac_files.len()).collect::<HashSet<_>>();
    let mut entries = Vec::with_capacity(flac_files.len());
    let mut emitted_tag_blocks = None;

    if emit_existing || emit_existing_to.is_some() {
        emitted_tag_blocks = Some(Vec::with_capacity(flac_files.len()));
    }

    for flac_file in flac_files {
        println!("Found input file: {}", flac_file.display());
        let flac_tag = Tag::read_from_path(&flac_file).unwrap();

        let track_num_str = helpers::expect_one(flac_tag.get_vorbis("tracknumber").unwrap());
        let track_num = track_num_str.parse::<usize>().unwrap();

        emitted_tag_blocks.as_mut().map(|etbs| {
            etbs.push((track_num, flac_tag));
        });

        assert!(
            expected_track_nums.remove(&track_num),
            "unexpected track number"
        );

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

        let tags = etbs.drain(..).map(|(_, tag)| tag);

        emit_source_tags(
            tags,
            emit_existing,
            emit_existing_to.as_ref().map(|p| p.as_path()),
        );
    });

    entries
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
