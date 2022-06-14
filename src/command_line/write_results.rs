//! Outputs the final results from processing a set of inputs according to [GroupByOptions].
//!
//! # Examples
//!
//! ```
//! use groupby::command_line;
//! use groupby::command_line::options::*;
//! use groupby::command_line::write_results::write_results;
//! use groupby::command_line::run_command::run_command;
//! use groupby::grouped_collections::GroupedCollection;
//! use std::collections::BTreeMap;
//!
//! let mut output = vec![];
//!
//! // In a command-line application, we might create this collection via build_groups.
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
//! [GroupByOptions]: crate::command_line::options::GroupByOptions

use crate::command_line::{OutputOptions, RecordWriter, Separator};
use crate::grouped_collections::GroupedCollection;
use std::collections::BTreeMap;
use std::io::Write;

/// The default OutputOptions. These are used when printing results after running commands.
pub const DEFAULT_OUTPUT_OPTIONS: OutputOptions = OutputOptions {
    separator: Separator::Line,
    only_group_names: false,
    run_command: None,
};

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
    use crate::command_line::test_helpers::*;

    mod write_results {
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
