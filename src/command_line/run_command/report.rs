//! Simple reporting of results from running commands on groups.

use std::collections::BTreeMap;
use std::sync::Mutex;

/// A common interface for single- and multi-threaded command runners to record results.
///
/// Single-threaded command runners  can use any type that implements this trait, such as
/// `BTreeMap<&str, T>`. Multi-threaded command runners can wrap any type that implements
/// this trait with `Mutex<_>` to gain access to the implementation of
/// [ReportInteriorMutable], which calls [Report::report] safely on the inner type.
pub trait Report<K, V> {
    fn report(&mut self, key: K, output: V);
}

impl<K: Ord, V> Report<K, V> for BTreeMap<K, V> {
    fn report(&mut self, key: K, output: V) {
        self.insert(key, output);
    }
}

/// Wraps [Report] in an `Mutex<_>` for multi-threaded reporting.
pub trait ReportInteriorMutable<K, V> {
    fn report(&self, key: K, output: V);
}

impl<K, R, V> ReportInteriorMutable<K, V> for Mutex<R>
where
    R: Report<K, V>,
{
    fn report(&self, key: K, output: V) {
        self.lock().unwrap().report(key, output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn works<'a>(map: &mut BTreeMap<&'a str, Vec<u8>>) {
        let results = "cat nap sofa sun warm smile";
        let key = "cat";
        map.report(key, results.as_bytes().to_vec());
        assert_eq!(results.as_bytes(), map.get(key).unwrap());
    }

    mod btree_map {
        use super::*;

        #[test]
        fn works() {
            super::works(&mut BTreeMap::new());
        }
    }

    mod arc_mutex_btree_map {
        use super::*;

        use std::ops::DerefMut;

        #[test]
        fn works_with_mutex() {
            let mutex = Mutex::new(BTreeMap::new());
            super::works(mutex.lock().unwrap().deref_mut());
        }

        #[test]
        fn works_with_arc_mutex() {
            let arc = Arc::new(Mutex::new(BTreeMap::new()));
            super::works(arc.lock().unwrap().deref_mut());
        }
    }
}
