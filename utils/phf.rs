//! This file is intended to be populated with the following command:
//!
//! ```
//! cd generate_phf_map
//! cargo run -- > ../utils/phf.rs
//! ```
//!
//! I keep it empty because the full file is about 100,000 lines, and I'd like to
//! keep the GitHub repository relatively clean.

pub const MAP: phf::Map<&'static str, ()> = phf::Map::new();
