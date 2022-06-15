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
//!     stats: false,
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

/// Builds an [OutputOptions] that uses safe defaults for printing while preserving some options.
///
/// The options reset are specified in the help text in [mod@super::args]. This function is used
/// for printing the results from [mod@super::run_command].
pub fn default_output_options(base: &OutputOptions) -> OutputOptions {
    OutputOptions {
        separator: Separator::Line,
        only_group_names: false,
        run_command: None,
        stats: base.stats,
    }
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
    let default_options = default_output_options(options);
    let options = match results {
        Some(_) => &default_options,
        None => options,
    };

    let separator = options.separator.sep();
    let mut writer = RecordWriter::new(output, separator.as_bytes());

    for (key, values) in map.iter() {
        if options.only_group_names {
            if options.stats {
                writer.write(&format!("{} ({})", key, item_count(values)));
            } else {
                writer.write(key);
            }
        } else {
            // Write header
            if options.stats {
                writer.write(&format!("{}: ({})", key, item_count(values)));
            } else {
                writer.write(&format!("{}:", key));
            }

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

    if options.stats {
        writer.write("");
        writer.write(&statistics_for(map));
    }
}

/// Provides a human-readable description of the length of a vector, like "1 item" or "48 items".
pub fn item_count<_T>(items: &Vec<_T>) -> String {
    if items.len() == 1 {
        "1 item".to_string()
    } else {
        format!("{} items", items.len())
    }
}

/// Reports statistics for a given [GroupedCollection].
pub fn statistics_for<M>(map: &M) -> String
where
    M: for<'s> GroupedCollection<'s, String, String, Vec<String>>,
{
    // We'll reuse this time and time again, so might as well cache it and sort it.
    let mut group_sizes: Vec<usize> = map.iter().map(|(_, items)| items.len()).collect();
    group_sizes.sort_unstable();
    let group_sizes = group_sizes;

    // Total items across all groups.
    let total_items: usize = group_sizes.iter().sum();

    // Number of groups in the collection.
    let total_groups: usize = group_sizes.len();

    // Smallest group size.
    let group_size_min: usize = group_sizes.iter().min().copied().unwrap_or(0);

    // Largest group size.
    let group_size_max: usize = group_sizes.iter().max().copied().unwrap_or(0);

    // Lower median group size.
    let group_size_median: usize = group_sizes.get(group_sizes.len() / 2).copied().unwrap_or(0);

    // Average group size.
    let group_size_average: f64 = if total_groups == 0 {
        0.00
    } else {
        total_items as f64 / total_groups as f64
    };

    format!(
        "Statistics:\n  \
          Total items: {}\n  \
          Total groups: {}\n\
          \n  \
          Group size:\n    \
            Median: {}\n    \
            Average: {:.2}\n    \
            Min: {}\n    \
            Max: {}\n",
        total_items,
        total_groups,
        group_size_median,
        group_size_average,
        group_size_min,
        group_size_max
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_line::options::*;
    use crate::command_line::test_helpers::*;

    mod default_output_options {
        use super::*;

        #[test]
        fn uses_safe_defaults() {
            let unsafe_base = OutputOptions {
                separator: Separator::Null,
                only_group_names: true,
                run_command: Some("command".to_string()),
                stats: false,
            };
            let expected = OutputOptions {
                separator: Separator::Line,
                only_group_names: false,
                run_command: None,
                stats: false,
            };
            assert_eq!(expected, default_output_options(&unsafe_base));
        }

        #[test]
        fn preserves_stats() {
            for val in [false, true] {
                let unsafe_base = OutputOptions {
                    separator: Separator::Null,
                    only_group_names: true,
                    run_command: Some("command".to_string()),
                    stats: val,
                };
                let expected = OutputOptions {
                    separator: Separator::Line,
                    only_group_names: false,
                    run_command: None,
                    stats: val,
                };
                assert_eq!(expected, default_output_options(&unsafe_base));
            }
        }
    }

    mod write_results {
        use super::*;

        fn options_for(only_group_names: bool, stats: bool) -> OutputOptions {
            OutputOptions {
                separator: Separator::Line,
                only_group_names,
                run_command: None,
                stats,
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

        fn statistics_report_for(
            ti: usize,
            tg: usize,
            gmed: usize,
            gavg: f64,
            min: usize,
            max: usize,
        ) -> String {
            format!(
                "Statistics:\n  \
                      Total items: {}\n  \
                      Total groups: {}\n\
                      \n  \
                      Group size:\n    \
                        Median: {}\n    \
                        Average: {:.2}\n    \
                        Min: {}\n    \
                        Max: {}\n",
                ti, tg, gmed, gavg, min, max,
            )
        }

        mod with_results {
            use super::*;

            #[test]
            fn writes_results_using_default_options() {
                let mut output: Vec<u8> = vec![];
                let mut options = options_for(true, false); // write_results should ignore `true`.
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
                let mut options = options_for(false, false);
                options.separator = Separator::Null;
                let map = map();

                write_results(&mut output, &map, &None, &options);

                let expected = "Cats:\0Meowser\0Mittens\0Dogs:\0Lassy\0Buddy\0".to_string();
                let actual = String::from_utf8_lossy(&output);
                assert_eq!(expected, actual);
            }

            mod with_only_group_names {
                use super::*;

                mod with_stats {
                    use super::*;

                    #[test]
                    fn works() {
                        let mut output: Vec<u8> = vec![];
                        let options = options_for(true, true);
                        let map = map();

                        write_results(&mut output, &map, &None, &options);

                        let expected = format!(
                            "Cats (2 items)\n\
                            Dogs (2 items)\n\
                            \n\
                            {}\n",
                            statistics_report_for(4, 2, 2, 2.00, 2, 2),
                        );
                        let actual = String::from_utf8_lossy(&output);
                        assert_eq!(expected, actual);
                    }
                }

                mod without_stats {
                    use super::*;

                    #[test]
                    fn works() {
                        let mut output: Vec<u8> = vec![];
                        let options = options_for(true, false);
                        let map = map();

                        write_results(&mut output, &map, &None, &options);

                        let expected = "Cats\nDogs\n".to_string();
                        let actual = String::from_utf8_lossy(&output);
                        assert_eq!(expected, actual);
                    }
                }
            }

            mod without_only_group_names {
                use super::*;

                mod with_stats {
                    use super::*;

                    #[test]
                    fn works() {
                        let mut output: Vec<u8> = vec![];
                        let options = options_for(false, true);
                        let map = map();

                        write_results(&mut output, &map, &None, &options);

                        let expected = format!(
                            "Cats: (2 items)\n\
                            Meowser\n\
                            Mittens\n\
                            Dogs: (2 items)\n\
                            Lassy\n\
                            Buddy\n\
                            \n\
                            {}\n",
                            statistics_report_for(4, 2, 2, 2.00, 2, 2)
                        );
                        let actual = String::from_utf8_lossy(&output);
                        assert_eq!(expected, actual);
                    }
                }

                mod without_stats {
                    use super::*;

                    #[test]
                    fn works() {
                        let mut output: Vec<u8> = vec![];
                        let options = options_for(false, false);
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

    mod item_count {
        use super::*;

        // Shouldn't happen, but should be correct if it somehow does.
        #[test]
        fn works_with_0_items() {
            let expected = "0 items".to_string();
            let actual = item_count(&Vec::<u8>::new());
            assert_eq!(expected, actual);
        }

        #[test]
        fn works_with_1_item() {
            let expected = "1 item".to_string();
            let actual = item_count(&vec![1]);
            assert_eq!(expected, actual);
        }

        #[test]
        fn works_with_more_than_1_item() {
            for i in 2..4 {
                let expected = format!("{} items", i);
                let actual = item_count(&(0..i).collect());
                assert_eq!(expected, actual);
            }
        }
    }

    mod statistics_for {
        use super::*;

        #[test]
        fn works_with_empty_collection() {
            assert_eq!(
                statistics_for(&BTreeMap::new()),
                "Statistics:\n  \
                  Total items: 0\n  \
                  Total groups: 0\n\
                  \n  \
                  Group size:\n    \
                    Median: 0\n    \
                    Average: 0.00\n    \
                    Min: 0\n    \
                    Max: 0\n",
            );
        }

        #[test]
        fn works_with_integral_average() {
            let mut map = BTreeMap::new();
            map.insert("A".to_string(), vec![]);
            map.insert("B".to_string(), vec!["1".to_string(), "2".to_string()]);
            map.insert("C".to_string(), (1..=4).map(|i| i.to_string()).collect());

            assert_eq!(
                statistics_for(&map),
                "Statistics:\n  \
                  Total items: 6\n  \
                  Total groups: 3\n\
                  \n  \
                  Group size:\n    \
                    Median: 2\n    \
                    Average: 2.00\n    \
                    Min: 0\n    \
                    Max: 4\n",
            );
        }

        #[test]
        fn works_with_rounded_rational_average() {
            let mut map = BTreeMap::new();
            map.insert("A".to_string(), vec![]);
            map.insert("B".to_string(), vec!["1".to_string(), "2".to_string()]);
            map.insert("C".to_string(), (1..=3).map(|i| i.to_string()).collect());

            assert_eq!(
                statistics_for(&map),
                "Statistics:\n  \
                  Total items: 5\n  \
                  Total groups: 3\n\
                  \n  \
                  Group size:\n    \
                    Median: 2\n    \
                    Average: 1.67\n    \
                    Min: 0\n    \
                    Max: 3\n",
            );
        }
    }
}
