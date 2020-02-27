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

impl GroupedCollection {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    pub fn add(&mut self, key: String, line: String) {
        match self.groups.entry(key) {
            Occupied(mut vec) => {
                vec.get_mut().push(line);
            }
            Vacant(slot) => {
                slot.insert(vec![line]);
            }
        }
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
