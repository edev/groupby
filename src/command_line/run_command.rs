//! High-level support for running commands over a [GroupedCollection] using [OutputOptions].
//!
//! This module provides layers of abstraction for easy composition and testing of functions. For
//! most use cases, you will probably only need [run_command()].
//!
//! # Module organization in detail
//!
//! [run_command()] provides a top-level entry point. It is the only method you're likely to need
//! as a user of this library.
//!
//! [current_shell()] and [shell_args()] abstract away the details of setting up a shell to run a
//! command. Both functions are trivial.
//!
//! [run_commands_in_parallel()] and [run_commands_sequentially()] take a shell command
//! configuration (specified using a [ShellCommandOptions]) and a [GroupedCollection] and run the
//! command over each group, building a [Report] with each group's standard output.
//!
//! [capture_command_output] runs a single shell command and captures its output. This function, in
//! turn, uses [command_runner::run()] to run the shell command.

use crate::command_line::command_runner::{self, *};
use crate::command_line::OutputOptions;
use crate::grouped_collections::GroupedCollection;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Mutex;

/// The environment variable that stores the name of the current shell.
const SHELL_VAR: &str = "SHELL";

/// Options needed for running a shell command over a group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellCommandOptions<'a> {
    /// The path to the shell, e.g. `/usr/bin/zsh`.
    pub shell: String,

    /// The arguments to pass to the shell, one per item in the [Vec], e.g. `vec!["-c", "do_thing |
    /// tail -n 4"]`
    pub shell_args: Vec<&'a str>,

    /// The string that should separate values passed to the command's standard input, e.g. `"\n"`.
    pub line_separator: String,

    /// If true, pass only the group's key followed by `line_separator` via the command's standard
    /// input.
    ///
    /// If false, for each value in the group, write the value followed by `line_separator` to the
    /// command's standard input.
    pub only_group_names: bool,
}

/// Runs commands over a [GroupedCollection], if requested by [OutputOptions].
///
/// If [OutputOptions::run_command] is `None`, returns `None` without doing anything else.
/// Otherwise, runs the command over each group, using the provided options, and returns a
/// [BTreeMap] mapping `map`'s keys to the captured standard output of each group's command.
///
/// If `parallel` is `true`, runs commands in parallel across all available CPU cores. If `false`,
/// runs one command at a time. Note that sequential commands run according to the key sort order,
/// whereas parallel commands may run in arbitrary order.
pub fn run_command<'a, M>(
    map: &'a M,
    options: &OutputOptions,
    parallel: bool,
) -> Option<BTreeMap<&'a String, Vec<u8>>>
where
    M: for<'s> GroupedCollection<'s, String, String, Vec<String>>,
    &'a M: IntoParallelIterator<Item = (&'a String, &'a Vec<String>)>,
{
    // Get the command to run, e.g. $SHELL -c "command", or return None.
    let command: &String = options.run_command.as_ref()?;

    // Set up the options our command runner needs.
    let shell_command_options = ShellCommandOptions {
        shell: current_shell(),
        shell_args: shell_args(command),
        line_separator: options.separator.sep(),
        only_group_names: options.only_group_names,
    };

    // Run commands and capture standard output in a BTreeMap.
    let results = BTreeMap::new();
    let results = if parallel {
        run_commands_in_parallel(map, shell_command_options, results)
    } else {
        run_commands_sequentially(map, shell_command_options, results)
    };

    Some(results)
}

/// Returns the current shell, e.g. `/usr/bin/zsh`.
///
/// # Panics
///
/// Exits with an error if it can't retrieve the current shell. This is because the function is
/// meant only for internal use in the context of [run_command()], which is a top-level convenience
/// method. A library user who prefers to handle this differently is free to invoke either
/// [run_commands_in_parallel] or [run_commands_sequentially] directly and provide their own
/// wrapping code.
pub fn current_shell() -> String {
    std::env::var(SHELL_VAR).unwrap_or_else(|e| {
        eprintln!(
            "Couldn't retrieve environment variable {}: {}",
            SHELL_VAR, e
        );
        std::process::exit(1);
    })
}

/// Initializes the shell arguments required to run a command via the current shell.
///
/// Note that this function is not smart in any way; it simply assumes that the shell accepts a
/// `-c` argument followed by an argument containing a command string to interpret.
pub fn shell_args(cmd: &str) -> Vec<&str> {
    vec!["-c", cmd]
}

/// Runs commands over groups in parallel.
///
/// Runs the command specified by `options` once per group. See [capture_command_output()] for
/// details on how the command is run.
///
/// This version uses [Rayon](rayon) to run as many commands at a time as there are logical CPU
/// cores. For a single-threaded version, see [run_commands_sequentially].
pub fn run_commands_in_parallel<'a, M, R>(map: &'a M, options: ShellCommandOptions, results: R) -> R
where
    M: for<'s> GroupedCollection<'s, String, String, Vec<String>>,
    &'a M: IntoParallelIterator<Item = (&'a String, &'a Vec<String>)>,
    R: Report<&'a String, Vec<u8>> + Send,
{
    let results = Mutex::new(results);
    map.par_iter().for_each(|(key, value)| {
        let result = capture_command_output(&options, key, value);
        results.report(key, result);
    });
    results.into_inner().unwrap()
}

/// Runs commands over groups, one at a time.
///
/// Runs the command specified by `options` once per group. See [capture_command_output()] for
/// details on how the command is run.
///
/// This version is single-threaded, running only one command at a time. For a multi-threaded
/// version, see [run_commands_in_parallel].
pub fn run_commands_sequentially<'a, M, R>(
    map: &'a M,
    options: ShellCommandOptions,
    mut results: R,
) -> R
where
    M: for<'s> GroupedCollection<'s, String, String, Vec<String>>,
    &'a M: IntoParallelIterator<Item = (&'a String, &'a Vec<String>)>,
    R: Report<&'a String, Vec<u8>>,
{
    // For simplicity, we'll match the format to run_commands_in_parallel.
    map.iter().for_each(|(key, value)| {
        let result = capture_command_output(&options, key, value);
        results.report(key, result);
    });
    results
}

/// Runs a shell command against a single group and returns its captured output.
///
/// Runs the command specified by `options` once. Depending on `options.only_group_names`, it will
/// pass either the group's `key` or the group's `values` to the command via standard input. In
/// either case, each item passed to the group is followed by `options.line_separator`.
///
/// This is meant to sit on the inside of an iterator of the user's choice.
/// [run_commands_in_parallel] and [run_commands_sequentially] essentially wrap this function in
/// different iterators to provide the user with multiple execution strategies.
///
/// # Returns
///
/// The captured standard output from the command. Standard error is not captured but is instead
/// written to the standard error inherited from the caller.
///
/// # Examples
///
/// ```
/// use groupby::command_line::run_command::*;
///
/// let options = ShellCommandOptions {
///     shell: "/usr/bin/bash".to_string(),
///     shell_args: vec!["-c", "cat"],
///     line_separator: "\n".to_string(),
///     only_group_names: false,
/// };
///
/// let key = "ABCs";
/// let values: Vec<String> = ["a", "b", "c"]
///     .iter()
///     .map(ToString::to_string)
///     .collect();
///
/// let output = capture_command_output(&options, &key, &values);
/// assert_eq!(&String::from_utf8_lossy(&output), "a\nb\nc\n");
/// ```
pub fn capture_command_output<'a>(
    options: &'a ShellCommandOptions,
    key: &'a str,
    values: &'a [String],
) -> Vec<u8> {
    // Spawn the new shell process.
    let mut handle = command_runner::run(
        &options.shell,
        options.shell_args.iter().map(Deref::deref),
        &options.line_separator,
    );

    // Pass along the group's contents (or name, if output.only_group_names) via stdin.
    if options.only_group_names {
        handle.stdin.write(key);
    } else {
        handle.stdin.write_all(values.iter());
    }

    // Wait for the process to finish, then record its output so we can print it later.
    let output = handle.wait_with_output().unwrap();
    output.stdout
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_line::options::*;
    use crate::command_line::test_helpers::*;

    mod run_command {
        use super::*;

        fn options_for(run_command: Option<String>, only_group_names: bool) -> GroupByOptions {
            GroupByOptions {
                input: InputOptions {
                    separator: Separator::Line,
                },
                grouping: GroupingSpecifier::FirstChars(1),
                output: OutputOptions {
                    separator: Separator::Line,
                    only_group_names,
                    run_command,
                    stats: false,
                },
            }
        }

        fn verify_results(left: &BTreeMap<&String, Vec<u8>>, right: &BTreeMap<&String, Vec<u8>>) {
            // Ensure that the keys are the same
            let left_keys = left.keys().collect::<Vec<&&String>>();
            let right_keys = right.keys().collect::<Vec<&&String>>();
            assert_eq!(left_keys, right_keys);

            // Ensure that the values for each key are the same and ensure that we use Unicode
            // Strings for clear error output.
            for (key, left) in left.iter() {
                let right = right.get(key).unwrap();
                assert_eq!(
                    String::from_utf8_lossy(left),
                    String::from_utf8_lossy(right),
                );
            }
        }

        mod with_run_command {
            use super::*;

            // Tests should run both parallel and sequential variants, since it's trivial. They
            // should be equal.
            mod with_or_without_parallel {
                use super::*;

                mod with_only_group_names {
                    use super::*;

                    #[test]
                    fn outputs_only_group_names() {
                        let map = map();
                        let options = options_for(Some(String::from("cat")), true);

                        let expected =
                            expected_results(&map, &options.output.separator.sep(), true);
                        let sequential_results = run_command(&map, &options.output, false);
                        let parallel_results = run_command(&map, &options.output, true);

                        verify_results(&expected, &sequential_results.as_ref().unwrap());
                        verify_results(&expected, &parallel_results.as_ref().unwrap());
                    }
                }

                mod without_only_group_names {
                    use super::*;

                    #[test]
                    fn outputs_results_correctly() {
                        let map = map();
                        let options = options_for(Some(String::from("cat")), false);

                        let expected =
                            expected_results(&map, &options.output.separator.sep(), false);
                        let sequential_results = run_command(&map, &options.output, false);
                        let parallel_results = run_command(&map, &options.output, true);

                        verify_results(&expected, &sequential_results.as_ref().unwrap());
                        verify_results(&expected, &parallel_results.as_ref().unwrap());
                    }

                    #[test]
                    fn uses_output_separator_correctly() {
                        let map = map();
                        let mut options = options_for(Some(String::from("cat")), false);
                        options.output.separator = Separator::Space;

                        let expected =
                            expected_results(&map, &options.output.separator.sep(), false);
                        let sequential_results = run_command(&map, &options.output, false);
                        let parallel_results = run_command(&map, &options.output, true);

                        verify_results(&expected, &sequential_results.as_ref().unwrap());
                        verify_results(&expected, &parallel_results.as_ref().unwrap());
                    }
                }
            }
        }
    }

    mod current_shell {
        use super::*;

        #[test]
        fn returns_current_shell() {
            // A cursory test will suffice here. Over-complicating things by swapping out the
            // environment variable for the running test probably doesn't make much sense.
            let expected = std::env::var(SHELL_VAR).unwrap();
            assert_eq!(expected, current_shell());
        }
    }

    mod shell_args {
        use super::*;

        #[test]
        fn returns_reasonable_shell_args() {
            // Some reasonably complex command that would require a shell to interpret it.
            let cmd = "head < /dev/random | uniq >> random_strings.txt";

            let expected = vec!["-c", cmd];
            assert_eq!(expected, shell_args(cmd));
        }
    }

    mod run_commands_in_parallel {
        use super::*;

        #[test]
        fn returns_correct_results() {
            let map = map();
            let options = options(false);
            let results = results();
            let results = run_commands_in_parallel(&map, options, results);
            let expected = expected_results(&map, "   ", false);
            assert_eq!(expected, results);
        }
    }

    mod run_commands_sequentially {
        use super::*;

        #[test]
        fn returns_correct_results() {
            let map = map();
            let options = options(false);
            let results = results();
            let results = run_commands_sequentially(&map, options, results);
            let expected = expected_results(&map, "   ", false);
            assert_eq!(expected, results);
        }
    }

    mod capture_command_output {
        use super::*;

        fn kv<'a>() -> (&'static str, Vec<String>) {
            (
                "dogs",
                vec!["Fido".to_string(), "Sam".to_string(), "Spot".to_string()],
            )
        }

        #[test]
        fn with_only_group_names_works() {
            let options = options(true);
            let (key, values) = kv();

            // By converting values to strings, we make error output much easier to read.
            let expected = "dogs   ".to_string();
            let actual = capture_command_output(&options, &key, &values);
            let actual = String::from_utf8_lossy(&actual);
            assert_eq!(expected, actual);
        }

        #[test]
        fn with_group_contents_works() {
            let options = options(false);
            let (key, values) = kv();

            // By converting values to strings, we make error output much easier to read.
            let expected = "Fido   Sam   Spot   ".to_string();
            let actual = capture_command_output(&options, &key, &values);
            let actual = String::from_utf8_lossy(&actual);
            assert_eq!(expected, actual);
        }
    }
}
