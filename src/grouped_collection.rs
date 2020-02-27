use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

/// Defines a data structure that can store a collection of items while associating each item with
/// a group. For instance, when storing a list of strings, one of these structs might enable you to
/// categorize them according to their first character, then iterate over the strings in each group.
/// A collection of groups, with associated functions for inserting into and processing groups.
#[derive(Default, Clone)]
pub struct GroupedCollection {
    groups: HashMap<String, Vec<String>>, // TODO If practical, make this generic.
}

/// Defines an iterator over groups in a GroupedCollection. To construct this iterator, see the
/// `iter()` method of `GroupedCollection`.
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

    /// Adds `line` to the group specified by `key`, creating a new group if necessary.
    pub fn add(&mut self, key: String, line: String) {
        // TODO If generic, rename `line` everywhere
        match self.groups.entry(key) {
            Occupied(mut vec) => {
                vec.get_mut().push(line);
            }
            Vacant(slot) => {
                slot.insert(vec![line]);
            }
        }
    }

    /// Returns a GroupedCollectionIter over the groups in the collection.
    ///
    /// This iterator will iterate over groups in sort order according to their keys, i.e. the
    /// values that define the groups.
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
    /// A single group in a GroupedCollection, consisting of a key and a list of values.
    type Item = (&'a String, &'a Vec<String>);

    /// Returns the next group, if any.
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
