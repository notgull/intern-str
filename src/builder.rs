//! A builder for graphs.
//!
//! This builder is not meant to be used in library code. Therefore, it is not thread-safe,
//! and uses an allocator.

#[cfg(feature = "std")]
extern crate std;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use core::marker::PhantomData;
use core::{fmt, mem};

use __private::Sealed;

/// A builder for graphs.
#[derive(Debug, Default)]
pub struct Builder<T, Type> {
    /// The nodes in the graph.
    nodes: Vec<Node<T>>,

    /// Whether or not the graph supports UTF-8.
    ty: PhantomData<Type>,
}

impl<'a, T, Type: GraphType<'a>> Builder<T, Type> {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            ty: PhantomData,
        }
    }

    /// Add a key/value pair to the map.
    pub fn add(&mut self, key: String, value: T) -> Result<(), AddError<T>> {
        if key.is_empty() {
            return Err(AddError::Empty(value));
        }

        if !Type::validate(&key) {
            return Err(AddError::Invalid(key, value));
        }

        // The node we are inserting.
        let mut node = Node {
            value: key,
            output: Some(value),
            children: Vec::new(),
        };

        // The current set of siblings we're trying to insert a node into.
        let mut siblings = &mut self.nodes;

        loop {
            // Iterate through the potential siblings to find a shared prefix.
            let closest_node = siblings.iter_mut().enumerate().find_map(|(i, sibling)| {
                // See if we have a shared prefix.
                let prefix = prefix(&node.value, &sibling.value);

                // If we share a prefix, match on this node.
                if !prefix.is_empty() {
                    Some((i, prefix))
                } else {
                    None
                }
            });

            let (index, prefix) = match closest_node {
                Some(result) => result,
                None => {
                    // No shared prefix, so we can just add the node as a direct sibling.
                    siblings.push(node);
                    return Ok(());
                }
            };

            // If the prefix is entirely equal to the node's value, we move on to the
            // node's children.
            if prefix == siblings[index].value || prefix == node.value {
                let prefix_len = prefix.len();

                // If both the keys are equal, we have a duplicate.
                if node.value == siblings[index].value {
                    // We may be able to just insert the value.
                    if siblings[index].output.is_none() {
                        siblings[index].output = node.output;
                        return Ok(());
                    }

                    // Otherwise, we have a duplicate.
                    return Err(AddError::Duplicate(node.value, node.output.unwrap()));
                }

                // Swap the node and the sibling if necessary.
                if prefix == node.value {
                    mem::swap(&mut node, &mut siblings[index]);
                }

                siblings = &mut siblings[index].children;
                node.value = node.value[prefix_len..].to_string();

                continue;
            }

            // Remove the new sibling node from the sibling set.
            let mut sibling = siblings.swap_remove(index);

            // In our node and the sibling, remove the prefix.
            let prefix = prefix.to_string();
            node.value = node.value[prefix.len()..].to_string();
            sibling.value = sibling.value[prefix.len()..].to_string();

            // Create a new node with no result that contains the shared prefix.
            let prefix_node = Node {
                value: prefix,
                output: None,
                children: vec![sibling, node],
            };

            // Push the new node into the sibling set.
            siblings.push(prefix_node);

            return Ok(());
        }
    }

    /// Build the graph.
    pub fn build<'nodes>(
        &'a mut self,
        node_buffer: &'nodes mut Vec<super::Node<'static, Type::InputKey, Option<T>>>,
    ) -> super::Graph<'static, 'nodes, Type::InputKey, Option<T>>
    where
        T: Clone,
    {
        // Clear the node buffer.
        node_buffer.clear();

        // Sort our children.
        self.nodes.sort_unstable_by(|a, b| a.value.cmp(&b.value));

        // Recursively sort node children.
        for node in &mut self.nodes {
            node.sort();
        }

        // Add a "default" node at position zero.
        node_buffer.push(super::Node {
            inputs: crate::MaybeSlice::Slice(&[]),
            output: None,
            default: 0,
            amount: core::usize::MAX,
        });

        // Build the graph.
        let initial_indices = self
            .nodes
            .iter()
            .map(|node| {
                let index = node.build::<Type>(node_buffer);
                let value = Type::key(&node.value);
                (value, index)
            })
            .collect::<Vec<_>>();

        // Create a root node.
        let root = super::Node {
            inputs: crate::MaybeSlice::Vec(initial_indices),
            output: None,
            default: 0,
            amount: core::usize::MAX, // TODO: broken in every way,
        };
        node_buffer.push(root);

        // The last node will be our starting node.
        let end = node_buffer.len() - 1;

        super::Graph::new(&*node_buffer, end)
    }
}

/// A node in the graph.
#[derive(Debug)]
struct Node<T> {
    /// The current value associated with this node.
    value: String,

    /// The output associated with this node, if any.
    output: Option<T>,

    /// The next node to use for each possible input.
    children: Vec<Node<T>>,
}

impl<T: Clone> Node<T> {
    /// Sort this node's children.
    fn sort(&mut self) {
        // Sort the children.
        self.children.sort_by(|a, b| a.value.cmp(&b.value));

        // Do the same for all children.
        for child in &mut self.children {
            child.sort();
        }
    }

    /// Add this node and its children to the graph.
    ///
    /// Returns the index of the node in the graph.
    fn build<'a, 'nodes, Type: GraphType<'a>>(
        &'a self,
        nodes: &'nodes mut Vec<super::Node<'static, Type::InputKey, Option<T>>>,
    ) -> usize {
        // Build each child.
        let child_indices = self
            .children
            .iter()
            .map(|child| {
                let index = child.build::<Type>(nodes);
                let value = Type::key(&child.value);
                (value, index)
            })
            .collect::<Vec<_>>();

        // Now, add our node.
        let node_index = nodes.len();
        nodes.push(super::Node {
            inputs: crate::MaybeSlice::Vec(child_indices),
            output: self.output.clone(),
            default: 0,
            amount: self.value.len(),
        });

        node_index
    }
}

/// The type that a graph can have.
pub trait GraphType<'a>: Sealed {
    /// The type of the input key.
    type InputKey: super::Segmentable;

    /// Validate the input.
    fn validate(input: &str) -> bool;

    /// Convert the input into a key.
    fn key(input: &'a str) -> Self::InputKey;
}

/// A graph that supports UTF-8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Utf8Graph;

impl Sealed for Utf8Graph {}
impl<'a> GraphType<'a> for Utf8Graph {
    type InputKey = &'a str;

    fn validate(_: &str) -> bool {
        true
    }

    fn key(input: &'a str) -> Self::InputKey {
        input
    }
}

/// A graph that only supports ASCII.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AsciiGraph;

impl Sealed for AsciiGraph {}
impl<'a> GraphType<'a> for AsciiGraph {
    type InputKey = &'a [u8];

    fn validate(input: &str) -> bool {
        input.is_ascii()
    }

    fn key(input: &'a str) -> Self::InputKey {
        input.as_bytes()
    }
}

mod __private {
    #[doc(hidden)]
    pub trait Sealed {}
}

/// An error that occurs when building a graph.
#[derive(Debug)]
pub enum AddError<T> {
    /// The key is empty.
    Empty(T),

    /// The key is not valid.
    Invalid(String, T),

    /// The key is already in the graph.
    Duplicate(String, T),
}

impl<T: fmt::Display> fmt::Display for AddError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty(value) => write!(f, "Cannot add an empty key to the graph: {}", value),
            Self::Invalid(key, value) => write!(
                f,
                "Cannot add an invalid key to the graph: {} ({})",
                key, value
            ),
            Self::Duplicate(key, value) => write!(
                f,
                "Cannot add a duplicate key to the graph: {} ({})",
                key, value
            ),
        }
    }
}

#[cfg(feature = "std")]
impl<T: fmt::Debug + fmt::Display> std::error::Error for AddError<T> {}

/// Get the shared prefix for two strings.
fn prefix<'a>(a: &'a str, b: &str) -> &'a str {
    let mut i = 0;

    for (a, b) in a.chars().zip(b.chars()) {
        if a != b {
            break;
        }

        i += a.len_utf8();
    }

    &a[..i]
}