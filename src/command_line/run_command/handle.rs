//! A high-level handle for a command started through [super::run::run()].
use super::*;
use crate::command_line::record_writer::RecordWriter;
use std::io;

/// A handle for a command started through [super::run::run()].
pub struct Handle<'a, CC: Child> {
    /// The handle's inner child. We prevent direct access without moving mainly to force the user
    /// to use our `wait_with_output` method, because calling
    /// [std::process::Child::wait_with_output()] without first dropping [Handle::stdin] will
    /// deadlock the calling code.
    ///
    /// If you know what you're doing and you want the raw child value back, you're free to take it;
    /// just remember that its standard input has been moved into a [RecordWriter].
    child: CC,

    /// A record-oriented writer for standard input. Each record you write is followed by the
    /// separator you indicated when creating the handle.
    pub stdin: RecordWriter<'a, CC::Stdin>,
}

impl<'a, CC: Child> Handle<'a, CC> {
    /// Creates a new handle for `child`.
    pub fn new(mut child: CC, sep: &'a str) -> Self {
        let stdin = RecordWriter::new(child.stdin(), sep.as_bytes());
        Handle { child, stdin }
    }

    /// Consumes Self and returns the underlying handle, e.g. [std::process::Child].
    pub fn child(self) -> CC {
        self.child
    }

    /// Equivalent to [std::process::Child::wait_with_output].
    ///
    /// If you mean to call that method, **call this one instead**! Because the handle's
    /// initializer moves the child's standard input into a [RecordWriter], it must be manually
    /// dropped to prevent deadlock. This method drops it before waiting.
    pub fn wait_with_output(self) -> io::Result<CC::Output> {
        drop(self.stdin);
        self.child.wait_with_output()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command() -> MockCommand {
        MockCommand::new("")
    }

    fn child() -> MockChild {
        MockChild::new(&command())
    }

    fn handle() -> Handle<'static, MockChild> {
        Handle::new(child(), " >> ")
    }

    mod stdin {
        use super::*;

        #[test]
        fn works() {
            // There's no easy way to establish equality due to the nature of the values involved,
            // so we have to write a mini integration test to reach a mocked stdin we can check.
            let mut handle = handle();
            let inputs = vec!["1", "2"];
            handle.stdin.write_all(inputs.iter());
            let buffer = handle.stdin.writer().into_inner().unwrap();
            assert_eq!(buffer, b"1 >> 2 >> ");
        }

        #[test]
        #[should_panic]
        fn panics_if_called_twice() {
            let mut child = child();
            child.stdin();
            child.stdin();
        }
    }

    mod child {
        use super::*;

        #[test]
        fn returns_child() {
            let program = "lazy-zebra";
            let command = MockCommand::new(program);
            let child = MockChild::new(&command);
            assert_eq!(child.command().calls[0], format!("new({})", program));
        }
    }
}
