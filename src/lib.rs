//! Utilities for creating collections of values grouped by common keys.
//!
//! This crate provides utilities for grouping values by common keys. A common use case might be
//! grouping values into equivalence classes and storing these groups in either a
//! [std::collections::HashMap] or a [std::collections::BTreeMap].

// TODO Expand documentation here once groupers and matchers are more mature:
// - Follow the examples of major crates like Rand
// - Add a description of the major products: GroupedCollection, groupers, matchers...
// - Add a quickstart
// - Link to benchmarks

pub mod command_line;
pub mod grouped_collections;
pub mod groupers;
pub mod matchers;
