use std::cmp::Eq;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::hash::Hash;

/// Defines a data structure that can store a collection of items while associating each item with
/// a group. For instance, when storing a list of strings, one of these structs might enable you to
/// categorize them according to their first character, then iterate over the strings in each group.
/// A collection of groups, with associated functions for inserting into and processing groups.
#[derive(Default, Clone)]
pub struct GroupedCollection<KeyType, ValueType>
where
    KeyType: Eq + Hash + Ord,
{
    groups: HashMap<KeyType, Vec<ValueType>>,
}

/// Defines an iterator over groups in a GroupedCollection. To construct this iterator, see the
/// `iter()` method of `GroupedCollection`.
pub struct GroupedCollectionIter<'a, KeyType, ValueType>
where
    KeyType: Eq + Hash + Ord,
{
    collection: &'a GroupedCollection<KeyType, ValueType>,
    keys: Vec<&'a KeyType>,
    current: usize,
}

impl<KeyType, ValueType> GroupedCollection<KeyType, ValueType>
where
    KeyType: Eq + Hash + Ord,
{
    pub fn new() -> Self {
        Self {
            groups: HashMap::<KeyType, Vec<ValueType>>::new(),
        }
    }

    /// Adds `value to the group specified by `key`, creating a new group if necessary.
    ///
    /// # Examples
    ///
    /// ```
    /// # use groupby::grouped_collection::*;
    /// let mut coll = GroupedCollection::new();
    /// coll.add("foo".to_string(), "foobarbaz".to_string());
    /// coll.add("foo".to_string(), "foolish mortal".to_string());
    /// let expected = vec!["foobarbaz".to_string(), "foolish mortal".to_string()];
    /// assert_eq!(Some(&expected), coll.get(&"foo".to_string()));
    /// ```
    pub fn add(&mut self, key: KeyType, value: ValueType) {
        match self.groups.entry(key) {
            Occupied(mut vec) => {
                vec.get_mut().push(value);
            }
            Vacant(slot) => {
                slot.insert(vec![value]);
            }
        }
    }

    /// Returns a reference to the list of values associated with `key`, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// # use groupby::grouped_collection::*;
    /// let mut coll = GroupedCollection::new();
    /// let entries = [
    ///     ("Favorite fruits", "Bananas"),
    ///     ("Favorite fruits", "Apples"),
    ///     ("Types of hats",   "Fedoras")
    /// ];
    /// for (k, v) in &entries {
    ///     coll.add(k.to_string(), v.to_string());
    /// }
    ///
    /// let expected_fruits = vec!["Bananas".to_string(), "Apples".to_string()];
    /// assert_eq!(Some(&expected_fruits), coll.get(&"Favorite fruits".to_string()));
    ///
    /// let expected_hats = vec!["Fedoras".to_string()];
    /// assert_eq!(Some(&expected_hats), coll.get(&"Types of hats".to_string()));
    ///
    /// assert_eq!(None, coll.get(&"Genres of books".to_string()));
    /// ```
    pub fn get(&self, key: &KeyType) -> Option<&Vec<ValueType>> {
        self.groups.get(key)
    }

    /// Returns an iterator over the groups in the collection that will iterate over groups
    /// in sort order according to their keys.
    ///
    /// # Examples
    ///
    /// ```
    /// # use groupby::grouped_collection::*;
    /// let mut coll = GroupedCollection::new();
    ///
    /// let entries = [
    ///     ("Types of hats",   "Fedoras"),
    ///     ("Favorite fruits", "Bananas"),
    ///     ("Favorite fruits", "Apples")
    /// ];
    /// for (k, v) in &entries {
    ///     coll.add(k.to_string(), v.to_string());
    /// }
    ///
    /// let expected_fruits = (
    ///     "Favorite fruits".to_string(),
    ///     vec!["Bananas".to_string(), "Apples".to_string()]
    /// );
    /// let expected_hats = (
    ///     "Types of hats".to_string(),
    ///     vec!["Fedoras".to_string()]
    /// );
    /// let mut iter = coll.iter();
    /// assert_eq!(Some((&expected_fruits.0, &expected_fruits.1)), iter.next());
    /// assert_eq!(Some((&expected_hats.0, &expected_hats.1)), iter.next());
    /// assert_eq!(None, iter.next());
    /// ```
    pub fn iter(&self) -> GroupedCollectionIter<KeyType, ValueType> {
        let mut keys = self.groups.keys().collect::<Vec<&KeyType>>();
        keys.sort();
        GroupedCollectionIter {
            collection: &self,
            keys,
            current: 0,
        }
    }
}

impl<'a, KeyType, ValueType> Iterator for GroupedCollectionIter<'a, KeyType, ValueType>
where
    KeyType: Eq + Hash + Ord,
{
    /// A single group in a GroupedCollection, consisting of a key and a list of values.
    type Item = (&'a KeyType, &'a Vec<ValueType>);

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
