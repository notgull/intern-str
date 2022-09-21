#![cfg(feature = "builder")]

use intern_str::builder::{Builder, Utf8Graph};
use intern_str::{Graph, Node};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Color {
    Red,
    Gray,
    Green,
    Black,
    Blue,
    Beige,
}

// First, let's test a manually-constructed graph.
const NODES: &[Node<'static, &'static str, Option<Color>>] = &[
    // Default trap node.
    Node::new(&[], None, 0, core::usize::MAX),
    // Origin node.
    Node::new(&[("B", 4), ("G", 3), ("R", 2)], None, 0, 1),
    // Node for "R".
    Node::new(&[("ed", 5)], None, 0, 2),
    // Node for "G"
    Node::new(&[("r", 6)], None, 0, 1),
    // Node for "B"
    Node::new(&[("e", 8), ("l", 7)], None, 0, 1),
    // Node for "Red"
    Node::new(&[], Some(Color::Red), 0, 1),
    // Node for "Gr"
    Node::new(&[("ay", 9), ("ee", 10)], None, 0, 2),
    // Node for "Bl"
    Node::new(&[("ac", 11), ("ue", 12)], None, 0, 2),
    // Node for "Be",
    Node::new(&[("ige", 13)], None, 0, 3),
    // Node for "Gray"
    Node::new(&[], Some(Color::Gray), 0, 1),
    // Node for "Gree"
    Node::new(&[("n", 14)], None, 0, 1),
    // Node for "Blac"
    Node::new(&[("k", 15)], None, 0, 1),
    // Node for "Blue"
    Node::new(&[], Some(Color::Blue), 0, 1),
    // Node for "Beige"
    Node::new(&[], Some(Color::Beige), 0, 1),
    // Node for "Green"
    Node::new(&[], Some(Color::Green), 0, 1),
    // Node for "Black"
    Node::new(&[], Some(Color::Black), 0, 1),
];

const GRAPH: Graph<'static, 'static, &'static str, Option<Color>> = Graph::new(NODES, 1);

#[test]
fn smoke() {
    assert_eq!(*GRAPH.process("Red"), Some(Color::Red));
    assert_eq!(*GRAPH.process("Gray"), Some(Color::Gray));
    assert_eq!(*GRAPH.process("Green"), Some(Color::Green));
    assert_eq!(*GRAPH.process("Black"), Some(Color::Black));
    assert_eq!(*GRAPH.process("Blue"), Some(Color::Blue));
    assert_eq!(*GRAPH.process("Beige"), Some(Color::Beige));
    assert_eq!(*GRAPH.process("Redish"), None);
    assert_eq!(*GRAPH.process("Re"), None);
    assert_eq!(*GRAPH.process(""), None);
    assert_eq!(*GRAPH.process("Indigo"), None);
}

#[test]
fn builder() {
    extern crate alloc;

    // Begin building a graph.
    let mut builder = Builder::<Color, Utf8Graph>::new();
    builder.add("Red".to_string(), Color::Red).unwrap();
    builder.add("Gray".to_string(), Color::Gray).unwrap();
    builder.add("Green".to_string(), Color::Green).unwrap();
    builder.add("Black".to_string(), Color::Black).unwrap();
    builder.add("Blue".to_string(), Color::Blue).unwrap();
    builder.add("Beige".to_string(), Color::Beige).unwrap();

    // Finish the graph.
    let mut buffer = alloc::vec![];
    let graph = builder.build(&mut buffer);

    // Test the graph.
    assert_eq!(*graph.process("Red"), Some(Color::Red));
    assert_eq!(*graph.process("Gray"), Some(Color::Gray));
    assert_eq!(*graph.process("Green"), Some(Color::Green));
    assert_eq!(*graph.process("Black"), Some(Color::Black));
    assert_eq!(*graph.process("Blue"), Some(Color::Blue));
    assert_eq!(*graph.process("Beige"), Some(Color::Beige));
    assert_eq!(*graph.process("Redish"), None);
    assert_eq!(*graph.process("Re"), None);
    assert_eq!(*graph.process(""), None);
    assert_eq!(*graph.process("Indigo"), None);
}
