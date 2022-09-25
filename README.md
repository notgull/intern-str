# `intern-str`

`intern-str` is a library for interning strings in Rust. It allows you to build a graph that can convert strings (and byte strings) into arbitrary values. In addition, these graphs are `no_std` and can be embedded into projects at compile time.

The goal is to allow for low-cost, efficient string interning that does not require an allocator or the standard library. The intended use case is [MIME Types], but it is likely to be useful in many other cases as well.

[MIME Types]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types

`intern-str` is `no_std`, no `extern crate alloc`, `forbid(unsafe_code)`, and has no dependencies aside from the Rust `core` library.

## Implementation

`intern-str` generates a DFA consisting of all possible options for a given set of strings. This DFA is then used to convert a string into a value. The DFA is generated at compile time, and is embedded into the binary. This allows for efficient string interning without the need for an allocator or the standard library.

In many cases, this approach is significantly faster than converting the string to lowercase and matching on it. When matching on `/usr/share/dict/words`, `intern-str` usually completes queries in 50 ns.

The main advantage of this approach is that it can be used to create case-insensitive matching, which is significantly more difficult to do with other libraries like [`phf`]. When compared against [`phf`] when [`phf`] has to convert strings to lower case before running, `intern-str` is usually as fast as [`phf`] if not faster.

[`phf`]: https://crates.io/crates/phf

## MSRV

The current Minimum Safe Rust Version (MSRV) is Rust 1.31.0. Any change in the MSRV will lead to a minor version bump at minimum.

## License

`intern_str` is licensed under one of the following:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.