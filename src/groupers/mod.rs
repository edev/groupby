//! Predefined helper methods on [GroupedCollection] for adding values to groups.
//!
//! Each submodule below provides **groupers**. A grouper is a method implemented on
//! [GroupedCollection] that calls a [matcher] to determine which group should hold a given value
//! and then adds the value to that group. For instance, a trivial matcher and grouper pair might
//! look something like this:
//!
//! ```
//! // Some required boilerplate has been cut for clarity; see source or any submodule for details
//! // if you're thinking of implementing your own matchers and groupers.
//!
//! use groupby::grouped_collections::GroupedCollection;
//! use std::collections::HashMap;
//!
//! // Standalone function defined under groupby::matchers
//! fn match_whole_string(string: &str) -> &str {
//!     string
//! }
//!
//! // Implemented on GroupedCollection<'s, String, String, _>
//! # trait ExampleGroupers<List> {
//! #     fn group_by_whole_string(&mut self, string: String);
//! # }
//! #
//! # impl<'s, List, GC> ExampleGroupers<List> for GC
//! # where
//! #     List: 's,
//! #     GC: GroupedCollection<'s, String, String, List>,
//! # {
//! fn group_by_whole_string(&mut self, string: String) {
//!     let key = match_whole_string(&string).to_string();
//!     self.add(key, string);
//! }
//! # }
//!
//! let mut map = HashMap::new();
//! let line = "Banana bread".to_string();
//! map.group_by_whole_string(line.clone());
//! assert_eq!(map.get(&line).unwrap().first().unwrap(), &line);
//! ```
//!
//! The organization of this module and submodules parallels that of [matchers].
//!
//! [GroupedCollection]: crate::grouped_collections::GroupedCollection
//! [matcher]: crate::matchers
//! [matchers]: crate::matchers

pub mod string;
