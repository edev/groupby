//! Matchers for [String] values.

use crate::command_line::CaptureGroup;
use global_counter::primitive::exact::CounterUsize;
use regex::Regex;

/// Returns the first n characters of a string.
///
/// Returns the first `n` characters of `string`, or all of `string` if `n > string.len()`.
///
/// If `string == ""` or `n == 0`, returns `""`.
///
/// # Examples
///
/// ```
/// use groupby::matchers::string;
///
/// let string = "Hello, world";
/// assert_eq!("Hello", string::match_first_n_chars(string, 5));
/// assert_eq!("Hello, world", string::match_first_n_chars(string, 20));
/// assert_eq!("", string::match_first_n_chars("", 5));
/// assert_eq!("", string::match_first_n_chars(string, 0));
/// ```
pub fn match_first_n_chars(string: &str, n: usize) -> &str {
    if n > string.len() {
        string
    } else {
        &string[0..n]
    }
}

/// Returns the lsat n characters of a string.
///
/// Returns the last `n` characters of `string`, or all of `string` if `n > string.len()`.
///
/// If `string == ""` or `n == 0`, returns `""`.
///
/// # Examples
///
/// ```
/// use groupby::matchers::string;
///
/// let string = "Hello, world";
/// assert_eq!("world", string::match_last_n_chars(string, 5));
/// assert_eq!("Hello, world", string::match_last_n_chars(string, 20));
/// assert_eq!("", string::match_last_n_chars("", 5));
/// assert_eq!("", string::match_last_n_chars(string, 0));
/// ```
pub fn match_last_n_chars(string: &str, n: usize) -> &str {
    if n > string.len() {
        string
    } else {
        &string[(string.len() - n)..]
    }
}

/// Returns the first match of the regular expression within a string, if any.
///
/// If the regular expression includes capture groups, returns the first capture group's match.
/// Otherwise, returns the overall match.
///
/// # Examples
///
/// ```
/// use groupby::command_line::CaptureGroup;
/// use groupby::matchers::string;
///
/// let first_word = regex::Regex::new(r"\w+").unwrap();
/// let second_word = regex::Regex::new(r"\w+\W+(\w+)").unwrap();
/// let third_word = regex::Regex::new(r"(?:\w+\W+){2}(\w+)").unwrap();
/// let third_capture_group = regex::Regex::new(r"(\w+)\W(\w+)\W(\w+)").unwrap();
///
/// assert_eq!(
///     Some("Bishop"),
///     string::match_regex("Bishop takes queen", &first_word, &CaptureGroup::Number(0)),
/// );
/// assert_eq!(
///     Some("takes"),
///     string::match_regex("Bishop takes queen", &second_word, &CaptureGroup::Number(1)),
/// );
/// assert_eq!(
///     Some("queen"),
///     string::match_regex("Bishop takes queen", &third_word, &CaptureGroup::Number(1)),
/// );
/// assert_eq!(
///     Some("queen"),
///     string::match_regex(
///         "Bishop takes queen", &third_capture_group, &CaptureGroup::Number(3)
///     ),
/// );
/// ```
pub fn match_regex<'a>(
    string: &'a str,
    regex: &Regex,
    capture_group: &CaptureGroup,
) -> Option<&'a str> {
    let captures = match regex.captures(string) {
        Some(captures) => captures,
        None => return None,
    };

    match capture_group {
        CaptureGroup::Number(n) => captures.get(*n).map(|mat| mat.as_str()),
        CaptureGroup::Name(s) => captures.name(s).map(|mat| mat.as_str()),
    }
}

/// Returns the characters after the last period in the string, if any. Doesn't match dotfiles.
///
/// Files with compound file extensions like `.tar.gz` will only match the last extension.
///
/// Files with no extension will yield `None`.
///
/// Files that start with a period and have no other periods will yield `None`.
///
/// Files that end with a period will yield `None` (and may be illegal on the local filesystem).
///
/// If you need a different definition of a file extension for your matcher, consider using
/// [match_regex] instead.
///
/// # Examples
///
/// ```
/// use groupby::matchers::string;
///
/// assert_eq!(Some("txt"), string::match_file_extension("some.file.of.mine.txt"));
/// assert_eq!(Some("gz"), string::match_file_extension("an archive.tar.gz"));
/// assert_eq!(Some("gz"), string::match_file_extension(".hidden.gz"));
/// assert_eq!(None, string::match_file_extension("Gemfile"));
/// assert_eq!(None, string::match_file_extension(".bashrc"));
/// assert_eq!(None, string::match_file_extension(".bashrc"));
/// assert_eq!(None, string::match_file_extension("probably illegal."));
/// ```
pub fn match_file_extension(filename: &str) -> Option<&str> {
    match filename.rfind('.') {
        Some(0) => None,
        Some(i) if i >= filename.len() - 1 => None,
        Some(i) => filename.get((i + 1)..),
        None => None,
    }
}

/// Returns the number of times the function has been called before.
///
/// Returns the next number from a thread-safe, global counter (starting from 0). This can be used
/// to provide a unique, stable, and readable key for each item in a collection, for instance.
///
/// ```
/// use groupby::matchers::string;
///
/// for i in 0..5 {
///     assert_eq!(i, string::match_counter());
/// }
/// ```
pub fn match_counter() -> usize {
    static COUNTER: CounterUsize = CounterUsize::new(0);
    COUNTER.inc()
}
