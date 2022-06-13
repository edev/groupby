//! Processes a [GroupedCollection] according to [OutputOptions] and outputs the results.
//!
//! # Examples
//!
//! ```
//! use groupby::command_line;
//! use groupby::command_line::options::*;
//! use groupby::command_line::output_results::{run_command, write_results};
//! use groupby::grouped_collections::GroupedCollection;
//! use std::collections::BTreeMap;
//!
//! let mut output = vec![];
//!
//! let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
//! let key = String::from("seasons");
//! let seasons = ["winter", "spring", "summer", "fall"];
//! for season in seasons {
//!     map.add(key.clone(), season.to_string());
//! }
//!
//! let options = OutputOptions {
//!     separator: Separator::Line,
//!     only_group_names: false,
//!     run_command: None,
//! };
//!
//! // If we didn't know that options.run_command would be None, we would call run_command here.
//!
//! command_line::write_results(&mut output, &map, &None, &options);
//!
//! let expected = "seasons:\n\
//!     winter\n\
//!     spring\n\
//!     summer\n\
//!     fall\n".to_string();
//!
//! assert_eq!(expected, String::from_utf8_lossy(&output));
//! ```
//!
//! [OutputOptions]: crate::command_line::options::OutputOptions

use crate::command_line::run_command::{self, *};
use crate::command_line::{OutputOptions, RecordWriter, Separator};
use crate::grouped_collections::GroupedCollection;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::io::Write;
use std::ops::Deref;
use std::sync::Mutex;

/// The environment variable that stores the name of the current shell.
const SHELL_VAR: &str = "SHELL";

/// The default OutputOptions. These are used when printing results after running commands.
const DEFAULT_OUTPUT_OPTIONS: OutputOptions = OutputOptions {
    separator: Separator::Line,
    only_group_names: false,
    run_command: None,
};

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
fn current_shell() -> String {
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
fn shell_args(cmd: &str) -> Vec<&str> {
    vec!["-c", cmd]
}

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
/// use groupby::command_line::output_results::*;
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
    let mut handle = run_command::run(
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

/// Write the final output from processing to a writer.
///
/// Provides the canonical implementation to write fully processed results (a [GroupedCollection]
/// and an optional map of outputs from running commands over the [GroupedCollection]). The rules
/// (with some minor details like punctuation omitted) are as follows:
///
/// - If `results` is a `Some` value, print each group's result instead of its contents, using
///   default options. Otherwise:
///
///   - If `results` is `None` and [OutputOptions::only_group_names] is true, print group headers
///     but not group contents.
///
///   - Write [OutputOptions::separator] after each header and each group member.
///
/// # Relationship between `map` and `results`
///
/// If `results` is a `Some` value, it should have the same set of keys as `map`. This method
/// iterates over `map` and looks up each key from `map` in `results`. As a result, any keys in
/// `results` that are not present in `map` will not be retrieved, and if any keys in `map` are
/// not present in `results`, the method will panic.
///
/// # Panics
///
/// This method panics if a key in `map` is not present in `results`.
pub fn write_results<'a, 'b, M, O>(
    output: O,
    map: &'a M,
    results: &Option<BTreeMap<&'b String, Vec<u8>>>,
    options: &'_ OutputOptions,
) where
    M: for<'s> GroupedCollection<'s, String, String, Vec<String>>,
    O: Write,
{
    let options = match results {
        Some(_) => &DEFAULT_OUTPUT_OPTIONS,
        None => options,
    };

    let separator = options.separator.sep();
    let mut writer = RecordWriter::new(output, separator.as_bytes());

    for (key, values) in map.iter() {
        if options.only_group_names {
            writer.write(key);
        } else {
            // Write header
            writer.write(&format!("{}:", key));

            // If there's a result set (from running a command over each group), write it as the
            // group's output, and do not write the grou's contents. Otherwise, write the group's
            // contents normally.
            if let Some(results) = results {
                let result_utf8 = results.get(key).unwrap();
                let result = String::from_utf8_lossy(result_utf8);
                writer.write(&result);
            } else {
                writer.write_all(values.iter());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_line::options::*;

    pub mod helpers {
        use super::*;

        // Returns a ShelLCommandOptions for use in run* tests.
        pub fn options<'a>(only_group_names: bool) -> ShellCommandOptions<'a> {
            ShellCommandOptions {
                shell: current_shell(),
                shell_args: shell_args("cat"),
                line_separator: "   ".to_string(),
                only_group_names,
            }
        }

        pub fn map() -> BTreeMap<String, Vec<String>> {
            let mut map = BTreeMap::new();
            map.insert(
                "Dogs".to_string(),
                vec!["Lassy".to_string(), "Buddy".to_string()],
            );
            map.insert(
                "Cats".to_string(),
                vec!["Meowser".to_string(), "Mittens".to_string()],
            );
            map
        }

        pub fn results<'a>() -> BTreeMap<&'a String, Vec<u8>> {
            BTreeMap::new()
        }

        pub fn expected_results<'a>(
            map: &'a BTreeMap<String, Vec<String>>,
            separator: &str,
            only_group_names: bool,
        ) -> BTreeMap<&'a String, Vec<u8>> {
            let mut expected = BTreeMap::new();
            for (key, vector) in map.iter() {
                if only_group_names {
                    // Group name plus separator.
                    let value = key.to_owned() + separator;

                    // Convert to Vec<u8> and insert.
                    let value = value.as_bytes().to_vec();
                    expected.insert(key, value);
                } else {
                    expected.insert(
                        key,
                        vector
                            .iter()
                            .map(|s| s.to_owned() + separator)
                            .collect::<String>()
                            .as_bytes()
                            .to_vec(),
                    );
                }
            }
            expected
        }
    }

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
                    use helpers::*;

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
                    use helpers::*;

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
        use super::helpers::*;
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
        use super::helpers::*;
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
        use super::helpers::*;
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

    mod write_results {
        use super::helpers::*;
        use super::*;

        fn options_for(only_group_names: bool) -> OutputOptions {
            OutputOptions {
                separator: Separator::Line,
                only_group_names,
                run_command: None,
            }
        }

        // Constructs a results map where each key's value is its reverse.
        fn results<'a, M>(map: &'a M) -> BTreeMap<&'a String, Vec<u8>>
        where
            M: for<'s> GroupedCollection<'s, String, String, Vec<String>>,
        {
            let mut results = BTreeMap::new();
            for (key, _) in map.iter() {
                let mut output = Vec::<u8>::from(key.clone());
                output.reverse();
                results.insert(key, output);
            }
            results
        }

        mod with_results {
            use super::*;

            #[test]
            fn writes_results_using_default_options() {
                let mut output: Vec<u8> = vec![];
                let mut options = options_for(true); // write_results should ignore `true`.
                options.separator = Separator::Null; // write_results should ignore this.
                let map = map();
                let results = Some(results(&map));

                write_results(&mut output, &map, &results, &options);

                let expected = "Cats:\nstaC\nDogs:\nsgoD\n".to_string();
                let actual = String::from_utf8_lossy(&output);
                assert_eq!(expected, actual);
            }
        }

        mod without_results {
            use super::*;

            #[test]
            fn uses_output_separator() {
                let mut output: Vec<u8> = vec![];
                let mut options = options_for(false);
                options.separator = Separator::Null;
                let map = map();

                write_results(&mut output, &map, &None, &options);

                let expected = "Cats:\0Meowser\0Mittens\0Dogs:\0Lassy\0Buddy\0".to_string();
                let actual = String::from_utf8_lossy(&output);
                assert_eq!(expected, actual);
            }

            mod with_only_group_names {
                use super::*;

                #[test]
                fn works() {
                    let mut output: Vec<u8> = vec![];
                    let options = options_for(true);
                    let map = map();

                    write_results(&mut output, &map, &None, &options);

                    let expected = "Cats\nDogs\n".to_string();
                    let actual = String::from_utf8_lossy(&output);
                    assert_eq!(expected, actual);
                }
            }

            mod without_only_group_names {
                use super::*;

                #[test]
                fn works() {
                    let mut output: Vec<u8> = vec![];
                    let options = options_for(false);
                    let map = map();

                    write_results(&mut output, &map, &None, &options);

                    let expected = "Cats:\nMeowser\nMittens\nDogs:\nLassy\nBuddy\n".to_string();
                    let actual = String::from_utf8_lossy(&output);
                    assert_eq!(expected, actual);
                }
            }
        }
    }
}
