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
//!
//! # For contributors: adding groupers
//!
//! Adding a grouper is deliberately simple and straightforward. To add a string grouper that you
//! expose to the user in the [groupby] binary, plan to spend around two hours. The steps are as
//! follows:
//!
//! 1. Add a [matcher], following the examples of the existing matchers. This will often be
//!    trivial. Remember to add documentation and tests, preferably as doctests. (You might find
//!    that you can reuse an existing matcher, but these cases are probably rare.)
//!
//! 1. Add a corresponding grouper, following the examples of the existing groupers. For String
//!    groupers, add your method to [Groupers]. (At time of writing, there are only
//!    String groupers. If you're adding the the first non-String grouper, please exercise your
//!    best judgement in designing the module and update the documentation here accordingly.)
//!    Remember to add documentation and tests, preferably as doctests.
//!
//! 1. For String groupers, you'll probably want to expand the command-line application. (If not,
//!    please justify this decision in your pull request.) To add your grouper:
//!
//!    1. Add a new command-line option as a method in [CommandBuilder]. Add your method to
//!       [groupers()] and [group_groupers()]. Remember to add documentation and update
//!       unit tests. Your new option won't do anything, yet. (If you try to use it, it should
//!       panic, saying, "No grouping option was specified, but the argument parser didn't catch
//!       the issue. Please report this!")
//!
//!    1. Add an appropriate option to [GroupingSpecifier]. Remember to document it! The compiler will
//!       point out the locations you need to modify to support the new option. Check the existing
//!       unit tests and add new tests as appropriate.
//!
//!    1. For sanity's sake, once all tests pass and you think you're done, make sure the
//!       command-line option actually works, e.g. `cargo run -- <options>`.
//!
//! 1. Update any other relevant documentation, code, or tests, using your best judgement.
//!
//! [args]: mod@crate::command_line::args
//! [CommandBuilder]: crate::command_line::args::CommandBuilder
//! [group_groupers()]: crate::command_line::args::CommandBuilder::group_groupers
//! [groupby]: https://github.com/edev/groupby/tree/master/src/bin/groupby.rs
//! [GroupByOptions]: crate::command_line::options::GroupByOptions
//! [Groupers]: string::Groupers
//! [groupers()]: crate::command_line::args::CommandBuilder::groupers
//! [GroupingSpecifier]: crate::command_line::options::GroupingSpecifier
//! [Runner]: string::Runner

pub mod string;
