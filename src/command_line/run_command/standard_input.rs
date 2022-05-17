use std::io::{BufWriter, Write};

pub struct StandardInput<'a, W: Write> {
    stdin: BufWriter<W>,
    separator: &'a [u8],
}

impl<'a, W: Write> StandardInput<'a, W> {
    pub fn new(stdin: W, separator: &'a [u8]) -> Self {
        let stdin = BufWriter::new(stdin);
        StandardInput { stdin, separator }
    }

    pub fn provide(&mut self, value: &'_ str) {
        self.write(value);
        self.stdin.flush().unwrap();
    }

    pub fn provide_all<I, S>(&mut self, values: I)
    where
        I: Iterator<Item = &'a S>,
        S: 'a + ToString,
    {
        for value in values {
            self.write(&value.to_string());
        }
        self.stdin.flush().unwrap();
    }

    fn write(&mut self, value: &str) {
        self.stdin.write_all(value.as_bytes()).unwrap();
        self.stdin.write_all(self.separator).unwrap();
    }

    pub fn stdin(self) -> BufWriter<W> {
        self.stdin
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
            let stdin = StandardInput::new(v.clone(), sep);
            assert_eq!(stdin.stdin.into_inner().unwrap(), v);
            assert_eq!(stdin.separator, sep);
        }
    }

    mod provide {
        use super::*;

        #[test]
        fn writes_with_separator_and_flushes() {
            let mut stdin = StandardInput::new(vec![], b"hoo");
            stdin.provide("boo");
            assert_eq!(stdin.stdin.into_inner().unwrap(), b"boohoo");
        }

        #[test]
        #[should_panic]
        fn panics_if_write_fails() {
            let mut buf = [0, 0];
            let writer = &mut buf[0..2];
            let mut stdin = StandardInput::new(writer, &[3, 4]);
            stdin.provide("ab");
            drop(stdin);
            println!("{:?}", buf);
        }
    }

    mod provide_all {
        use super::*;

        #[test]
        fn writes_all_values_and_flushes() {
            let values = ["My", "dog", "ate", "my", "homework"];
            let sep = ",\t";
            let mut buf = vec![];

            let mut stdin = StandardInput::new(&mut buf, sep.as_bytes());
            stdin.provide_all(values.iter());

            let expected: Vec<u8> = (values.join(sep) + sep).into_bytes();
            assert_eq!(&expected, stdin.stdin.into_inner().unwrap());
        }
    }

    mod stdin {
        use super::*;

        #[test]
        fn moves_stdin() {
            // Nearly the same as provide_all::writes_all_values_and_flushes(), except here we'll
            // use stdin.stdin() instead of stdin.stdin.

            let values = ["My", "dog", "ate", "my", "homework"];
            let sep = ",\t";
            let mut buf = vec![];

            let mut stdin = StandardInput::new(&mut buf, sep.as_bytes());
            stdin.provide_all(values.iter());

            let expected: Vec<u8> = (values.join(sep) + sep).into_bytes();
            assert_eq!(&expected, stdin.stdin().into_inner().unwrap());
        }
    }
}
