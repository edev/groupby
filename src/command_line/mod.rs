//! Toolkit for building command-line applications that use this library.

pub mod args;
pub mod options;
pub mod output_results;
pub mod parse_args;
pub mod process_input;
pub mod run_command;

pub use args::{args, command};
pub use options::*;
pub use parse_args::parse;
