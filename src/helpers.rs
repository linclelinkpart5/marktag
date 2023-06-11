use std::io::{Read, Write};

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
