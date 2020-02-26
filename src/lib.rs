// ONLY for the design phase!
#![allow(dead_code)]
#![allow(unused_variables)]

use regex::Regex;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

// A collection of groups, with associated functions for inserting into and processing groups.
#[derive(Default, Clone)]
pub struct GroupedCollection {
    // A hash map of vectors or something like that goes here.
    groups: HashMap<String, Vec<String>>,
}

pub struct GroupedCollectionIter<'a> {
    collection: &'a GroupedCollection,
    keys: Vec<&'a String>,
    current: usize,
}

impl<'a> Iterator for GroupedCollectionIter<'a> {
    type Item = (&'a String, &'a Vec<String>);
    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            x if x < self.keys.len() => {
                self.current += 1;
                self.collection
                    .groups
                    .get_key_value(self.keys[self.current - 1])
            }
            _ => None,
        }
    }
}

impl GroupedCollection {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    fn add(&mut self, key: String, line: String) {
        match self.groups.entry(key) {
            Occupied(mut vec) => {
                vec.get_mut().push(line);
            }
            Vacant(slot) => {
                slot.insert(vec![line]);
            }
        }
    }

    // Functions to insert lines - you CAN safely mix and match, though it's questionable whether
    // you should.

    pub fn group_by_first_chars(&mut self, line: String, n: usize) {
        let key = match_first_n_chars(&line, n).to_string();
        self.add(key, line);
    }

    pub fn group_by_last_chars(&mut self, line: String, n: usize) {
        let key = match_last_n_chars(&line, n).to_string();
        self.add(key, line);
    }

    pub fn group_by_regex(&mut self, line: String, regex: &Regex) {
        let key = match match_regex(&line, &regex) {
            Some(s) => s,
            None => "",
        }
        .to_string();

        self.add(key, line);
    }

    pub fn iter(&self) -> GroupedCollectionIter {
        let mut keys = self.groups.keys().collect::<Vec<&String>>();
        keys.sort();
        GroupedCollectionIter {
            collection: &self,
            keys,
            current: 0,
        }
    }
}

// Easily testable matchers that correspond to the implementations above.
// Can easily be used standalone or via GroupedCollection.

/// Returns the first `n` characters of `line`, or all of `line` if `n > line.len()`.
/// If `line == ""` or `n == 0`, returns `""`.
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
/// let line = "This is not an empty string.";
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
/// let line = "This is not an empty string.";
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
/// ```
/// let first_word = regex::Regex::new(r"\w+").unwrap();
/// assert_eq!("Bishop", groupby::match_regex("Bishop takes queen", &first_word).unwrap());
/// ```
pub fn match_regex<'a, 'b>(line: &'a str, regex: &'b Regex) -> Option<&'a str> {
    match regex.find(line) {
        Some(mat) => Some(&line[(mat.start())..(mat.end())]),
        None => None,
    }
}
