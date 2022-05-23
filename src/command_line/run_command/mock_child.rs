// Because fo the trivial nature of the methods below, the fact that they're used in many tests,
// and the fact that they're only used in tests, this module is not unit-tested.

use super::*;
use std::io;

// Simulates a std::process:Child for testing purposes.
#[derive(Clone, Eq, PartialEq)]
pub struct MockChild {
    pub command: MockCommand,
    pub output: &'static [u8],

    // We use Options here to mirror the way std::process::Child works. This, in turn, helps
    // the types and traits we're implementing work out much more easily.
    pub stdin: Option<Vec<u8>>,
    pub stdout: Option<&'static [u8]>,
}

impl MockChild {
    pub fn new(command: &MockCommand) -> MockChild {
        let out = b"the program output is a lie";
        MockChild {
            command: command.clone(),
            output: out,
            stdin: Some(vec![]),
            stdout: Some(out),
        }
    }

    // Convenience method; provides more consistent syntax when writing, e.g.,
    // handle.child().command().calls.
    pub fn command(&self) -> &MockCommand {
        &self.command
    }
}

impl Child for MockChild {
    type Output = Self::Stdout; // If needed, we can switch this to a trait later.
    type Stdin = Vec<u8>;
    type Stdout = &'static [u8];

    // Panics if called more than once, just like the real thing.
    fn stdin(&mut self) -> Self::Stdin {
        self.stdin.take().unwrap()
    }

    // Panics if called more than once, just like the real thing.
    fn stdout(&mut self) -> Self::Stdout {
        self.stdout.take().unwrap()
    }

    // Always succeeds.
    fn wait_with_output(self) -> io::Result<Self::Output> {
        Ok(self.output)
    }
}
