//! We deal with a larger graph this time: the list of words on most Unix systems.

#[cfg(all(unix, feature = "builder"))]
#[test]
fn words_list() {
    use std::fs::File;
    use std::io::{BufRead, BufReader, ErrorKind};
    use std::path::Path;

    use intern_str::builder::{Builder, IgnoreCase, Utf8Graph};
    use intern_str::CaseInsensitive;

    // Read in lines from /usr/share/dict/words
    let file = BufReader::new(match File::open(Path::new("/usr/share/dict/words")) {
        Ok(file) => file,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            // If the file is not found, we skip the test.
            return;
        }
        Err(e) => panic!("{}", e),
    });

    let mut builder = Builder::<_, IgnoreCase<Utf8Graph>>::new();
    let mut euclid_index = None;

    for (i, line) in file.lines().enumerate() {
        // Intern lines into the builder.
        let mut line = line.unwrap();
        if line.ends_with('\n') {
            line.pop();
        }

        if !line.is_ascii() {
            continue;
        }

        if line.eq_ignore_ascii_case("euclid") {
            euclid_index = Some(i);
        }

        // Eat duplicates.
        builder.add(line, i).ok();
    }

    let euclid_index = match euclid_index {
        Some(i) => i,
        None => {
            // If the file does not contain the word "euclid", we skip the test.
            return;
        }
    };

    // Build the graph.
    let mut buffer = vec![];
    let graph = builder.build(&mut buffer);

    // Check that the graph contains the word "euclid".
    assert_eq!(
        *graph.process(CaseInsensitive("euclid")),
        Some(euclid_index)
    );

    // The word "sfdlkjafldksakdfls" should not be in the graph.
    assert_eq!(*graph.process(CaseInsensitive("sfdlkjafldksakdfls")), None);
}
