#![cfg(test)]

use crate::grouped_collections::GroupedCollection;

// A test double that records calls to GroupedCollection::add().
pub struct FakeMap {
    calls: Vec<String>,
}

impl<'s> GroupedCollection<'s, String, String, Vec<String>> for FakeMap {
    type Iter = FakeMapIter<'s>;

    // Record the key so we can check which grouper was used.
    fn add(&mut self, key: String, value: String) {
        self.calls.push(format!("{}:{}", key, value));
    }

    fn get(&'s self, _key: &String) -> Option<&'s Vec<String>> {
        None
    }

    fn iter(&'s self) -> Self::Iter {
        FakeMapIter {
            _keys: "".to_string(),
            _values: vec![],
            _fake_ref: &4,
        }
    }
}

impl FakeMap {
    pub fn new() -> Self {
        FakeMap { calls: vec![] }
    }

    pub fn calls(&self) -> &Vec<String> {
        &self.calls
    }
}

// Quickest thing that will work for both Iterator and GroupedCollection.
pub struct FakeMapIter<'s> {
    // Fields will, in fact, be empty.
    _keys: String,
    _values: Vec<String>,
    _fake_ref: &'s usize, // We have to use 's, and we need it for the impl.
}

impl<'s> Iterator for FakeMapIter<'s> {
    type Item = (&'s String, &'s Vec<String>);
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
