use std::io::{BufWriter, Write};

/// Record-oriented wrapper around a [writer](Write).
///
/// Provides a simple interface for writing either individual records or collections of records to
/// a writer, adding a separator after each one.
///
/// # Warnings
///
/// Values of this type take ownership of the writer you provide. This means, for instance, that if
/// you provide a [ChildStdin](std::process::ChildStdin), it will live as long as this struct and
/// will not be automatically dropped by methods on [Child](std::process::Child) such as
/// [wait_with_output()](std::process::Child::wait_with_output()). **This guarantees a deadlock
/// if you don't drop the StandardWriter before calling a method that waits!**
pub struct RecordWriter<'a, W: Write> {
    writer: BufWriter<W>,
    separator: &'a [u8],
}

impl<'a, W: Write> RecordWriter<'a, W> {
    pub fn new(writer: W, separator: &'a [u8]) -> Self {
        let writer = BufWriter::new(writer);
        RecordWriter { writer, separator }
    }

    pub fn write(&mut self, value: &'_ str) {
        self._write(value);
        self.writer.flush().unwrap();
    }

    pub fn write_all<I, S>(&mut self, values: I)
    where
        I: Iterator<Item = &'a S>,
        S: 'a + ToString,
    {
        for value in values {
            self._write(&value.to_string());
        }
        self.writer.flush().unwrap();
    }

    fn _write(&mut self, value: &str) {
        self.writer.write_all(value.as_bytes()).unwrap();
        self.writer.write_all(self.separator).unwrap();
    }

    pub fn writer(self) -> BufWriter<W> {
        self.writer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct MockWriter {
        received: String,
        flushed: bool,
    }

    impl MockWriter {
        fn new() -> Self {
            MockWriter {
                received: String::new(),
                flushed: false,
            }
        }

        fn check(&self, expected: &str, expected_flush: bool) {
            assert_eq!(&self.received, expected);
            assert_eq!(self.flushed, expected_flush);
        }
    }

    impl Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.received.push_str(&String::from_utf8_lossy(buf));
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushed = true;
            Ok(())
        }
    }

    mod new {
        use super::*;

        #[test]
        fn works() {
            let v: Vec<u8> = vec![b'a', b'c'];
            let sep = b"foo";
            let writer = RecordWriter::new(v.clone(), sep);
            assert_eq!(writer.writer.into_inner().unwrap(), v);
            assert_eq!(writer.separator, sep);
        }
    }

    mod write {
        use super::*;

        #[test]
        fn writes_with_separator_and_flushes() {
            let mut writer = RecordWriter::new(MockWriter::new(), b"hoo");
            writer.write("boo");
            writer.writer.into_inner().unwrap().check("boohoo", true);
        }

        #[test]
        #[should_panic(expected = "WriteZero")]
        fn panics_if_write_fails() {
            let mut buf = [0, 0];
            let writer = &mut buf[0..2];
            let mut writer = RecordWriter::new(writer, b"\0\0");
            writer.write("ab");
        }
    }

    mod write_all {
        use super::*;

        #[test]
        fn writes_all_values_and_flushes() {
            let values = ["My", "dog", "ate", "my", "homework"];
            let sep = ",\t";

            let mut writer = RecordWriter::new(MockWriter::new(), sep.as_bytes());
            writer.write_all(values.iter());

            let expected: String = values.join(sep) + sep;
            writer.writer.into_inner().unwrap().check(&expected, true);
        }
    }

    mod writer {
        use super::*;

        #[test]
        fn moves_writer() {
            let values = ["My", "dog", "ate", "my", "homework"];
            let sep = ",\t";
            let mut buf = vec![];

            let mut writer = RecordWriter::new(&mut buf, sep.as_bytes());
            writer.write_all(values.iter());

            let expected: Vec<u8> = (values.join(sep) + sep).into_bytes();

            // into_inner() will fail if writer() doesn't move RecordWriter::writer.
            assert_eq!(&expected, writer.writer().into_inner().unwrap());
        }
    }
}
