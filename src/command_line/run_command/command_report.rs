use std::collections::BTreeMap;
use std::io::Read;
use std::sync::{Arc, Mutex};

/// A common interface for single- and multi-threaded command runners to record results.
///
/// Single-threaded command runners  can use `BTreeMap<&str, Vec<u8>>`, and multi-threaded runners
/// can use `Arc<Mutex<BTreeMap<&str, Vec<u8>>>>`.
trait CommandReport<'a, R: Read> {
    fn report(&mut self, key: &'a str, output: R);
}

impl<'a, R: Read> CommandReport<'a, R> for BTreeMap<&'a str, Vec<u8>> {
    fn report(&mut self, key: &'a str, mut output: R) {
        let mut buf: Vec<u8> = Vec::new();
        output.read_to_end(&mut buf).unwrap();
        self.insert(key, buf);
    }
}

impl<'a, R, CR> CommandReport<'a, R> for Arc<Mutex<CR>>
where
    R: Read,
    CR: CommandReport<'a, R>,
{
    fn report(&mut self, key: &'a str, output: R) {
        self.lock().unwrap().report(key, output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{self, Error, ErrorKind};

    struct PanicReader {}

    impl Read for PanicReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(Error::new(
                ErrorKind::PermissionDenied,
                "This is only a test",
            ))
        }
    }

    // Some helpers to keep us DRY.

    fn works<'a>(map: &mut BTreeMap<&'a str, Vec<u8>>) {
        let results = "cat nap sofa sun warm smile";
        let key = "cat";
        map.report(key, results.as_bytes());
        assert_eq!(results.as_bytes(), map.get(key).unwrap());
    }

    fn panics_on_output_error<'a>(map: &mut BTreeMap<&'a str, Vec<u8>>) {
        map.report("foo", PanicReader {});
    }

    mod btree_map {
        use super::*;

        #[test]
        fn works() {
            super::works(&mut BTreeMap::new());
        }

        #[test]
        #[should_panic]
        fn panics_on_output_error() {
            super::panics_on_output_error(&mut BTreeMap::new());
        }
    }

    mod arc_mutex_btree_map {
        use super::*;

        use std::ops::DerefMut;

        #[test]
        fn works() {
            let arc = Arc::new(Mutex::new(BTreeMap::new()));
            super::works(arc.lock().unwrap().deref_mut());
        }

        #[test]
        #[should_panic]
        fn panics_on_output_error() {
            let arc = Arc::new(Mutex::new(BTreeMap::new()));
            super::panics_on_output_error(arc.lock().unwrap().deref_mut());
        }
    }
}
