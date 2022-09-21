//! Basic utility for converting an `intern-str` DFA into an easy-to-comprehend graph.

use intern_str::{Graph, Segmentable};
use std::fmt::{Debug, Display};
use std::io;

/// Convert a DFA into a graphviz dot file.
pub fn as_graphviz<Input: Segmentable + Display, Output: Debug>(
    graph: &Graph<'_, '_, Input, Output>,
    out: &mut impl io::Write,
    name: &str,
) -> io::Result<()> {
    writeln!(out, "digraph {} {{", name)?;

    // Write out each node.
    for (i, node) in graph.nodes().iter().enumerate() {
        writeln!(out, "s{} [label=\"{:?}\"]", i, node.output())?;

        // Write out each connection.
        for (input, next) in node.inputs() {
            writeln!(out, "s{} -> s{} [label=\"{}\"];", i, next, input)?;
        }

        writeln!(out, "s{} -> s{};", i, node.default())?;
    }

    writeln!(out, "}}")?;

    Ok(())
}
