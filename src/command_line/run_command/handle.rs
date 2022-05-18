use super::*;
use std::io;

/// A handle for a command started through [super::command::run_command()].
pub struct Handle<'a, CC: Child> {
    child: CC,
    pub stdin: StandardInput<'a, CC::Stdin>,
}

impl<'a, CC: Child> Handle<'a, CC> {
    /// Creates a new handle for `child`.
    pub fn new(mut child: CC, sep: &'a str) -> Self {
        let stdin = StandardInput::new(child.stdin(), sep.as_bytes());
        Handle { child, stdin }
    }

    /// Consumes Self and returns the underlying handle, e.g. [std::process::Child].
    pub fn child(self) -> CC {
        self.child
    }

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
            handle.stdin.provide_all(inputs.iter());
            let buffer = handle.stdin.stdin().into_inner().unwrap();
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
