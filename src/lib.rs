// ONLY for the design phase!
#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::HashMap;

// A collection of groups, with associated functions for inserting into and processing groups.
#[derive(Default, Clone)]
pub struct GroupedCollection {
    // A hash map of vectors or something like that goes here.
    groups: HashMap<String, String>,
}

// impl Iterator for GroupedCollection, returning a vector or something from the iterator.

impl GroupedCollection {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    // Functions to insert lines - you CAN safely mix and match, though it's questionable whether
    // you should.

    pub fn group_by_first_chars(&mut self, line: String, n: usize) {
        self.groups
            .insert(match_first_n_chars(&line, n).to_string(), line);
    }

    pub fn group_by_last_chars(line: String, n: usize) {}
    pub fn group_by_first_words(line: String, n: usize) {}
    pub fn group_by_last_words(line: String, n: usize) {}
    pub fn group_by_regexp(line: String, re: String) {} // Change re to regex. Mocked for simplicity.

    // ...
}

// Easily testable matchers that correspond to the implementations above.
// Can easily be used standalone or via GroupedCollection.

/// Returns the first `n` characters of `line`, or all of `line` if `n > line.len()`.
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
pub fn match_first_n_chars(line: &str, n: usize) -> &str {
    if n > line.len() {
        line
    } else {
        &line[0..n]
    }
}
