//! A collection of helper methods for grouping strings into a [GroupedCollection].

use crate::grouped_collections::*;
use crate::matchers::*;
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
    /// use groupby::groupers::Groupers;
    /// use std::collections::HashMap;
    ///
    /// let expected = vec!["kaledonia".to_string()];
    /// let mut map = HashMap::new();
    /// map.group_by_first_chars(expected[0].clone(), 4);
    ///
    /// assert_eq!(Some(&expected), map.get(&"kale".to_string()));
    /// ```
    fn group_by_first_chars(&mut self, line: String, n: usize);

    /// Groups a String according to its last `n` characters and adds it to the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use groupby::grouped_collections::*;
    /// use groupby::groupers::Groupers;
    /// use std::collections::BTreeMap;
    ///
    /// let expected = vec!["Sally".to_string()];
    /// let mut map = BTreeMap::new();
    /// map.group_by_last_chars(expected[0].clone(), 4);
    ///
    /// assert_eq!(Some(&expected), map.get(&"ally".to_string()));
    /// ```
    fn group_by_last_chars(&mut self, line: String, n: usize);

    /// Groups a String according to the provided Regex and adds it to the collection.
    ///
    /// See `match_regex` for details on how the key is determined.
    ///
    /// # Examples
    ///
    /// ```
    /// use groupby::grouped_collections::*;
    /// use groupby::groupers::Groupers;
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
    fn group_by_regex(&mut self, line: String, regex: &Regex);
}

impl<'s, List, GC> Groupers<List> for GC
where
    List: 's,
    GC: GroupedCollection<'s, String, String, List>,
{
    fn group_by_first_chars(&mut self, line: String, n: usize) {
        let key = match_first_n_chars(&line, n).to_string();
        self.add(key, line);
    }

    fn group_by_last_chars(&mut self, line: String, n: usize) {
        let key = match_last_n_chars(&line, n).to_string();
        self.add(key, line);
    }

    fn group_by_regex(&mut self, line: String, regex: &Regex) {
        let key = match_regex(&line, regex).unwrap_or("").to_string();
        self.add(key, line);
    }
}
