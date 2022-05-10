use crate::grouped_collections::GroupedCollection;
use std::collections::{hash_map, HashMap};
use std::hash::Hash;

impl<'s, Key, Value> GroupedCollection<'s, Key, Value, Vec<Value>> for HashMap<Key, Vec<Value>>
where
    Self: 's,
    Key: Eq + Hash,
{
    type Iter = hash_map::Iter<'s, Key, Vec<Value>>;

    /// Adds `value` to the `Vec<Value>`  at `key` in insertion order.
    ///
    /// ```
    /// # use groupby::grouped_collections::GroupedCollection;
    /// # use std::collections::HashMap;
    /// let mut map: HashMap<bool, Vec<usize>> = HashMap::new();
    /// map.add(true, 1);
    /// assert_eq!(map.get(&true).unwrap(), &vec![1]);
    /// map.add(true, 2);
    /// assert_eq!(map.get(&true).unwrap(), &vec![1, 2]);
    /// ```
    fn add(&mut self, key: Key, value: Value) {
        match self.entry(key) {
            hash_map::Entry::Occupied(mut vec) => {
                vec.get_mut().push(value);
            }
            hash_map::Entry::Vacant(slot) => {
                slot.insert(vec![value]);
            }
        }
    }

    /// Wraps [HashMap::get()](std::collections::HashMap::get()).
    ///
    /// ```
    /// # use groupby::grouped_collections::GroupedCollection;
    /// # use std::collections::HashMap;
    /// let mut map: HashMap<bool, Vec<usize>> = HashMap::new();
    /// map.add(true, 1);
    /// assert_eq!(GroupedCollection::get(&map, &true).unwrap(), &vec![1]);
    /// ```
    fn get(&'s self, key: &Key) -> Option<&'s Vec<Value>> {
        Self::get(self, key)
    }

    /// Wraps [HashMap::iter()](std::collections::HashMap::iter()).
    ///
    /// Iterates over key->group mappings in arbitrary order. (Groups still preserve insertion
    /// order on values.)
    fn iter(&'s self) -> Self::Iter {
        Self::iter(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::grouped_collections::test_helpers::*;
    use std::collections::{HashMap};

    #[test]
    fn add_get_iter() {
        verify_grouped_collection(HashMap::new());
    }
}
