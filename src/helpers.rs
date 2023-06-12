use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) struct Track {
    pub index: usize,
    pub path: PathBuf,
}

/// Pauses the program, and outputs a prompt for the user to
/// press Enter to continue.
pub(crate) fn pause() {
    let mut stdout = std::io::stdout();
    let mut stdin = std::io::stdin();
    write!(stdout, "Press <Enter> to continue...").unwrap();
    stdout.flush().unwrap();
    stdin.read(&mut [0u8]).unwrap();
}

/// Attempts to pull a single element from an iterator. Panics if there are
/// zero elements, or if there is more than more element.
pub(crate) fn expect_one<T, I: IntoIterator<Item = T>>(it: I) -> T {
    let mut it = it.into_iter();
    let first = it.next();
    let second = it.next();

    match (first, second) {
        (Some(e), None) => e,
        _ => panic!("did not find exactly one value"),
    }
}

pub(crate) fn calculate_gain(output_dir: &Path, temp_dir_path: &Path) {
    // Run `bs1770gain` as an external command.
    // This will/should also copy the tracks to their final destination directory.
    let status = Command::new("bs1770gain")
        .arg("--replaygain")
        .arg("-irt")
        .arg("--output")
        .arg(output_dir.as_os_str())
        .arg(temp_dir_path.as_os_str())
        .status()
        .unwrap();

    assert!(status.success());
}

/// Generates an output filename for a track once it is finished processing.
pub(crate) fn generate_output_file_name(
    track_num: usize,
    track_padding: usize,
    display_artist: &str,
    display_title: &str,
    ext: &str,
) -> String {
    let mut output_file_name = format!(
        "{:0width$}. {} - {}.{}",
        track_num,
        display_artist,
        display_title,
        ext,
        width = track_padding,
    );

    // Fixing bug with fields that have path separators embedded in them.
    output_file_name.retain(|c| c != '/');

    output_file_name
}
