//! Simple, fast and allocation-free string interning.
//!
//! `intern-str` is a library for interning strings in Rust. It allows
//! you to build a graph that can convert strings (and byte strings) 
//! into arbitrary values. In addition, these graphs are `no_std` and 
//! can be embedded into projects at compile time.
//! 
//! The goal is to allow for low-cost, efficient string interning that
//!  does not require an allocator or the standard library. The intended 
//! use case is [MIME Types], but it is likely to be useful in many 
//! other cases as well.
//! 
//! [MIME Types]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types
//! 
//! `intern-str` is `no_std`, no `extern crate alloc`, 
//! `forbid(unsafe_code)`, and has no dependencies aside from the Rust
//! `core` library.
//! 
//! To generate `intern-str` graphs as code, see the [`builder`] module
//! and the [`intern-str-codegen`] crate.
//! 
//! [`intern-str-codegen`]: https://crates.io/crates/intern-str-codegen
//! 
//! ## Implementation
//! 
//! `intern-str` generates a DFA consisting of all possible options for 
//! a given set of strings. This DFA is then used to convert a string 
//! into a value. The DFA is generated at compile time, and is embedded 
//! into the binary. This allows for efficient string interning without 
//! the need for an allocator or the standard library.
//! 
//! In many cases, this approach is significantly faster than converting 
//! the string to lowercase and matching on it. When matching on 
//! `/usr/share/dict/words`, `intern-str` usually completes queries in
//! 50 ns.
//! 
//! The main advantage of this approach is that it can be used to create 
//! case-insensitive matching, which is significantly more difficult to 
//! do with other libraries like [`phf`]. When compared against [`phf`] 
//! when [`phf`] has to convert strings to lower case before running, 
//! `intern-str` is usually as fast as [`phf`] if not faster.
//! 
//! [`phf`]: https://crates.io/crates/phf

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

#[cfg(all(feature = "builder", not(intern_str_no_alloc)))]
extern crate alloc;
#[cfg(all(feature = "builder", intern_str_no_alloc))]
extern crate std as alloc;

#[cfg(feature = "std")]
extern crate std;

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

impl<'inst, Input, Output> Node<'inst, Input, Output> {
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
}

impl<'inst, Input: Segmentable, Output> Node<'inst, Input, Output> {
    /// Determine the next index to go to based on the input.
    fn next(&self, input: &Input) -> usize {
        // Use a binary search, since the input is sorted.
        match self.inputs.binary_search_by(|(i, _)| i.cmp(input)) {
            Ok(i) => self.inputs[i].1,
            Err(_) => self.default,
        }
    }

    /// Get the inputs of this node.
    pub fn inputs(&self) -> &[(Input, usize)] {
        match &self.inputs {
            MaybeSlice::Slice(s) => s,
            #[cfg(feature = "builder")]
            MaybeSlice::Vec(v) => v,
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

impl<'nodes, 'inst, Input, Output> Graph<'inst, 'nodes, Input, Output> {
    /// Create a new graph from a set of nodes.
    pub const fn new(nodes: &'nodes [Node<'inst, Input, Output>], start: usize) -> Self {
        Self { nodes, start }
    }
}

impl<'nodes, 'inst, Input: Segmentable, Output> Graph<'inst, 'nodes, Input, Output> {
    /// Get the nodes of this graph.
    pub fn nodes(&self) -> &'nodes [Node<'inst, Input, Output>] {
        self.nodes
    }

    /// Get the start node index.
    pub fn start(&self) -> usize {
        self.start
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

    /// Get the length of the item.
    fn len(&self) -> usize;

    /// Tell if the item is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> Segmentable for &'a str {
    fn split(self, at: usize) -> Option<(Self, Self)> {
        if at > self.len() {
            return None;
        }

        let (left, right) = self.split_at(at);
        Some((left, right))
    }

    fn len(&self) -> usize {
        str::len(self)
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

    fn len(&self) -> usize {
        <[T]>::len(self)
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
        CaseInsensitive(value)
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

    fn len(&self) -> usize {
        T::len(&self.0)
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
            MaybeSlice::Slice(slice) => slice,
            #[cfg(feature = "builder")]
            MaybeSlice::Vec(vec) => vec,
        }
    }
}
