//! A collection of helper methods for grouping [Strings](String) into a [GroupedCollection].

use crate::command_line::options::GroupingSpecifier;
use crate::grouped_collections::*;
use crate::matchers::string::*;
use regex::Regex;

/// Provides helper methods for grouping strings into a [GroupedCollection].
///
/// Each method corresponds to a [matcher](crate::matchers).
pub trait Groupers<List> {
    /// Groups a String according to its first `n` characters and adds it to the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use groupby::grouped_collections::*;
    /// use groupby::groupers::string::Groupers;
    /// use std::collections::HashMap;
    ///
    /// let expected = vec!["kaledonia".to_string()];
    /// let mut map = HashMap::new();
    /// map.group_by_first_chars(expected[0].clone(), 4);
    ///
    /// assert_eq!(Some(&expected), map.get(&"kale".to_string()));
    /// ```
    fn group_by_first_chars<S: Into<String>>(&mut self, line: S, n: usize);

    /// Groups a String according to its last `n` characters and adds it to the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use groupby::grouped_collections::*;
    /// use groupby::groupers::string::Groupers;
    /// use std::collections::BTreeMap;
    ///
    /// let expected = vec!["Sally".to_string()];
    /// let mut map = BTreeMap::new();
    /// map.group_by_last_chars(expected[0].clone(), 4);
    ///
    /// assert_eq!(Some(&expected), map.get(&"ally".to_string()));
    /// ```
    fn group_by_last_chars<S: Into<String>>(&mut self, line: S, n: usize);

    /// Groups a String according to the provided Regex and adds it to the collection.
    ///
    /// See [match_regex] for details on how the key is determined.
    ///
    /// # Examples
    ///
    /// ```
    /// use groupby::grouped_collections::*;
    /// use groupby::groupers::string::Groupers;
    /// use regex::Regex;
    /// use std::collections::HashMap;
    ///
    /// let expected = vec!["Nineteen99".to_string()];
    /// let regex = Regex::new(r"\d+").unwrap();
    /// let mut map = HashMap::new();
    /// map.group_by_regex(expected[0].clone(), &regex);
    ///
    /// assert_eq!(Some(&expected), map.get(&"99".to_string()));
    /// ```
    fn group_by_regex<S: Into<String>>(&mut self, line: S, regex: &Regex);

    /// Groups a filename string by its extension.
    ///
    /// See [match_file_extension] for details on how file extensions are matched.
    ///
    /// # Examples
    ///
    /// ```
    /// use groupby::grouped_collections::*;
    /// use groupby::groupers::string::Groupers;
    /// use std::collections::BTreeMap;
    ///
    /// let expected_gz = vec!["foo.tar.gz".to_string(), "bar.gz".to_string()];
    /// let expected_none = vec!["my_file".to_string(), ".zshrc".to_string()];
    ///
    /// let mut map = BTreeMap::new();
    /// for s in &expected_gz {
    ///     map.group_by_file_extension(s.clone());
    /// }
    /// for s in &expected_none {
    ///     map.group_by_file_extension(s.clone());
    /// }
    ///
    /// assert_eq!(Some(&expected_gz), map.get(&"gz".to_string()));
    /// assert_eq!(Some(&expected_none), map.get(&"".to_string()));
    /// ```
    fn group_by_file_extension<S: Into<String>>(&mut self, filename: S);

    /// Assigns a unique, incremental index to each line provided, starting at 0.
    ///
    /// This allows each line to occupy its own group. It uses a thread-safe global counter.
    ///
    /// # Examples
    ///
    /// ```
    /// use groupby::grouped_collections::*;
    /// use groupby::groupers::string::Groupers;
    /// use std::collections::BTreeMap;
    ///
    /// let values = vec!["Zeroth".to_string(), "First".to_string(), "Second".to_string()];
    /// let mut map = BTreeMap::new();
    /// for v in &values {
    ///     map.group_by_counter(v.clone());
    /// }
    ///
    /// for (i, v) in values.iter().enumerate() {
    ///     assert_eq!(&vec![v.clone()], map.get(&i.to_string()).unwrap());
    /// }
    /// ```
    fn group_by_counter<S: Into<String>>(&mut self, line: S);
}

impl<'s, List, GC> Groupers<List> for GC
where
    List: 's,
    GC: GroupedCollection<'s, String, String, List>,
{
    fn group_by_first_chars<S: Into<String>>(&mut self, line: S, n: usize) {
        let line = line.into();
        let key = match_first_n_chars(&line, n).to_string();
        self.add(key, line);
    }

    fn group_by_last_chars<S: Into<String>>(&mut self, line: S, n: usize) {
        let line = line.into();
        let key = match_last_n_chars(&line, n).to_string();
        self.add(key, line);
    }

    fn group_by_regex<S: Into<String>>(&mut self, line: S, regex: &Regex) {
        let line = line.into();
        let key = match_regex(&line, regex).unwrap_or("").to_string();
        self.add(key, line);
    }

    fn group_by_file_extension<S: Into<String>>(&mut self, filename: S) {
        let filename = filename.into();
        let key = match_file_extension(&filename).unwrap_or("").to_string();
        self.add(key, filename);
    }

    fn group_by_counter<S: Into<String>>(&mut self, line: S) {
        let line = line.into();
        let key = match_counter().to_string();
        self.add(key, line);
    }
}

/// Provides a uniform interface to all string groupers.
///
/// Providing a uniform interface to all string groupers reduces the complexity of calling code
/// that might need to invoke the groupers at different times or under different conditions.
/// Specifically, it reduces the complexity of running a particular grouper based on a
/// [crate::command_line::options::GroupingSpecifier] from a match statement to simply
/// `runner.run(value)`.
///
/// # Examples
///
/// ```
/// use groupby::command_line::options::GroupingSpecifier;
/// use groupby::grouped_collections;
/// use groupby::groupers::string::Runner;
/// use std::collections::BTreeMap;
///
/// let mut map = BTreeMap::new();
/// let spec = GroupingSpecifier::FirstChars(2);
/// let mut runner = Runner::new(&mut map, &spec);
///
/// runner.run("Hi there".to_string());
/// drop(runner); // Runner stores &mut map and is meant for batch insertions
///
/// assert_eq!(map.get("Hi"), Some(&vec!["Hi there".to_string()]));
/// ```
pub struct Runner<'a, S: Into<String>> {
    run: Box<dyn FnMut(S) + 'a>,
}

impl<'a, S: Into<String>> Runner<'a, S> {
    pub fn new<Map>(map: &'a mut Map, spec: &'a GroupingSpecifier) -> Self
    where
        Map: for<'s> GroupedCollection<'s, String, String, Vec<String>>,
    {
        let run: Box<dyn FnMut(S)> = match spec {
            GroupingSpecifier::FirstChars(n) => Box::new(move |s| map.group_by_first_chars(s, *n)),
            GroupingSpecifier::LastChars(n) => Box::new(move |s| map.group_by_last_chars(s, *n)),
            GroupingSpecifier::Regex(re, _cg) => Box::new(move |s| map.group_by_regex(s, re)),
            GroupingSpecifier::FileExtension => Box::new(move |s| map.group_by_file_extension(s)),
            GroupingSpecifier::Counter => Box::new(move |s| map.group_by_counter(s)),
        };
        Runner { run }
    }

    /// Syntactic sugar so you can write `runner.run(value)` instead of `(runner.run)(value)`.
    pub fn run(&mut self, value: S) {
        (self.run)(value);
    }
}

#[cfg(test)]
mod tests {
    mod runner {
        use super::super::*;
        use crate::command_line::options::CaptureGroup;
        use crate::grouped_collections::fake_map::*;

        // Verifies that Runner actually uses a given GroupingSpecifier properly.
        fn matches(spec: GroupingSpecifier, value: &str, expected_key: &str) {
            let mut map = FakeMap::new();
            let mut runner = Runner::new(&mut map, &spec);
            runner.run(value.to_string());
            drop(runner);
            assert_eq!(*map.calls(), vec![format!("{}:{}", expected_key, value)]);
        }

        #[test]
        fn matches_first_chars() {
            matches(GroupingSpecifier::FirstChars(1), "abc", "a");
        }

        #[test]
        fn matches_last_chars() {
            matches(GroupingSpecifier::LastChars(1), "abc", "c");
        }

        #[test]
        fn matches_regex() {
            matches(
                GroupingSpecifier::Regex(Regex::new("b").unwrap(), CaptureGroup::Number(0)),
                "abc",
                "b",
            );
        }

        #[test]
        fn matches_file_extension() {
            matches(GroupingSpecifier::FileExtension, "abc.txt", "txt");
        }

        #[test]
        fn matches_counter() {
            matches(GroupingSpecifier::Counter, "abc", "0");
            matches(GroupingSpecifier::Counter, "abc", "1");
            matches(GroupingSpecifier::Counter, "abc", "2");
        }
    }
}
