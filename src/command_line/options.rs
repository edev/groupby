//! Provides a data structure for storing parsed command-line options.
//!
//! It serves as an abstraction layer separating the command-line option parser from the rest
//! of the system.
//!
//! The root data structure is [GroupByOptions]. The other types here are contained within
//! GroupByOptions.

use regex::Regex;

/// Specifies what character to use as a separator between records/tokens.
///
/// This may be used in multiple contexts, e.g. parsing inputs and printing results.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Separator {
    /// Use a newline character (`\n`) as a separator.
    Line,

    /// Use a space (` `) as a separator.
    Space,

    /// Use a null separator (`\0`).
    Null,

    /// Use a user-provided string as a separator.
    Custom(String),
}

/// Options for handling program input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputOptions {
    /// Specifies what type of separator to look for when parsing records.
    pub separator: Separator,
}

/// A named or numbered regular expression capture group.
///
/// Group 0 is default; see [Regex] for more.
///
/// This enum simply represents a specification of a capture group. It does not guarantee that the
/// capture group is present in a regular expression, nor does it even guarantee that a regular
/// expression exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureGroup {
    /// A numbered capture group, where 0 is the default group, i.e. the whole match.
    Number(usize),

    /// A named capture group.
    Name(String),
}

/// Specifies the user's chosen grouper.
#[derive(Clone, Debug)]
pub enum GroupingSpecifier {
    /// Group by the first `usize` characters of each token.
    FirstChars(usize),

    /// Group by the last `usize` characters of each token.
    LastChars(usize),

    /// Group by the provided regular expression. See [crate::matchers::string::match_regex] for
    /// details.
    Regex(Regex, CaptureGroup),

    /// Group by file extension. See [crate::matchers::string::match_file_extension] for details.
    FileExtension,

    /// Group by counter. See [crate::matchers::string::match_counter] for details.
    Counter,
}

// For ease of use implementing PartialEq below.
use GroupingSpecifier::*;

/// Options for controlling the program's output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputOptions {
    /// Specifies what type of separator to output between records.
    pub separator: Separator,

    /// Output only group names; do not group contents.
    pub only_group_names: bool,

    /// If `Some`, pass each group to the command string as its stdin instead of printing
    /// the group's contents. Instead, print any output from the command under the
    /// group's header.
    ///
    /// If the user specifies this option, all other members of this struct apply to the command
    /// invocations instead of the final output; the final output should use default options. This
    /// is for sanity as well as generality: it might make sense to provide input to a program in
    /// an easy-to-parse, hard-to-read way (such as Separator::Null), but the final output should
    /// be tailored for human consumption. If a need arises, we can add an option or set of options
    /// to accommodate specific final output requirements for program output.
    pub run_command: Option<String>,

    /// If true, run commands in parallel, in arbitrary order (using work stealing).
    ///
    /// If false, run commands in sequence rather than in parallel, using a single thread of
    /// execution. While much slower, this guarantees that commands run in the same order as the
    /// groups they represent, which is sometimes necessary (e.g. for some database operations).
    pub parallel: bool,

    /// Whether to print a header for each group with final output.
    ///
    /// When [OutputOptions::run_command] is a `Some` value, the commands' behavior is not affected;
    /// instead, this option controls whether the final output, i.e. the printing of commands'
    /// results, includes headers for each group or just each group's contents back-to-back.
    pub headers: bool,

    /// Print statistics: an item count for each group and stats about the collection overall.
    /// Not affected by run_command.
    pub stats: bool,
}

/// The main options struct that holds all other options.
///
/// Each field in this struct is a category of options.
///
/// Note: for safety, users are strongly recommended to own such a struct immutably.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupByOptions {
    pub input: InputOptions,
    pub grouping: GroupingSpecifier,
    pub output: OutputOptions,
}

impl Separator {
    /// Returns a static str separator that corresponds to the enum variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use groupby::command_line::options::Separator;
    /// assert_eq!(Separator::Line.sep(), "\n");
    /// assert_eq!(Separator::Space.sep(), " ");
    /// assert_eq!(Separator::Null.sep(), "\0");
    /// ```
    pub fn sep(&self) -> String {
        match self {
            Separator::Line => "\n".to_string(),
            Separator::Space => " ".to_string(),
            Separator::Null => "\0".to_string(),
            Separator::Custom(s) => s.clone(),
        }
    }
}

/// We can't derive PartialEq and Eq for GroupingSpecifier because Regex is not PartialEq
/// or Eq, so we manually implement them with the following definitions:
///
/// FirstChars(m) == FirstChars(n) iff m == n
/// LastChars(m) == LastChars(n) iff m == n
/// Regex(re1, cg1) == Regex(re2, cg2) iff re1.as_str() == re2.as_str() && cg1 == cg2
///
/// # Examples
///
/// ```
/// use groupby::command_line::options::{GroupingSpecifier::*, CaptureGroup};
/// use regex;
///
/// // Same == same.
/// assert_eq!(FirstChars(7), FirstChars(7));
/// assert_eq!(LastChars(8), LastChars(8));
/// assert_eq!(
///     Regex(regex::Regex::new("foo").unwrap(), CaptureGroup::Number(4)),
///     Regex(regex::Regex::new("foo").unwrap(), CaptureGroup::Number(4))
/// );
/// assert_eq!(FileExtension, FileExtension);
/// assert_eq!(Counter, Counter);
///
/// // Same variant with different contained values are !=.
/// assert_ne!(FirstChars(7), FirstChars(8));
/// assert_ne!(LastChars(8), LastChars(9));
/// assert_ne!(
///     Regex(regex::Regex::new("foo").unwrap(), CaptureGroup::Number(0)),
///     Regex(regex::Regex::new("bar").unwrap(), CaptureGroup::Number(0))
/// );
/// assert_ne!(
///     Regex(regex::Regex::new("foo").unwrap(), CaptureGroup::Number(0)),
///     Regex(regex::Regex::new("foo").unwrap(), CaptureGroup::Number(1))
/// );
///
/// // Different variants are !=.
/// assert_ne!(FirstChars(7), Regex(regex::Regex::new("bar").unwrap(), CaptureGroup::Number(0)));
/// assert_ne!(LastChars(8), FirstChars(8));
/// assert_ne!(
///     Regex(regex::Regex::new("foo").unwrap(), CaptureGroup::Number(3)),
///     LastChars(9)
/// );
/// assert_ne!(FirstChars(7), FileExtension);
/// assert_ne!(FileExtension, Counter);
/// ```
impl PartialEq for GroupingSpecifier {
    fn eq(&self, other: &Self) -> bool {
        match self {
            FirstChars(m) => match other {
                FirstChars(n) => m == n,
                _ => false,
            },
            LastChars(m) => match other {
                LastChars(n) => m == n,
                _ => false,
            },
            Regex(re1, cg1) => match other {
                Regex(re2, cg2) => re1.as_str() == re2.as_str() && cg1 == cg2,
                _ => false,
            },
            FileExtension => matches!(other, FileExtension),
            Counter => matches!(other, Counter),
        }
    }
}

/// GroupingSpecifier has a full equivalence relation.
impl Eq for GroupingSpecifier {}
