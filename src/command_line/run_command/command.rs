use super::*;
use std::convert::AsRef;
use std::ffi::OsStr;
use std::process::{Child, Command, Stdio};

/// Spawns a [std::process::Command] with piped I/O and returns a handle to it.
// TODO Figure out what strategy for handling stderr is best.
// Should we give the option to abort if any command prints to stderr?
pub fn run_command<'a, I>(program: &'a str, shell_args: I) -> CommandHandle<Child>
where
    I: IntoIterator<Item = &'a str>,
{
    command::<Command, _, _>(program, shell_args)
}

// A testable function that holds the main logic of run_command().
fn command<C, I, S>(program: S, shell_args: I) -> CommandHandle<C::Child>
where
    C: RunCommand,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let child = C::new(program)
        .args(shell_args)
        .stdin(Stdio::piped()) // Stdio::piped is not tested.
        .stdout(Stdio::piped()) // Stdio::piped is not tested.
        .spawn()
        .expect("Shell command failed.");

    CommandHandle::new(child)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod command {
        use super::*;

        #[test]
        fn spawns_command_correctly() {
            let program = "groupby";
            let shell_args = ["-f3", "-c", "echo recursion five!"];
            let handle = command::<MockCommand, _, _>(program.clone(), shell_args.clone());

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

            assert_eq!(expected, handle.child().command().calls);
        }
    }
}
