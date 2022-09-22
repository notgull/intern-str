//! Benchmarks comparing matching on string to `intern-str`.

#[path = "../utils/phf.rs"]
mod phf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn compare(c: &mut Criterion) {
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

    for line in file.lines() {
        // Intern lines into the builder.
        let mut line = line.unwrap();
        if line.ends_with('\n') {
            line.pop();
        }

        if !line.is_ascii() {
            continue;
        }

        // Eat duplicates.
        builder.add(line, ()).ok();
    }

    // Build the graph.
    let mut buffer = vec![];
    let graph = builder.build(&mut buffer);

    // Use a seeded RNG to ensure that the same words are used for each benchmark.
    let rng = fastrand::Rng::with_seed(0xD3ADB33F);

    // Sample a handful of random words.
    let graph_len = graph.nodes().len();
    let mut test_words = vec![];
    while test_words.len() < 10_000 {
        let index = rng.usize(..graph_len);
        let node = &graph.nodes()[index];

        test_words.extend(node.inputs().iter().map(|(input, _)| input.0));
    }
    let test_words_len = test_words.len();

    c.bench_function("intern_str::Graph::process", |b| {
        b.iter(|| {
            // Get a random word.
            let word = &test_words[rng.usize(..test_words_len)];
            black_box(graph.process(black_box(CaseInsensitive(word))))
        })
    });

    c.bench_function("phf::Map::get", |b| {
        b.iter(|| {
            // Get a random word.
            let word = &test_words[rng.usize(..test_words_len)];
            let word = word.to_lowercase();
            black_box(phf::MAP.get(black_box(&word)))
        });
    });
}

criterion_group! {
    compare_methods,
    compare,
}

criterion_main!(compare_methods);
