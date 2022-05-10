//! The [GroupedCollection] trait and implementations for
//! [BTreeMap](std::collections::BTreeMap) and [HashMap](std::collections::HashMap).
//!
//! If you're here, you're probably looking for the [GroupedCollection] trait, which provides a
//! common interface over different mapping data structures so that you can swap them out without
//! affecting calling code.

pub mod btree_map;
pub mod grouped_collection;
pub mod hash_map;
mod test_helpers;

pub use grouped_collection::GroupedCollection;
