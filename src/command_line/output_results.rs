use crate::command_line::run_command::{self, *};
use crate::command_line::GroupByOptions;
use crate::grouped_collections::GroupedCollection;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::io::Write;
use std::ops::Deref;
use std::sync::Mutex;

// The environment variable that stores the name of the current shell.
const SHELL_VAR: &str = "SHELL";

pub fn output_results<'a, M, O>(output: O, map: &'a M, options: &'a GroupByOptions)
where
    M: for<'s> GroupedCollection<'s, String, String, Vec<String>>,
    &'a M: IntoParallelIterator<Item = (&'a String, &'a Vec<String>)>,
    O: Write,
{
    if let Some(ref cmd) = options.output.run_command {
        // Instead of outputting results directly, the options say to run a command on each group
        // and print the outputs from those commands.

        let shell_command_options = ShellCommandOptions {
            shell: current_shell(),
            shell_args: shell_args(cmd),
            line_separator: self::line_separator(options),
            only_group_names: options.output.only_group_names,
        };

        let results: BTreeMap<&str, Vec<u8>> = BTreeMap::new();
        let results = run_commands_in_parallel(map, shell_command_options, results);
    }
}

/// Returns the line separator specified in `options`.
fn line_separator(options: &GroupByOptions) -> &str {
    options.output.separator.sep()
}

/// Returns the current shell, e.g. `/usr/bin/zsh`.
// TODO Instead of exiting here, return result so the caller can handle it.
// Retrieve the current shell for later use (if needed).
fn current_shell() -> String {
    std::env::var(SHELL_VAR).unwrap_or_else(|e| {
        eprintln!(
            "Couldn't retrieve environment variable {}: {}",
            SHELL_VAR, e
        );
        std::process::exit(1);
    })
}

// Initialize the shell arguments required to run a command via the current shell.
// TODO Add a command-line option to specify the exact shell invocation.
fn shell_args(cmd: &str) -> Vec<&str> {
    vec!["-c", cmd]
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellCommandOptions<'a> {
    shell: String,
    shell_args: Vec<&'a str>,
    line_separator: &'a str,
    only_group_names: bool,
}

pub fn run_commands_in_parallel<'a, M, R>(map: &'a M, options: ShellCommandOptions, results: R) -> R
where
    M: for<'s> GroupedCollection<'s, String, String, Vec<String>>,
    &'a M: IntoParallelIterator<Item = (&'a String, &'a Vec<String>)>,
    R: Report<'a, Vec<u8>> + Send,
{
    let results = Mutex::new(results);
    map.par_iter().for_each(|(key, value)| {
        let result = run(&options, key, value);
        results.report(key, result);
    });
    results.into_inner().unwrap()
}

pub fn run_commands_sequentially<'a, M, R>(
    map: &'a M,
    options: ShellCommandOptions,
    mut results: R,
) -> R
where
    M: for<'s> GroupedCollection<'s, String, String, Vec<String>>,
    &'a M: IntoParallelIterator<Item = (&'a String, &'a Vec<String>)>,
    R: Report<'a, Vec<u8>>,
{
    // For simplicity, we'll match the format to run_commands_in_parallel.
    map.iter().for_each(|(key, value)| {
        let result = run(&options, key, value);
        results.report(key, result);
    });
    results
}

// Everything inside an iterator; caller can decide whether to run it in parallel or in sequence.
pub fn run<'a>(options: &'a ShellCommandOptions, key: &'a str, values: &'a [String]) -> Vec<u8> {
    // Spawn the new shell process.
    let mut handle = run_command::run(
        &options.shell,
        options.shell_args.iter().map(Deref::deref),
        options.line_separator,
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

    mod line_separator {
        use super::*;

        fn options_for(sep: Separator) -> GroupByOptions {
            GroupByOptions {
                input: InputOptions {
                    separator: Separator::Line,
                },
                grouping: GroupingSpecifier::FirstChars(1),
                output: OutputOptions {
                    separator: sep,
                    only_group_names: false,
                    run_command: None,
                },
            }
        }

        #[test]
        fn returns_correct_line_separator() {
            let options = options_for(Separator::Line);
            assert_eq!(line_separator(&options), "\n");
        }

        #[test]
        fn returns_correct_space_separator() {
            let options = options_for(Separator::Space);
            assert_eq!(line_separator(&options), " ");
        }

        #[test]
        fn returns_correct_null_separator() {
            let options = options_for(Separator::Null);
            assert_eq!(line_separator(&options), "\0");
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

    pub mod run_helpers {
        use super::*;

        // Returns a ShelLCommandOptions for use in run* tests.
        pub fn options<'a>(only_group_names: bool) -> ShellCommandOptions<'a> {
            ShellCommandOptions {
                shell: current_shell(),
                shell_args: shell_args("cat"),
                line_separator: "   ",
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

        pub fn results<'a>() -> BTreeMap<&'a str, Vec<u8>> {
            BTreeMap::new()
        }

        pub fn expected<'a>(map: &'a BTreeMap<String, Vec<String>>) -> BTreeMap<&'a str, Vec<u8>> {
            let mut expected = BTreeMap::new();
            for (key, vector) in map.iter() {
                expected.insert(
                    &key[..],
                    vector
                        .iter()
                        .map(|s| s.to_owned() + "   ")
                        .collect::<String>()
                        .as_bytes()
                        .to_vec(),
                );
            }
            expected
        }
    }

    mod run_commands_in_parallel {
        use super::run_helpers::*;
        use super::*;

        #[test]
        fn returns_correct_results() {
            let map = map();
            let options = options(false);
            let results = results();
            let results = run_commands_in_parallel(&map, options, results);
            let expected = expected(&map);
            assert_eq!(expected, results);
        }
    }

    mod run_commands_sequentially {
        use super::run_helpers::*;
        use super::*;

        #[test]
        fn returns_correct_results() {
            let map = map();
            let options = options(false);
            let results = results();
            let results = run_commands_sequentially(&map, options, results);
            let expected = expected(&map);
            assert_eq!(expected, results);
        }
    }

    mod run {
        use super::run_helpers::*;
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
            let actual = run(&options, &key, &values);
            let actual = String::from_utf8_lossy(&actual);
            assert_eq!(expected, actual);
        }

        #[test]
        fn with_group_contents_works() {
            let options = options(false);
            let (key, values) = kv();

            // By converting values to strings, we make error output much easier to read.
            let expected = "Fido   Sam   Spot   ".to_string();
            let actual = run(&options, &key, &values);
            let actual = String::from_utf8_lossy(&actual);
            assert_eq!(expected, actual);
        }
    }
}
