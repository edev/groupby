// Because fo the trivial nature of the methods below, the fact that they're used in many tests,
// and the fact that they're only used in tests, this module is not unit-tested.

use super::*;

// Simulates a std::process:Child for testing purposes.
#[derive(Clone, Eq, PartialEq)]
pub struct MockCommandChild {
    pub command: MockCommand,

    // We use Options here to mirror the way std::process::Child works. This, in turn, helps
    // the types and traits we're implementing work out much more easily.
    pub stdin: Option<Vec<u8>>,
    pub stdout: Option<&'static [u8]>,
}

impl MockCommandChild {
    pub fn new(command: &MockCommand) -> MockCommandChild {
        MockCommandChild {
            command: command.clone(),
            stdin: Some(vec![]),
            stdout: Some(b"the program output is a lie"),
        }
    }

    pub fn command(&self) -> &MockCommand {
        &self.command
    }
}

impl RunCommandChild for MockCommandChild {
    type Stdin = Vec<u8>;
    type Stdout = &'static [u8];

    fn stdin(&mut self) -> Self::Stdin {
        self.stdin.take().unwrap()
    }

    fn stdout(&mut self) -> Self::Stdout {
        self.stdout.take().unwrap()
    }
}
