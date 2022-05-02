//! Provides a data structure for storing parsed command-line options.
//!
//! It serves as an abstraction layer separating the command-line option parser from the rest
//! of the system.
//!
//! The root data structure is [GroupByOptions]. The other types here are contained within
//! GroupByOptions.

use regex::Regex;

// TODO Derive Copy, Clone. Others?

/// Specifies what character to use as a separator between records/tokens.
///
/// This may be used in multiple contexts, e.g. parsing inputs and printing results.
pub enum Separator {
    /// Use a newline character (`\n`) as a separator.
    Line,

    /// Use a space (` `) as a separator.
    Space,

    /// Use a null separator (`\0`).
    Null,

    // TODO Consider adding Custom(String)
}

/// Options for handling program input.
pub struct InputOptions {
    /// Specifies what type of separator to look for when parsing records.
    pub separator: Separator,
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
pub struct OutputOptions {
    /// Specifies what type of separator to output between records.
    pub separator: Separator,

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
