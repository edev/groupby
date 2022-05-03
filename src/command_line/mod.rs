//! Provides a toolkit for building command-line applications based on this library.

pub mod args;
pub mod options;
pub mod parse_args;

pub use args::{args, command};
pub use options::*;
pub use parse_args::parse;
