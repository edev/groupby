use regex::Regex;
use crate::matchers::*;
use crate::grouped_collection::*;

impl GroupedCollection {
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
}

