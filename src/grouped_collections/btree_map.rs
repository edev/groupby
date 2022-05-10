use crate::grouped_collections::GroupedCollection;
use std::collections::{btree_map, BTreeMap};

impl<'s, Key, Value> GroupedCollection<'s, Key, Value, Vec<Value>> for BTreeMap<Key, Vec<Value>>
where
    Self: 's,
    Key: Ord,
{
    type Iter = btree_map::Iter<'s, Key, Vec<Value>>;

    /// Adds `value` to the `Vec<Value>`  at `key` in insertion order.
    ///
    /// ```
    /// # use groupby::grouped_collections::GroupedCollection;
    /// # use std::collections::BTreeMap;
    /// let mut map: BTreeMap<bool, Vec<usize>> = BTreeMap::new();
    /// map.add(true, 1);
    /// assert_eq!(map.get(&true).unwrap(), &vec![1]);
    /// map.add(true, 2);
    /// assert_eq!(map.get(&true).unwrap(), &vec![1, 2]);
    /// ```
    fn add(&mut self, key: Key, value: Value) {
        match self.entry(key) {
            btree_map::Entry::Occupied(mut vec) => {
                vec.get_mut().push(value);
            }
            btree_map::Entry::Vacant(slot) => {
                slot.insert(vec![value]);
            }
        }
    }

    /// Wraps [BTreeMap::get()](std::collections::BTreeMap::get()).
    fn get(&'s self, key: &Key) -> Option<&'s Vec<Value>> {
        Self::get(self, key)
    }

    /// Wraps [BTreeMap::iter()](std::collections::BTreeMap::iter()).
    ///
    /// Iterates over key->group mappings in sort order by `key`. (Groups still preserve insertion
    /// order on values.)
    ///
    /// ```
    /// # use groupby::grouped_collections::GroupedCollection;
    /// # use std::collections::BTreeMap;
    /// let mut map: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    /// for value in ["hello", "there", "friend"] {
    ///     map.add(value.len(), value.to_string());
    /// }
    /// assert_eq!(
    ///     map.iter().collect::<Vec<_>>(),
    ///     vec![
    ///         (&5, &vec!["hello".to_string(), "there".to_string()]),
    ///         (&6, &vec!["friend".to_string()]),
    ///     ]
    /// );
    /// ```
    fn iter(&'s self) -> Self::Iter {
        Self::iter(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grouped_collections::test_helpers::*;

    #[test]
    fn add_get_iter() {
        verify_grouped_collection(BTreeMap::new());
    }
}
