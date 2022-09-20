//! Simple, fast and allocation-free string interning.

#![no_std]
#![forbid(
    unsafe_code,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    future_incompatible,
    rust_2018_idioms
)]

#[cfg(feature = "builder")]
pub mod builder;

#[cfg(feature = "builder")]
extern crate alloc;

#[cfg(feature = "builder")]
use alloc::vec::Vec;

use core::{cmp, hash, ops};

/// A node in a DFA.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node<'inst, Input, Output> {
    /// The slice of values that this node accepts, combined with the index of the
    /// next node.
    ///
    /// The slice is sorted by the input value.
    inputs: MaybeSlice<'inst, (Input, usize)>,

    /// The output resulting from the DFA halting on this node.
    output: Output,

    /// The index of the default node to go to if no input matches.
    default: usize,

    /// The "slice" of the input that we need to match on.
    amount: usize,
}

/// A deterministic finite automaton (DFA) that can be used to process sequential
/// input to produce an output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Graph<'inst, 'nodes, Input, Output> {
    /// The nodes in the graph.
    nodes: &'nodes [Node<'inst, Input, Output>],

    /// The index of the start node.
    start: usize,
}

impl<'inst, Input: Segmentable, Output> Node<'inst, Input, Output> {
    /// Create a new node from its parts.
    pub const fn new(
        inputs: &'inst [(Input, usize)],
        output: Output,
        default: usize,
        amount: usize,
    ) -> Self {
        Self {
            inputs: MaybeSlice::Slice(inputs),
            output,
            default,
            amount,
        }
    }

    /// Determine the next index to go to based on the input.
    fn next(&self, input: &Input) -> usize {
        // Use a binary search, since the input is sorted.
        match self.inputs.binary_search_by(|(i, _)| i.cmp(input)) {
            Ok(i) => self.inputs[i].1,
            Err(_) => self.default,
        }
    }

    /// Get the inputs of this node.
    pub fn inputs(&self) -> &'inst [(Input, usize)] {
        match &self.inputs {
            MaybeSlice::Slice(s) => s,
            #[cfg(feature = "builder")]
            MaybeSlice::Vec(_) => panic!("Cannot get reference to inputs"),
        }
    }

    /// Get the output of this node.
    pub fn output(&self) -> &Output {
        &self.output
    }

    /// Get the default node index.
    pub fn default(&self) -> usize {
        self.default
    }

    /// Get the amount of input to match on.
    pub fn amount(&self) -> usize {
        self.amount
    }
}

impl<'nodes, 'inst, Input: Segmentable, Output> Graph<'inst, 'nodes, Input, Output> {
    /// Create a new graph from a set of nodes.
    pub const fn new(nodes: &'nodes [Node<'inst, Input, Output>], start: usize) -> Self {
        Self { nodes, start }
    }

    /// Process the input and return the output.
    pub fn process(&self, mut input: Input) -> &Output {
        let mut node = &self.nodes[self.start];

        // Process the input in chunks.
        loop {
            // Get the next input chunk.
            let (chunk, rest) = match input.split(node.amount) {
                Some(result) => result,
                None => {
                    // Return the value of the current node.
                    return &node.output;
                }
            };

            // Get the next node.
            node = &self.nodes[node.next(&chunk)];
            input = rest;
        }
    }
}

/// An item that can be segmented into parts.
pub trait Segmentable: Ord + Sized {
    /// Split the item into two parts.
    fn split(self, at: usize) -> Option<(Self, Self)>;
}

impl<'a> Segmentable for &'a str {
    fn split(self, at: usize) -> Option<(Self, Self)> {
        if at > self.len() {
            return None;
        }

        let (left, right) = self.split_at(at);
        Some((left, right))
    }
}

impl<'a, T: Ord> Segmentable for &'a [T] {
    fn split(self, at: usize) -> Option<(Self, Self)> {
        if at > self.len() {
            return None;
        }

        let (left, right) = self.split_at(at);
        Some((left, right))
    }
}

/// The wrapper type for a string that is compared case-insensitively.
///
/// The inner string is implied to be ASCII.
#[derive(Debug, Clone, Copy, Default)]
pub struct CaseInsensitive<T>(pub T);

impl<T> ops::Deref for CaseInsensitive<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> ops::DerefMut for CaseInsensitive<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> From<T> for CaseInsensitive<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: AsRef<[u8]>> PartialEq for CaseInsensitive<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref().eq_ignore_ascii_case(other.0.as_ref())
    }
}

impl<T: AsRef<[u8]>> Eq for CaseInsensitive<T> {}

impl<T: AsRef<[u8]>> PartialOrd for CaseInsensitive<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: AsRef<[u8]>> Ord for CaseInsensitive<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let this = self.0.as_ref();
        let other = other.0.as_ref();
        let common_len = cmp::min(this.len(), other.len());

        let this_seg = &this[..common_len];
        let other_seg = &other[..common_len];

        // Compare the common segment.
        for (a, b) in this_seg.iter().zip(other_seg.iter()) {
            let a = a.to_ascii_lowercase();
            let b = b.to_ascii_lowercase();

            match a.cmp(&b) {
                cmp::Ordering::Equal => continue,
                other => return other,
            }
        }

        // Compare the lengths.
        this.len().cmp(&other.len())
    }
}

impl<T: AsRef<[u8]>> hash::Hash for CaseInsensitive<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        for byte in self.0.as_ref() {
            state.write_u8(byte.to_ascii_lowercase());
        }
    }
}

impl<T: Segmentable + AsRef<[u8]>> Segmentable for CaseInsensitive<T> {
    fn split(self, at: usize) -> Option<(Self, Self)> {
        T::split(self.0, at).map(|(left, right)| (left.into(), right.into()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MaybeSlice<'a, T> {
    Slice(&'a [T]),
    #[cfg(feature = "builder")]
    Vec(Vec<T>),
}

impl<'a, T> core::ops::Deref for MaybeSlice<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Slice(slice) => slice,
            #[cfg(feature = "builder")]
            Self::Vec(vec) => vec,
        }
    }
}