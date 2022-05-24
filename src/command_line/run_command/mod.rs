//! Types for running shell commands over groups of values.
//!
//! This module provides support for running a shell command over a group of values. The purpose of
//! this module is to provide tested, reusable functions and types to support such workflows.
//!
//! The [super::output_results] module is the chief internal consumer for the types and functions
//! defined here. It provides higher-level code to iterate over a [GroupedCollection], run commands
//! if needed, and ultimately output the results.
//!
//! [GroupedCollection]: crate::grouped_collections::GroupedCollection

mod child;
mod command;
pub mod handle;
#[cfg(test)]
pub mod mock_child;
#[cfg(test)]
pub mod mock_command;
pub mod record_writer;
pub mod report;
pub mod run;

pub use child::Child;
pub use command::Command;
pub use handle::Handle;
#[cfg(test)]
pub use mock_child::MockChild;
#[cfg(test)]
pub use mock_command::MockCommand;
pub use record_writer::RecordWriter;
pub use report::Report;
pub use report::ReportInteriorMutable;
pub use run::run;
