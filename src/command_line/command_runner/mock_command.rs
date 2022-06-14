// Because fo the trivial nature of the methods below, the fact that they're used in many tests,
// and the fact that they're only used in tests, this module is not unit-tested.

use super::*;
use std::convert::AsRef;
use std::ffi::OsStr;
use std::io;
use std::process::Stdio;

// Simulates a std::process::Command for testing purposes.
#[derive(Clone, Eq, PartialEq)]
pub struct MockCommand {
    // Records all calls so you can check them later. See methods for the exact text added.
    pub calls: Vec<String>,
}

impl Command for MockCommand {
    type Child = MockChild;

    fn new<S: AsRef<OsStr>>(program: S) -> Self {
        let s = format!("new({})", program.as_ref().to_string_lossy());
        MockCommand { calls: vec![s] }
    }

    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        // Parse args into Unicode Strings, then join them with commas.
        let args = args
            .into_iter()
            .map(|s| s.as_ref().to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(", ");

        self.calls.push(format!("args([{}])", args));
        self
    }

    fn spawn(&mut self) -> io::Result<Self::Child> {
        self.calls.push("spawn()".to_string());
        Ok(MockChild::new(&self))
    }

    fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.calls.push(format!("stdin({:?})", cfg.into()));
        self
    }

    fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.calls.push(format!("stdout({:?})", cfg.into()));
        self
    }
}
