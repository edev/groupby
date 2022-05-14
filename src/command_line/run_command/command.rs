use std::convert::AsRef;
use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// Runs a [std::process::Command] and returns its stdout.
// TODO Figure out what strategy for handling stderr is best.
// Should we give the option to abort if any command prints to stderr?
pub fn run_command<I, S>(program: S, shell_args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    command::<Command, _, _>(program, shell_args);
}

// A testable function that holds the main logic of run_command().
fn command<C, I, S>(program: S, shell_args: I) -> C::Child
where
    C: RunCommand,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    C::new(program)
        .args(shell_args)
        .stdin(Stdio::piped()) // Stdio::piped is not tested.
        .stdout(Stdio::piped()) // Stdio::piped is not tested.
        .spawn()
        .expect("Shell command failed.")
}

trait RunCommand {
    type Child;

    fn new<S: AsRef<OsStr>>(program: S) -> Self;

    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    fn spawn(&mut self) -> io::Result<Self::Child>;

    fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self;

    fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self;
}

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

trait RunCommandChild {
    type Stdin: Write;
    type Stdout: Read;

    fn stdin(&mut self) -> &mut Self::Stdin;
    fn stdout(&mut self) -> &mut Self::Stdout;
}

impl RunCommandChild for Child {
    type Stdin = ChildStdin;
    type Stdout = ChildStdout;

    fn stdin(&mut self) -> &mut Self::Stdin {
        self.stdin.as_mut().unwrap()
    }

    fn stdout(&mut self) -> &mut Self::Stdout {
        self.stdout.as_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct MockCommand {
        calls: Vec<String>,
    }

    impl RunCommand for MockCommand {
        type Child = MockCommandChild;

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
            Ok(MockCommandChild::new(&self))
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

    struct MockCommandChild {
        command: MockCommand,
        stdin: Vec<u8>,
        stdout: &'static [u8],
    }

    impl RunCommandChild for MockCommandChild {
        type Stdin = Vec<u8>;
        type Stdout = &'static [u8];

        fn stdin(&mut self) -> &mut Self::Stdin {
            &mut self.stdin
        }

        fn stdout(&mut self) -> &mut Self::Stdout {
            &mut self.stdout
        }
    }

    impl MockCommandChild {
        fn new(command: &MockCommand) -> MockCommandChild {
            MockCommandChild {
                command: command.clone(),
                stdin: vec![],
                stdout: b"the program output is a lie",
            }
        }
    }

    mod command {
        use super::*;

        #[test]
        fn spawns_command_correctly() {
            let program = "groupby";
            let shell_args = ["-f3", "-c", "echo recursion five!"];
            let child = command::<MockCommand, _, _>(program.clone(), shell_args.clone());

            let expected: Vec<String> = vec![
                "new(groupby)",
                "args([-f3, -c, echo recursion five!])",
                "stdin(Stdio { .. })",
                "stdout(Stdio { .. })",
                "spawn()",
            ]
            .iter()
            .map(ToString::to_string)
            .collect();

            assert_eq!(expected, child.command.calls);
        }
    }
}
