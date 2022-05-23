//! The [Child] trait, which allows dependency injection for child processes.

use std::io::{self, Read, Write};
use std::process::{self, ChildStdin, ChildStdout, Output};

/// Mirrors the way we use [std::process::Child]. This allows us to use dependency injection in our
/// tests: MockChild also implements Child.
///
/// This trait requires implementors to provide simple getters for standard input and output so
/// that calling code does not need to concern itself with the messy details of how they are
/// retrieved. For some implementations, these methods may panic if called more than once.
pub trait Child {
    type Output;
    type Stdin: Write;
    type Stdout: Read;

    /// Returns a writer for standard input.
    ///
    /// Calling this method more than once may result in a panic, depending on the implementation.
    fn stdin(&mut self) -> Self::Stdin;

    /// Returns a reader for standard output.
    ///
    /// Calling this method more than once may result in a panic, depending on the implementation.
    fn stdout(&mut self) -> Self::Stdout;

    /// Synchronously wait for the child process to finish, then return its captured output.
    ///
    /// Depending on the implementor, output may include, for instance, return status, standard
    /// output, and standard error). If you have called `stdout()`, then the standard output may be
    /// blank or missing.
    fn wait_with_output(self) -> io::Result<Self::Output>;
}

// These methods are not tested, since it is not feasible to test them.
impl Child for process::Child {
    type Output = Output;
    type Stdin = ChildStdin;
    type Stdout = ChildStdout;

    /// Panics if called more than once.
    fn stdin(&mut self) -> Self::Stdin {
        self.stdin.take().unwrap()
    }

    /// Panics if called more than once.
    fn stdout(&mut self) -> Self::Stdout {
        self.stdout.take().unwrap()
    }

    fn wait_with_output(self) -> io::Result<Self::Output> {
        process::Child::wait_with_output(self)
    }
}
