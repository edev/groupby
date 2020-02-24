// ONLY for the design phase!
#![allow(dead_code)]
#![allow(unused_variables)]

// A collection of groups, with associated functions for inserting into and processing groups.
struct GroupedCollection {
    // A hash map of vectors or something like that goes here.
}

// impl Iterator for GroupedCollection, returning a vector or something from the iterator.

impl GroupedCollection {
    // Functions to insert lines - you CAN safely mix and match, though it's questionable whether
    // you should.
    fn group_by_first_chars(line: String, n: u64) { }
    fn group_by_last_chars(line: String, n: u64) { }
    fn group_by_first_words(line: String, n: u64) { }
    fn group_by_last_words(line: String, n: u64) { }
    fn group_by_regexp(line: String, re: String) { } // Change re to regex. Mocked for simplicity.
    // ...
}
