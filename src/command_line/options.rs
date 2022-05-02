//! Provides a data structure for storing parsed command-line options.
//!
//! It serves as an abstraction layer separating the command-line option parser from the rest
//! of the system.
//!
//! The root data structure is [GroupByOptions]. The other types here are contained within
//! GroupByOptions.

use regex::Regex;

// TODO Derive Copy, Clone. Others?

/// Options for handling program input.
pub struct InputOptions {
    /// If `true`, split the input on whitespace rather than on line breaks. In other
    /// words, group words (including surrounding punctuation, etc.) instead of lines.
    pub split_on_whitespace: bool,
}

/// Specifies the user's chosen grouper.
pub enum GroupingSpecifier {
    /// Group by the first `usize` characters of each token.
    FirstChars(usize),

    /// Group by the last `usize` characters of each token.
    LastChars(usize),

    /// Group by the provided regular expression. See [crate::matchers::match_regex] for details.
    Regex(Regex),
}

/// Options for controlling the program's output.
// TODO Make a separators enum to specify line, null, or space. Do the same for input.
pub struct OutputOptions {
    /// Output a null character (`\0`) rather than a newline (`\n`) to separate entries.
    /// This is intended for use with `xargs -0`.
    pub null_separators: bool,

    /// Output a space character (` `) rather than a newline (`\n`) to separate entries.
    pub space_separators: bool,

    /// Output only group names; do not group contents.
    pub only_group_names: bool,

    /// If `Some`, pass each group to the command string as its stdin instead of printing
    /// the group's contents. Instead, print any output from the command under the
    /// group's header.
    // TODO Improve the documentation on this feature, e.g. shell invocation and
    // interaction with only_group_names.
    pub run_command: Option<String>,
}

/// The main options struct that holds all other options.
///
/// Each field in this struct is a category of options.
///
/// Note: for safety, users are strongly recommended to own such a struct immutably.
pub struct GroupByOptions {
    pub input: InputOptions,
    pub grouping: GroupingSpecifier,
    pub output: OutputOptions,
}
