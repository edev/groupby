//! Provides the [GroupedCollection] trait.

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
// # use groupby::grouped_collections::GroupedCollection;
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
/// The main reason for this trait is to allow us to define generic [groupers](crate::groupers)
/// over this trait rather than over individual concrete types. For instance, [string groupers] are
/// implemented on `GroupedCollection<'s, String, String, List>` where `List` is arbitrary (but
/// should probably be `Vec<String>`).
///
/// [string groupers]: crate::groupers::string
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
/// use groupby::grouped_collections::GroupedCollection;
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
///   rather than trait objects where possible to avoid these issues. As far as I know, monomorphic
///   types (including generics) work perfectly.
///
/// If you have a solution to these issues, please submit a GitHub issue: at time of writing, I am
/// still fairly new to Rust and find these lifetime issues quite perplexing, so it's entirely
/// possible that a more experienced Rustacean might have an elegant solution! If you're interested
/// in attempting it, you may be want to read two StackOverflow questions I asked while designing
/// this trait ([one], [two]). I spent around a week learning all I could about advanced lifetimes,
/// trying to understand the complexities of the situation, and trying to find the correct
/// solution; what's here is the best I've been able to do. Help from a more experienced Rustacean
/// is very welcome!
///
/// [one]: https://stackoverflow.com/questions/72114666/lifetime-mismatch-in-generic-trait-with-iterator-bound
/// [two]: https://stackoverflow.com/questions/72133462/lifetime-issue-with-generic-trait-bound
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
    /// # use groupby::grouped_collections::GroupedCollection;
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
    /// # use groupby::grouped_collections::GroupedCollection;
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
}
