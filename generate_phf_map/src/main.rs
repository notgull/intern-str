//! Generate a PHF map for comparison against `intern-str`.

use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut cout = stdout.lock();

    write!(cout, "pub const MAP: phf::Map<&'static str, ()> = ",)?;

    // Read in lines from /usr/share/dict/words
    let mut builder = phf_codegen::Map::new();
    let words = io::BufReader::new(fs::File::open("/usr/share/dict/words")?);
    let mut existing = HashSet::new();

    for word in words.lines() {
        let line = word?;
        let word = line.trim().to_lowercase();

        if !word.is_ascii() {
            continue;
        }

        if existing.insert(word.clone()) {
            builder.entry(word, "()");
        }
    }

    // Write map to file.
    writeln!(cout, "{};", builder.build())?;

    Ok(())
}
