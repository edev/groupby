//! Utilities for creating collections of values grouped by common keys.
//!
//! This crate provides utilities for grouping values by common keys. A common use case might be
//! grouping values into equivalence classes and storing these groups in either a
//! [std::collections::HashMap] or a [std::collections::BTreeMap].
//!
//! # Examples
//!
//! ```
//! use groupby::grouped_collections::*;
//! use groupby::groupers::string::*;
//! use std::collections::BTreeMap;
//!
//! let values = ["Alphabet", "Alps", "Alpine", "Equine", "Equator", "Equivalence"]
//!     .into_iter()
//!     .map(|s| s.to_string());
//!
//! let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
//! for value in values {
//!     map.group_by_first_chars(value, 3);
//! }
//!
//! let expected_alps: Vec<String> = ["Alphabet", "Alps", "Alpine"]
//!     .into_iter()
//!     .map(|s| s.to_string())
//!     .collect();
//! assert_eq!(map.get(&"Alp".to_string()).unwrap(), &expected_alps);
//!
//! let expected_equs: Vec<String> = ["Equine", "Equator", "Equivalence"]
//!     .into_iter()
//!     .map(|s| s.to_string())
//!     .collect();
//! assert_eq!(map.get(&"Equ".to_string()).unwrap(), &expected_equs);
//! ```

// TODO Expand documentation here once groupers and matchers are more mature:
// - Follow the examples of major crates like Rand
// - Add a description of the major products: GroupedCollection, groupers, matchers...
// - Add a quickstart
// - Link to benchmarks

pub mod command_line;
pub mod grouped_collections;
pub mod groupers;
pub mod matchers;
