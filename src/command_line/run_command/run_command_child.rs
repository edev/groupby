use std::io::{self, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Output};

// Mirrors the way we use std::process::Child. This allows us to use dependency injection in our
// tests: MockCommandChild implements RunCommandChild.
//
// In order to clean up both tests and the functions they're testing, this provides a clean
// interface; implementors handle any messy details.
pub trait RunCommandChild {
    type Output;
    type Stdin: Write;
    type Stdout: Read;

    fn stdin(&mut self) -> Self::Stdin;
    fn stdout(&mut self) -> Self::Stdout;
    fn wait_with_output(self) -> io::Result<Self::Output>;
}

// These methods are not tested, since it is not feasible to test them.
impl RunCommandChild for Child {
    type Output = Output;
    type Stdin = ChildStdin;
    type Stdout = ChildStdout;

    fn stdin(&mut self) -> Self::Stdin {
        self.stdin.take().unwrap()
    }

    fn stdout(&mut self) -> Self::Stdout {
        self.stdout.take().unwrap()
    }

    fn wait_with_output(self) -> io::Result<Self::Output> {
        Child::wait_with_output(self)
    }
}
