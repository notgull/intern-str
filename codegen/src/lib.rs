//! Convert a `Graph` into its Rust code equivalent.
//!
//! This allows one to generate a `Graph` at build time and then insert it
//! into the binary. This allows large graphs to be used in embedded systems
//! with next to no cost.
//!
//! ## Example
//!
//! ```no_run
//! use intern_str::{Graph, Segmentable};
//! use intern_str::builder::{Builder, Utf8Graph};
//! use intern_str_codegen::generate;
//! use std::{fs::File, io::{prelude::*, BufWriter}};
//!
//! # fn main() -> std::io::Result<()> {
//! let mut builder = Builder::<_, Utf8Graph>::new();
//!
//! builder.add("hello", 1).unwrap();
//! builder.add("world", 2).unwrap();
//!
//! let mut buffer = Vec::new();
//! let graph = builder.build(&mut buffer);
//!
//! // Convert to string.
//! let code = generate(
//!     &graph,
//!     "&'static str",
//!     "usize",
//!     |f, out| write!(f, "{}", out),
//! );
//!
//! let mut out = BufWriter::new(File::create("graph.rs").unwrap());
//! writeln!(
//!     out,
//!     "const GRAPH: intern_str::Graph<'static, 'static, &'static str, usize> = {}",
//!     code,
//! )?;
//! # Ok(()) }
//! ```

#![no_std]
#![forbid(
    unsafe_code,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    future_incompatible,
    rust_2018_idioms
)]

extern crate alloc;

use alloc::string::String;
use core::fmt::{self, Write};
use core::{write, writeln};

use intern_str::{CaseInsensitive, Graph, Segmentable};

/// The whole point.
///
/// See the crate documentation for more information.
pub fn generate<Input: Key, Output>(
    graph: &Graph<'_, '_, Input, Output>,
    input_type: &str,
    output_type: &str,
    mut write_output: impl FnMut(&mut dyn Write, &Output) -> fmt::Result,
) -> String {
    let mut out = String::new();

    writeln!(out, "{{").ok();

    // Write the nodes.
    writeln!(
        out,
        "{}const NODES: &[intern_str::Node<'static, {}, {}>] = &[",
        Indent(4),
        input_type,
        output_type
    )
    .ok();

    for node in graph.nodes().iter() {
        writeln!(out, "{}intern_str::Node::new(", Indent(8)).ok();

        writeln!(out, "{}&[", Indent(12)).ok();

        for (input, next) in node.inputs() {
            writeln!(
                out,
                "{}({}, {}),",
                Indent(16),
                WriteKey(input),
                next
            )
            .ok();
        }

        writeln!(out, "{}],", Indent(12)).ok();

        write!(out, "{}", Indent(12)).ok();
        write_output(&mut out, node.output()).ok();
        writeln!(out, ",").ok();

        writeln!(out, "{}{},", Indent(12), node.default(),).ok();

        writeln!(out, "{}{},", Indent(12), Index(node.amount()),).ok();

        writeln!(out, "{}),", Indent(8)).ok();
    }

    writeln!(out, "{}];", Indent(4)).ok();

    // Write the graph.
    writeln!(
        out,
        "{}const GRAPH: intern_str::Graph<'static, 'static, {}, {}> = intern_str::Graph::new(NODES, {});",
        Indent(4),
        input_type,
        output_type,
        graph.start(),
    ).ok();

    writeln!(out, "{}GRAPH", Indent(4)).ok();

    writeln!(out, "}}").ok();

    out
}

/// An item that can be used as a key.
pub trait Key: Segmentable {
    /// Format the key as a Rust expression.
    fn format(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl<'a> Key for &'a str {
    fn format(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self)
    }
}

impl<'a, T: fmt::Debug + Ord> Key for &'a [T] {
    fn format(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "&[")?;

        for (i, item) in self.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }

            write!(f, "{:?}", item)?;
        }

        write!(f, "]")
    }
}

impl<T: AsRef<[u8]> + Key> Key for CaseInsensitive<T> {
    fn format(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "intern_str::CaseInsensitive({})", WriteKey(&self.0))
    }
}

struct WriteKey<'a, T>(&'a T);

impl<'a, T: Key> fmt::Display for WriteKey<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.format(f)
    }
}

struct Indent(usize);

impl fmt::Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.0 {
            write!(f, " ")?;
        }

        Ok(())
    }
}

struct Index(usize);

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == core::usize::MAX {
            f.write_str("core::usize::MAX")
        } else {
            fmt::Display::fmt(&self.0, f)
        }
    }
}
