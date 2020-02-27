use crate::grouped_collection::*;
use crate::matchers::*;
use regex::Regex;

///! A collection of associated functions to insert Strings into a GroupedCollection.

impl GroupedCollection {
    /// Groups a String according to its first `n` characters and adds it to the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use groupby::grouped_collection::*;
    /// let mut coll = GroupedCollection::new();
    /// let expected = vec!["kaledonia".to_string()];
    /// coll.group_by_first_chars(expected[0].clone(), 4);
    /// assert_eq!(Some(&expected), coll.get("kale"));
    /// ```
    pub fn group_by_first_chars(&mut self, line: String, n: usize) {
        let key = match_first_n_chars(&line, n).to_string();
        self.add(key, line);
    }

    /// Groups a String according to its last `n` characters and adds it to the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use groupby::grouped_collection::*;
    /// let mut coll = GroupedCollection::new();
    /// let expected = vec!["Sally".to_string()];
    /// coll.group_by_last_chars(expected[0].clone(), 4);
    /// assert_eq!(Some(&expected), coll.get("ally"));
    /// ```
    pub fn group_by_last_chars(&mut self, line: String, n: usize) {
        let key = match_last_n_chars(&line, n).to_string();
        self.add(key, line);
    }

    /// Groups a String according to the provided Regex and adds it to the collection.
    ///
    /// See `match_regex` for details on how the key is determined.
    ///
    /// # Examples
    ///
    /// ```
    /// # use groupby::grouped_collection::*;
    /// # use regex::Regex;
    /// let mut coll = GroupedCollection::new();
    /// let expected = vec!["Nineteen99".to_string()];
    /// let regex = Regex::new(r"\d+").unwrap();
    /// coll.group_by_regex(expected[0].clone(), &regex);
    /// assert_eq!(Some(&expected), coll.get("99"));
    /// ```
    pub fn group_by_regex(&mut self, line: String, regex: &Regex) {
        let key = match match_regex(&line, &regex) {
            Some(s) => s,
            None => "",
        }
        .to_string();

        self.add(key, line);
    }
}
