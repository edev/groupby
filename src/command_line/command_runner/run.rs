//! The [run()] function, which spawns a new process to run a shell command.

use super::*;
use std::convert::AsRef;
use std::ffi::OsStr;
use std::process::{self, Stdio};

/// Spawns a [std::process::Command] with piped I/O and returns a handle to it.
///
/// Note that standard error is not piped. Because we assume that we can't possibly know how the
/// user will want to handle error output, we simply allow it to immediately be displayed. It's
/// possible that this behavior might change in the future.
///
/// # Examples
///
/// ```
/// use groupby::command_line::command_runner::run::run;
///
/// let handle = run("bash", ["-c", "echo hi"], "");
/// let output = handle.wait_with_output().unwrap();
/// assert_eq!(String::from_utf8_lossy(&output.stdout), String::from("hi\n"));
/// ```
pub fn run<'a, I>(program: &'a str, shell_args: I, separator: &'a str) -> Handle<'a, process::Child>
where
    I: IntoIterator<Item = &'a str>,
{
    command::<process::Command, _, _>(program, shell_args, separator)
}

/// A testable function that holds the main logic of run().
///
/// Uses dependency injection to allow tests to mock [std::process::Command].
fn command<C, I, S>(program: S, shell_args: I, separator: &str) -> Handle<'_, C::Child>
where
    C: Command,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let child = C::new(program)
        .args(shell_args)
        .stdin(Stdio::piped()) // Stdio::piped is not tested.
        .stdout(Stdio::piped()) // Stdio::piped is not tested.
        .spawn()
        .expect("Shell command failed.");

    Handle::new(child, separator)
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
            let handle = command::<MockCommand, _, _>(program.clone(), shell_args.clone(), ", ");

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
