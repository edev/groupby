use std::cmp::Eq;
use std::collections::{btree_map, hash_map, BTreeMap, HashMap};
use std::hash::Hash;

// TODO Diagnose & hopefully fix issue lifetime issue on Box<GroupedCollection>.iter().
//
// The example below doesn't work, and I don't quite know why. I've done everything I can at
// this stage to try to understand and resolve it. My guess is that it requires generic attribute
// types (GAT) to correctly map the lifetimes and resolve this, but I'm not yet experienced
// enough to feel confident in that conclusion, and I genuinely don't know what's going wrong.
// The trait works beautifully in every other situation I have tried.
//
// let mut map: Box<dyn GroupedCollection<u8, i8, Vec<i8>, Iter = hash_map::Iter<_, _>>> =
//     Box::new(HashMap::new());
// for i in (-3..0).rev() {
//     map.add(4, i);
// }
// map.iter();
// assert_eq!(&vec![-1, -2, -3], map.get(&4).unwrap());

// TODO Diagnose & hopefully fix borrowing issue with  &mut dyn GroupedCollection.
//
// The example below works perfectly if I use a HashMap variable instead of a trait object. I don't
// know what causes this issue or why. It's not a problem with my code as far as I can tell.
// ```
// # use groupby::grouped_collection::GroupedCollection;
// # use std::collections::{hash_map, HashMap};
// let map: &mut dyn GroupedCollection<
//     bool,
//     usize,
//     Vec<usize>,
//     Iter = hash_map::Iter<bool, Vec<usize>>,
//  > = &mut HashMap::new();
//
// map.add(true, 1);
// assert_eq!(map.get(&true).unwrap(), &vec![1]);
// map.add(true, 2);
// assert_eq!(map.get(&true).unwrap(), &vec![1, 2]);
// ```

/// Provides a common interface over collections that map keys to lists of values, e.g.
/// `BTreeMap<K, Vec<V>>`
///
/// For instance, a text-processing function might use `BTreeMap<String, Vec<String>>`.
///
/// # Generic parameters
///
/// - `Key`:   the type used for keys in the mapping data structure.
/// - `Value`: the type of value to be grouped under each key.
/// - `List`:  the list type that contains each group (probably `Vec<Value>`).
///
/// # Examples
///
/// ```
/// use groupby::grouped_collection::GroupedCollection;
/// use std::collections::{HashMap, BTreeMap};
///
/// fn foo<Map>(mut map: Map)
/// where
///     Map: for<'s> GroupedCollection<'s, u8, i8, Vec<i8>>,
/// {
///     for i in (-3..0).rev() {
///         map.add(4, i);
///     }
///     assert_eq!(&vec![-1, -2, -3], GroupedCollection::get(&map, &4).unwrap());
/// }
///
/// foo(HashMap::new());
/// foo(BTreeMap::new());
/// ```
///
/// # Limitations
///
/// I believe the correct way to express the lifetimes in this trait is through generic attribute
/// types (GAT), but they are not yet stable. Therefore, for now, there are some limitations to the
/// trait due to stable Rust's inability to express the lifetimes the trait employs or the
/// compiler's inability to understand them (or possibly a flaw in the trait design of which I am
/// unaware):
///
/// - Boxed trait objects (`Box<GroupedCollection<..>>`) do not work at this time.
///
/// - Trait objects can run into borrowing and lifetime issues; you should probably use generics
/// rather than trait objects where possible to avoid these issues. As far as I know, monomorphic
/// types (including generics) work perfectly.
///
/// If you have a solution to these issues, please submit a GitHub issue: at time of writing, I am
/// still fairly new to Rust and find these lifetime issues quite perplexing, so it's entirely
/// possible that a more experienced Rustacean might have an elegant solution! If you're interested
/// in attempting it, you may be want to read two StackOverflow questions I asked while designing
/// this trait
/// ([one](https://stackoverflow.com/questions/72114666/lifetime-mismatch-in-generic-trait-with-iterator-bound),
/// [two](https://stackoverflow.com/questions/72133462/lifetime-issue-with-generic-trait-bound)).
/// I spent around a week learning all I could about advanced lifetimes, trying to understand the
/// complexities of the situation, and trying to find the correct solution; what's here is the best
/// I've been able to do. Help from a more experienced Rustacean is very welcome!
pub trait GroupedCollection<'s, Key: 's, Value: 's, List: 's> {
    /// The type of iterator that [iter](GroupedCollection::iter) returns.
    type Iter: Iterator<Item = (&'s Key, &'s List)>;

    /// Adds `value` to the list at `key`.
    ///
    /// If `key` is not found, adds `key` to the mapping with a new `List` containing `value`.
    ///
    /// The order of values in a group depends on the `List` used; see the implementation you plan
    /// to use for details.
    ///
    /// ```
    /// # use groupby::grouped_collection::GroupedCollection;
    /// # use std::collections::{hash_map, HashMap};
    /// let map: &mut dyn GroupedCollection<bool, usize, Vec<usize>, Iter = hash_map::Iter<bool, Vec<usize>>> = &mut HashMap::new();
    /// map.add(true, 1);
    /// map.add(true, 2);
    /// assert_eq!(map.get(&true).unwrap(), &vec![1, 2]);
    /// ```
    fn add(&mut self, key: Key, value: Value);

    /// Retrieves the group (i.e. `List`) of values corresponding to `key`, if any.
    fn get(&'s self, key: &Key) -> Option<&'s List>;

    /// Returns an iterator over key->group mappings.
    ///
    /// The order in which the iterator returns the mappings depends on the implementor; see the
    /// implementation you plan to use for details.
    ///
    /// ```
    /// # use groupby::grouped_collection::GroupedCollection;
    /// # use std::collections::BTreeMap;
    /// let mut map: BTreeMap<bool, Vec<usize>> = BTreeMap::new();
    /// map.add(true, 1);
    /// map.add(true, 2);
    /// let mut iter = GroupedCollection::iter(&map);
    /// let (key, group): (&bool, &Vec<usize>) = iter.next().unwrap();
    /// assert_eq!(key, &true);
    /// assert_eq!(group, &vec![1, 2]);
    /// ```
    fn iter(&'s self) -> Self::Iter;

    // TODO Add more methods to form a more complete set of actions.
}

impl<'s, Key, Value> GroupedCollection<'s, Key, Value, Vec<Value>> for BTreeMap<Key, Vec<Value>>
where
    Self: 's,
    Key: Ord,
{
    type Iter = btree_map::Iter<'s, Key, Vec<Value>>;

    /// Adds `value` to the `Vec<Value>`  at `key` in insertion order.
    ///
    /// ```
    /// # use groupby::grouped_collection::GroupedCollection;
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
        Self::get(&self, key)
    }

    /// Wraps [BTreeMap::iter()](std::collections::BTreeMap::iter()).
    ///
    /// Iterates over key->group mappings in sort order by `key`. (Groups still preserve insertion
    /// order on values.)
    ///
    /// ```
    /// # use groupby::grouped_collection::GroupedCollection;
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
        Self::iter(&self)
    }
}

impl<'s, Key, Value> GroupedCollection<'s, Key, Value, Vec<Value>> for HashMap<Key, Vec<Value>>
where
    Self: 's,
    Key: Eq + Hash,
{
    type Iter = hash_map::Iter<'s, Key, Vec<Value>>;

    /// Adds `value` to the `Vec<Value>`  at `key` in insertion order.
    ///
    /// ```
    /// # use groupby::grouped_collection::GroupedCollection;
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
    /// # use groupby::grouped_collection::GroupedCollection;
    /// # use std::collections::HashMap;
    /// let mut map: HashMap<bool, Vec<usize>> = HashMap::new();
    /// map.add(true, 1);
    /// assert_eq!(GroupedCollection::get(&map, &true).unwrap(), &vec![1]);
    /// ```
    fn get(&'s self, key: &Key) -> Option<&'s Vec<Value>> {
        Self::get(&self, key)
    }

    /// Wraps [HashMap::iter()](std::collections::HashMap::iter()).
    ///
    /// Iterates over key->group mappings in arbitrary order. (Groups still preserve insertion
    /// order on values.)
    fn iter(&'s self) -> Self::Iter {
        Self::iter(&self)
    }
}

#[cfg(test)]
mod grouped_collection_tests {
    use super::*;

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum Animal {
        Beaver,
        Cat,
        Horse,
        Donkey,
    }
    use Animal::*;

    #[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
    enum Foot {
        Claw,  // Pairs with Beaver, Cat
        Hoof,  // Pairs with Horse, Donkey
        Talon, // Does not pair (empty group)
    }
    use Foot::*;

    fn verify_grouped_collection<Map>(mut map: Map)
    where
        Map: for<'a> GroupedCollection<'a, Foot, Animal, Vec<Animal>> + Clone,
    {
        let claw_pairs = [
            (Claw, Beaver),
            (Claw, Cat),
            (Claw, Beaver),
        ];

        let hoof_pairs = [
            (Hoof, Horse),
            (Hoof, Donkey),
        ];

        // Just to try to provide some additional safety checks against strange lifetime issues,
        // we'll run our tests against an owned map as well as some references and smart pointers.
        let refmap: &mut Map = &mut map.clone();
        let mut boxmap: Box<Map> = Box::new(map.clone());

        // The following does not work, hence the limitations section in the docs.
        // let mut boxdynmap: Box<dyn GroupedCollection<_, _, _, Iter = _>> = Box::new(map.clone());

        // Use add(); tests below will verify that it worked.
        for (k, v) in &claw_pairs {
            map.add(k.clone(), v.clone());
            refmap.add(k.clone(), v.clone());
            boxmap.add(k.clone(), v.clone());
        }
        for (k, v) in &hoof_pairs {
            map.add(k.clone(), v.clone());
            refmap.add(k.clone(), v.clone());
            boxmap.add(k.clone(), v.clone());
        }

        let claws = claw_pairs
            .iter()
            .map(|(_, v)| v.clone())
            .collect::<Vec<Animal>>();

        let hooves = hoof_pairs
            .iter()
            .map(|(_, v)| v.clone())
            .collect::<Vec<Animal>>();

        // Check get()
        assert_eq!(map.get(&Claw), Some(&claws));
        assert_eq!(map.get(&Hoof), Some(&hooves));
        assert_eq!(map.get(&Talon), None);

        assert_eq!(refmap.get(&Claw), Some(&claws));
        assert_eq!(refmap.get(&Claw), Some(&claws));
        assert_eq!(refmap.get(&Claw), Some(&claws));

        assert_eq!(boxmap.get(&Hoof), Some(&hooves));
        assert_eq!(boxmap.get(&Talon), None);
        assert_eq!(boxmap.get(&Hoof), Some(&hooves));

        // Check iter()
        let pairs = map.iter().collect::<Vec<_>>();
        assert!(pairs.contains(&(&Claw, &vec![Beaver, Cat, Beaver])));
        assert!(pairs.contains(&(&Hoof, &vec![Horse, Donkey])));

        let pairs = refmap.iter().collect::<Vec<_>>();
        assert!(pairs.contains(&(&Claw, &vec![Beaver, Cat, Beaver])));
        assert!(pairs.contains(&(&Hoof, &vec![Horse, Donkey])));

        let pairs = boxmap.iter().collect::<Vec<_>>();
        assert!(pairs.contains(&(&Claw, &vec![Beaver, Cat, Beaver])));
        assert!(pairs.contains(&(&Hoof, &vec![Horse, Donkey])));

        // There have been some weird lifetime issues with using add() after get() with references;
        // let's just quickly make sure it works here.
        map.add(Claw, Beaver);
        refmap.add(Claw, Beaver);
        boxmap.add(Claw, Beaver);
    }

    #[test]
    fn add_get_iter() {
        verify_grouped_collection(BTreeMap::new());
        verify_grouped_collection(HashMap::new());
    }
}
