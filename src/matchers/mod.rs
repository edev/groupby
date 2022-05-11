//! Functions for assigning values to equivalence classes, e.g. matching parts of strings.
//!
//! Each submodule below provides **matchers**. A matcher is a function that takes an input value
//! and possibly some additional parameters and returns a key by which the input value may be
//! grouped with other values. For instance, a matcher to divide `isize` values into equivalence
//! classes Even and Odd might look like:
//!
//! ```
//! fn match_by_evenness(value: isize) -> bool {
//!     value % 2 == 0
//! }
//!
//! assert_eq!(match_by_evenness(35), false);
//! assert_eq!(match_by_evenness(18), true);
//! ```
//!
//! The organization of this module and submodules parallels that of [groupers](crate::groupers).

pub mod string;
