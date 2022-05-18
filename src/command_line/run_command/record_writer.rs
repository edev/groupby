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

    mod new {
        use super::*;

        #[test]
        fn works() {
            let v = vec![];
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
            let mut writer = RecordWriter::new(vec![], b"hoo");
            writer.write("boo");
            assert_eq!(writer.writer.into_inner().unwrap(), b"boohoo");
        }

        #[test]
        #[should_panic]
        fn panics_if_write_fails() {
            let mut buf = [0, 0];
            let writer = &mut buf[0..2];
            let mut writer = RecordWriter::new(writer, &[3, 4]);
            writer.write("ab");
            drop(writer);
            println!("{:?}", buf);
        }
    }

    mod write_all {
        use super::*;

        #[test]
        fn writes_all_values_and_flushes() {
            let values = ["My", "dog", "ate", "my", "homework"];
            let sep = ",\t";
            let mut buf = vec![];

            let mut writer = RecordWriter::new(&mut buf, sep.as_bytes());
            writer.write_all(values.iter());

            let expected: Vec<u8> = (values.join(sep) + sep).into_bytes();
            assert_eq!(&expected, writer.writer.into_inner().unwrap());
        }
    }

    mod writer {
        use super::*;

        #[test]
        fn moves_writer() {
            // Nearly the same as write_all::writes_all_values_and_flushes(), except here we'll
            // use writer.writer() instead of writer.writer.

            let values = ["My", "dog", "ate", "my", "homework"];
            let sep = ",\t";
            let mut buf = vec![];

            let mut writer = RecordWriter::new(&mut buf, sep.as_bytes());
            writer.write_all(values.iter());

            let expected: Vec<u8> = (values.join(sep) + sep).into_bytes();
            assert_eq!(&expected, writer.writer().into_inner().unwrap());
        }
    }
}
