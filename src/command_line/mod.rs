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
//! [groupby]: https://github.com/edev/groupby/tree/master/src/bin/groupby.rs

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
