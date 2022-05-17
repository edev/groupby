use super::*;
use std::convert::AsRef;
use std::ffi::OsStr;
use std::io;
use std::process::{Child, Command, Stdio};

// Mirrors the way we use std::process::Command. This allows us to use dependency injection in our
// tests: MockCommand implements RunCommand.
pub trait RunCommand {
    type Child: RunCommandChild;

    fn new<S: AsRef<OsStr>>(program: S) -> Self;

    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    fn spawn(&mut self) -> io::Result<Self::Child>;

    fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self;

    fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self;
}

// These methods are not tested, since it is not feasible to test them.
impl RunCommand for std::process::Command {
    type Child = Child;

    fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Command::new(program)
    }

    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args(args)
    }

    fn spawn(&mut self) -> io::Result<Self::Child> {
        self.spawn()
    }

    fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.stdin(cfg)
    }

    fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.stdout(cfg)
    }
}
