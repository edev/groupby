//! Toolkit for building command-line applications that use this library.
//!
//! This module re-exports the types and functions that form the top-level module API. For basic
//! use, simply import this module, e.g. `use groupby::command_line`.
//!
//! # Examples
//!
//! The [groupby] binary offers a simple map of the top-level API for this module. If you need
//! finer control or want to reuse specific components within the top-level methods here, this map
//! should provide jumping-off points for further reading.
//!
//! # Architecture
//!
//! This module is divided into two primary stages: processing input and outputting results. The
//! first stage populates a [GroupedCollection] based on user input. The second stage takes such a
//! collection and processes it for output, e.g. running commands over it (if requested) and
//! outputting final results. Broadly speaking, each stage involves the following steps:
//!
//! **Processing input:**
//!
//! 1. [args()]: Generates a [clap] Command.
//!
//! 1. [parse()]: Parse command-line arguments using the Command from the previous step. Generate
//!    a [GroupByOptions] value that stores command-line options in a parser-agnostic way.
//!
//! 1. [process_input()]: process input through the selected [String grouper] using [Runner],
//!    adding each token into a [GroupedCollection].
//!
//! **Outputting results:**
//!
//! 1. If [GroupByOptions] requests to run a command against each group, call [run()]
//!    once for each group. Write each group to the standard input for its command (following the
//!    options specified in [GroupByOptions::output]). Record each command's standard output using
//!    [run_command::report].
//!
//! 1. Print either the command results from the previous step or the contents of the
//!    [GroupedCollection]. In the latter case, follow the options specified in
//!    [GroupByOptions::output].
//!
//! [clap]: https://crates.io/crates/clap
//! [groupby]: https://github.com/edev/groupby/tree/master/src/bin/groupby.rs
//! [GroupedCollection]: crate::grouped_collections::GroupedCollection
//! [run()]: run_command::run()
//! [Runner]: crate::groupers::string::Runner
//! [String grouper]: crate::groupers::string::Groupers

pub mod args;
pub mod options;
pub mod output_results;
pub mod parse_args;
pub mod process_input;
pub mod record_writer;
pub mod run_command;

pub use args::{args, command};
pub use options::*;
pub use output_results::output_results;
pub use parse_args::parse;
pub use process_input::process_input;
pub use record_writer::RecordWriter;
