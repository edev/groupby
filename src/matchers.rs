use regex::Regex;

///! A collection of predefined string matchers for some common, universal use cases.

/// Returns the first `n` characters of `line`, or all of `line` if `n > line.len()`.
/// If `line == ""` or `n == 0`, returns `""`.
///
/// # Examples
///
/// ```
/// let line = "Hello, world";
/// assert_eq!("Hello", groupby::match_first_n_chars(line, 5));
/// ```
///
/// ```
/// let line = "Hi";
/// assert_eq!("Hi", groupby::match_first_n_chars(line, 5));
/// ```
///
/// ```
/// let line = "";
/// assert_eq!("", groupby::match_first_n_chars(line, 8));
/// ```
///
/// ```
/// let line = "This is not a string.";
/// assert_eq!("", groupby::match_first_n_chars(line, 0));
/// ```
pub fn match_first_n_chars(line: &str, n: usize) -> &str {
    if n > line.len() {
        line
    } else {
        &line[0..n]
    }
}

/// Returns the last `n` characters of `line`, or all of `line` if `n > line.len()`.
/// If `line == ""` or `n == 0`, returns `""`.
///
/// # Examples
///
/// ```
/// let line = "Hello, world";
/// assert_eq!("world", groupby::match_last_n_chars(line, 5));
/// ```
///
/// ```
/// let line = "Hi";
/// assert_eq!("Hi", groupby::match_last_n_chars(line, 5));
/// ```
///
/// ```
/// let line = "";
/// assert_eq!("", groupby::match_last_n_chars(line, 8));
/// ```
///
/// ```
/// let line = "This is not a string.";
/// assert_eq!("", groupby::match_last_n_chars(line, 0));
/// ```
pub fn match_last_n_chars(line: &str, n: usize) -> &str {
    if n > line.len() {
        line
    } else {
        &line[(line.len() - n)..]
    }
}

/// Returns the first match of the regular expression within the given line, if any.
///
/// If the regular expression includes capture groups, returns the first capture group's match.
/// Otherwise, returns the overall match.
///
/// # Examples
///
/// ```
/// let first_word = regex::Regex::new(r"\w+").unwrap();
///
/// let second_word = regex::Regex::new(r"\w+\W+(\w+)").unwrap();
///
/// let third_word = regex::Regex::new(r"(?:\w+\W+){2}(\w+)").unwrap();
///
/// assert_eq!("Bishop", groupby::match_regex("Bishop takes queen", &first_word).unwrap());
/// assert_eq!("takes",  groupby::match_regex("Bishop takes queen", &second_word).unwrap());
/// assert_eq!("queen",  groupby::match_regex("Bishop takes queen", &third_word).unwrap());
/// ```
pub fn match_regex<'a, 'b>(line: &'a str, regex: &'b Regex) -> Option<&'a str> {
    match regex.captures(line) {
        Some(caps) => match caps.get(1) {
            Some(mat) => Some(&line[(mat.start()..mat.end())]),
            None => match caps.get(0) {
                Some(mat) => Some(&line[(mat.start()..mat.end())]),
                None => None,
            },
        },
        None => None,
    }
}

// TODO Add matcher: first n words
// TODO Add matcher: last n words
// TODO Add matcher: file extension
// TODO Add matcher: nth word
