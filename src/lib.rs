//! Utilities for creating collections of values grouped by common keys.
//!
//! This crate provides utilities for grouping values by common keys. A common use case might be
//! grouping values into equivalence classes and storing these groups in either a
//! [std::collections::HashMap] or a [std::collections::BTreeMap].
//!
//! # Example: grouping
//!
//! A basic example demonstrating what grouping means and how it works:
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
//!
//! # Example: command-line processing
//!
//! A quick demonstration of the crate's command-line capabilities. Simulate a directory listing
//! of files with structured names, group the file names, and count the files in each group using
//! `wc`:
//!
//! ```
//! use groupby::command_line;
//! use groupby::command_line::options::*;
//! use std::collections::BTreeMap;
//!
//! // A complete application might parse GroupByOptions from command-line options, e.g. by calling
//! // command_line::parse(command_line::args()).
//! let options = GroupByOptions {
//!     input: InputOptions {
//!         separator: Separator::Null,
//!     },
//!     grouping: GroupingSpecifier::FirstChars(6),
//!     output: OutputOptions {
//!         separator: Separator::Line,
//!         only_group_names: false,
//!         run_command: Some("wc -l".to_string()),
//!     },
//! };
//!
//! // The GroupedCollection we'll use. HashMap is also supported but doesn't preserve group order.
//! let mut map = BTreeMap::new();
//!
//! // A complete application might use io::stdin().
//! let simulated_input = "\
//!     ecs440 class notes.tex\0\
//!     ecs450 class notes.tex\0\
//!     ecs450 study guide.pdf";
//!
//! // Split tokens by null characters, group them by class, and add them to the map.
//! command_line::build_groups(simulated_input.as_bytes(), &mut map, &options);
//!
//! // A complete application might use io::stdout().
//! let mut output = Vec::new();
//!
//! // Run `wc -l` once for each group, pass the group's contents to the group's stdin, and collect
//! // each command's stdout.
//! command_line::output_results(&mut output, &map, &options);
//!
//! assert_eq!(String::from_utf8_lossy(&output),
//! "ecs440:
//! 1
//!
//! ecs450:
//! 2\n\n");
//! ```

pub mod command_line;
pub mod grouped_collections;
pub mod groupers;
pub mod matchers;
