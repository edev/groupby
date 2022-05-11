//! Matchers for [String] values.
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
/// use groupby::matchers::string;
///
/// let first_word = regex::Regex::new(r"\w+").unwrap();
/// let second_word = regex::Regex::new(r"\w+\W+(\w+)").unwrap();
/// let third_word = regex::Regex::new(r"(?:\w+\W+){2}(\w+)").unwrap();
///
/// assert_eq!("Bishop", string::match_regex("Bishop takes queen", &first_word).unwrap());
/// assert_eq!("takes",  string::match_regex("Bishop takes queen", &second_word).unwrap());
/// assert_eq!("queen",  string::match_regex("Bishop takes queen", &third_word).unwrap());
/// ```
pub fn match_regex<'a, 'b>(string: &'a str, regex: &'b Regex) -> Option<&'a str> {
    match regex.captures(string) {
        Some(caps) => match caps.get(1) {
            Some(mat) => Some(&string[(mat.start()..mat.end())]),
            None => match caps.get(0) {
                Some(mat) => Some(&string[(mat.start()..mat.end())]),
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
// TODO Add matcher: nth regex capture group (for more complex scenarios with existing regex)
