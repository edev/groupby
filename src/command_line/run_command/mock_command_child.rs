// Because fo the trivial nature of the methods below, the fact that they're used in many tests,
// and the fact that they're only used in tests, this module is not unit-tested.

use super::*;
use std::io;

// Simulates a std::process:Child for testing purposes.
#[derive(Clone, Eq, PartialEq)]
pub struct MockCommandChild {
    pub command: MockCommand,
    pub output: &'static [u8],

    // We use Options here to mirror the way std::process::Child works. This, in turn, helps
    // the types and traits we're implementing work out much more easily.
    pub stdin: Option<Vec<u8>>,
    pub stdout: Option<&'static [u8]>,
}

impl MockCommandChild {
    pub fn new(command: &MockCommand) -> MockCommandChild {
        let out = b"the program output is a lie";
        MockCommandChild {
            command: command.clone(),
            output: out,
            stdin: Some(vec![]),
            stdout: Some(out),
        }
    }

    pub fn command(&self) -> &MockCommand {
        &self.command
    }
}

impl RunCommandChild for MockCommandChild {
    type Output = Self::Stdout; // If needed, we can switch this to a trait later.
    type Stdin = Vec<u8>;
    type Stdout = &'static [u8];

    fn stdin(&mut self) -> Self::Stdin {
        self.stdin.take().unwrap()
    }

    fn stdout(&mut self) -> Self::Stdout {
        self.stdout.take().unwrap()
    }

    fn wait_with_output(self) -> io::Result<Self::Output> {
        Ok(self.output)
    }
}
