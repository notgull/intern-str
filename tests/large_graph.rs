//! We deal with a larger graph this time: the list of words on most Unix systems.

#[cfg(unix)]
#[test]
fn words_list() {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::Path;

    use intern_str::builder::{Builder, Utf8Graph};

    // Read in lines from /usr/share/dict/words
    let mut file = BufReader::new(File::open(Path::new("/usr/share/dict/words")).unwrap());

    let mut builder = Builder::<_, Utf8Graph>::new();

    for (i, line) in file.lines().enumerate() {
        let mut line = line.unwrap();
        if line.ends_with('\n') {
            line.pop();
        }
        builder.add(line, i).unwrap();
    }
}