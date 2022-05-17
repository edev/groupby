use super::*;
use std::io;

/// A handle for a command started through [super::command::run_command()].
pub struct CommandHandle<RCC: RunCommandChild> {
    child: RCC,
}

impl<RCC: RunCommandChild> CommandHandle<RCC> {
    /// Creates a new handle for `child`.
    pub fn new(child: RCC) -> Self {
        CommandHandle { child }
    }

    /// Returns a [StandardInput] that uses `separator` between entries.
    ///
    /// This method is only safe to call once, since it [takes](Option::take()) the
    /// handle's standard input.
    pub fn stdin<'a>(&mut self, separator: &'a str) -> StandardInput<'a, RCC::Stdin> {
        StandardInput::new(self.child.stdin(), separator.as_bytes())
    }

    /// Consumes Self and returns the underlying handle, e.g. [std::process::Child].
    pub fn child(self) -> RCC {
        self.child
    }

    pub fn wait_with_output(self) -> io::Result<RCC::Output> {
        self.child.wait_with_output()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command() -> MockCommand {
        MockCommand::new("")
    }

    fn child() -> MockCommandChild {
        MockCommandChild::new(&command())
    }

    fn handle() -> CommandHandle<MockCommandChild> {
        CommandHandle::new(child())
    }

    mod stdin {
        use super::*;

        #[test]
        fn works() {
            // There's no easy way to establish equality due to the nature of the values involved,
            // so we have to write a mini integration test to reach a mocked stdin we can check.
            let mut handle = handle();
            let mut stdin: StandardInput<Vec<u8>> = handle.stdin("2");
            let inputs = vec!["1", "3"];
            stdin.provide_all(inputs.iter());
            let buffer = stdin.stdin().into_inner().unwrap();
            assert_eq!(buffer, b"1232");
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
            let child = MockCommandChild::new(&command);
            assert_eq!(child.command().calls[0], format!("new({})", program));
        }
    }
}
