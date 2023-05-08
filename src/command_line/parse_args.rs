//! Parses args from [args](mod@super::args) into [GroupByOptions](super::options::GroupByOptions).

use crate::command_line::options::*;
use clap::{ArgMatches, Command};
use num::Num;
use regex::{self, Regex};
use std::str::FromStr;

// A testable function that holds the main logic of parse().
fn parse_from<M>(command: Command<'static>, matcher: M) -> GroupByOptions
where
    M: FnOnce(Command<'static>) -> ArgMatches,
{
    let matches = matcher(command);

    // Note: clap knows to validate groupings, e.g. "exactly one" or "zero or one" of a given
    // group. The logic below does not need to check for this.

    // Parse input options.
    let input = InputOptions {
        separator: if matches.is_present("input_split_on_whitespace") {
            Separator::Space
        } else if matches.is_present("input_split_on_null") {
            Separator::Null
        } else if matches.is_present("input_split_on_custom") {
            let s = matches
                .value_of("input_split_on_custom")
                .unwrap()
                .to_string();
            Separator::Custom(s)
        } else {
            Separator::Line
        },
    };

    // Dummy match statement. If you're seeing an error here, you probably just added a Separator
    // variant. This error is meant to remind you to add logic for your new separator to the block
    // just above this comment (if appropriate). Otherwise, command-line arguments won't actually
    // translate into GroupByOptions, even though all tests might very well pass!
    match Separator::Space {
        Separator::Space => (),
        Separator::Null => (),
        Separator::Custom(_) => (),
        Separator::Line => (),
    };

    // Parse grouping specifier.
    let grouping = if matches.is_present("groupers_by_first_chars") {
        let n = parse_numeric_value(&matches, "groupers_by_first_chars");
        GroupingSpecifier::FirstChars(n)
    } else if matches.is_present("groupers_by_last_chars") {
        let n = parse_numeric_value(&matches, "groupers_by_last_chars");
        GroupingSpecifier::LastChars(n)
    } else if matches.is_present("groupers_by_regex") {
        let re = parse_regex_value(&matches, "groupers_by_regex");
        let cg = parse_capture_group(&matches);
        GroupingSpecifier::Regex(re, cg)
    } else if matches.is_present("groupers_by_file_extension") {
        GroupingSpecifier::FileExtension
    } else if matches.is_present("groupers_by_counter") {
        GroupingSpecifier::Counter
    } else {
        panic!(
            "No grouping option was specified, but the argument parser didn't catch \
            the issue. Please report this!"
        );
    };

    // Dummy match statement. If you're seeing an error here, you probably just added a
    // GroupingSpecifier variant. This error is meant to remind you to add logic for your new
    // grouping specifier to the block just above this comment. Otherwise, command-line arguments
    // won't actually translate into GroupByOptions, even though all tests might very well pass!
    match GroupingSpecifier::FirstChars(4) {
        GroupingSpecifier::FirstChars(_) => (),
        GroupingSpecifier::LastChars(_) => (),
        GroupingSpecifier::Regex(_, _) => (),
        GroupingSpecifier::FileExtension => (),
        GroupingSpecifier::Counter => (),
    };

    // Parse output options. The nested scope prevents name confusion with nested options.
    let output;
    {
        let separator = if matches.is_present("output_space_separators") {
            Separator::Space
        } else if matches.is_present("output_null_separators") {
            Separator::Null
        } else {
            Separator::Line
        };

        let only_group_names = matches.is_present("output_only_group_names");

        // Unfortunately, ArgMatches::value_of() returns Option<&str>, but we need
        // Option<String>, so we can't just unwrap.
        let run_command = matches.value_of("output_run_command").map(str::to_string);

        let parallel = !matches.is_present("output_sequential");

        let headers = !matches.is_present("output_no_headers");

        let stats = matches.is_present("output_stats");

        output = OutputOptions {
            separator,
            only_group_names,
            run_command,
            parallel,
            headers,
            stats,
        };
    }

    // Dummy match statement. If you're seeing an error here, you probably just added a Separator
    // variant. This error is meant to remind you to add logic for your new separator to the block
    // just above this comment (if appropriate). Otherwise, command-line arguments won't actually
    // translate into GroupByOptions, even though all tests might very well pass!
    match Separator::Space {
        Separator::Space => (),
        Separator::Null => (),
        Separator::Custom(_) => (),
        Separator::Line => (),
    };

    GroupByOptions {
        input,
        grouping,
        output,
    }
}

/// Converts a clap::Command into a [GroupByOptions].
pub fn parse(command: Command<'static>) -> GroupByOptions {
    // parse() wraps parse_from() so we can use dependency injection for testing.
    parse_from(command, |c| c.get_matches())
}

// Parses the capture group option, defaulting to 0 if not present.
//
// The capture group can be a number or a name, so if it doesn't parse as a usize, we'll assume
// it's a name.
fn parse_capture_group(matches: &ArgMatches) -> CaptureGroup {
    match matches.value_of("grouper_options_capture_group") {
        Some(s) => match s.parse() {
            Ok(n) => CaptureGroup::Number(n),
            Err(_) => CaptureGroup::Name(s.to_string()),
        },
        None => CaptureGroup::Number(0),
    }
}

// Parses a key with a numeric value; expects that the key is present and has a value.
fn parse_numeric_value<T>(matches: &ArgMatches, key: &str) -> T
where
    T: Num + FromStr,
{
    let s = matches.value_of(key).unwrap();
    match s.parse() {
        Ok(n) => n,
        Err(_) => {
            panic!("Expected a number, but got: {}", s);
        }
    }
}

// Parses a regex value; expects that the key is present and has a value.
fn parse_regex_value(matches: &ArgMatches, key: &str) -> Regex {
    let pattern = matches.value_of(key).unwrap();
    Regex::new(pattern).unwrap() // The provided messages are actually really good.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_line::args::CommandBuilder;
    use clap::command;
    use std::fmt::Debug;

    fn cb() -> CommandBuilder {
        CommandBuilder::new(command!())
    }

    #[cfg(test)]
    mod parse_from {
        use super::*;
        use crate::command_line::args;

        fn parses<S, T>(args: &Vec<&'static str>, selector: S, expected: T)
        where
            S: FnOnce(GroupByOptions) -> T,
            T: Eq + Debug,
        {
            let command = args::args();
            let options =
                crate::command_line::parse_args::parse_from(command, |c| c.get_matches_from(args));
            let parsed_value: T = selector(options);
            assert_eq!(expected, parsed_value);
        }

        #[test]
        fn parses_input_split_on_whitespace() {
            // Short
            parses(
                &vec!["app", "-w", "-f1"],
                |gbo: GroupByOptions| gbo.input.separator,
                Separator::Space,
            );
            // No long option
        }

        #[test]
        fn parses_input_split_on_null() {
            // Short
            parses(
                &vec!["app", "-0", "-f1"],
                |gbo: GroupByOptions| gbo.input.separator,
                Separator::Null,
            );
            // No long option
        }

        #[test]
        fn parses_input_split_on_custom() {
            // No short option

            // Long
            parses(
                &vec!["app", "--split", "ZyX", "-f1"],
                |gbo: GroupByOptions| gbo.input.separator,
                Separator::Custom("ZyX".to_string()),
            );
        }

        #[test]
        fn parses_input_split_default() {
            parses(
                &vec!["app", "-f1"],
                |gbo: GroupByOptions| gbo.input.separator,
                Separator::Line,
            );
        }

        #[test]
        fn parses_groupers_by_first_chars() {
            // Short
            parses(
                &vec!["app", "-w", "-f8"],
                |gbo: GroupByOptions| gbo.grouping,
                GroupingSpecifier::FirstChars(8),
            );
            // No long option
        }

        #[test]
        fn parses_groupers_by_last_chars() {
            // Short
            parses(
                &vec!["app", "-w", "-l9"],
                |gbo: GroupByOptions| gbo.grouping,
                GroupingSpecifier::LastChars(9),
            );
        }

        #[test]
        fn parses_groupers_by_regex() {
            // Short
            parses(
                &vec!["app", "-w", "-r", "foo"],
                |gbo: GroupByOptions| gbo.grouping,
                GroupingSpecifier::Regex(Regex::new("foo").unwrap(), CaptureGroup::Number(0)),
            );

            // Long
            parses(
                &vec!["app", "-w", "--regex", "bar"],
                |gbo: GroupByOptions| gbo.grouping,
                GroupingSpecifier::Regex(Regex::new("bar").unwrap(), CaptureGroup::Number(0)),
            );
        }

        #[test]
        fn parses_groupers_by_file_extension() {
            // No short option

            // Long
            parses(
                &vec!["app", "-w", "--extension"],
                |gbo: GroupByOptions| gbo.grouping,
                GroupingSpecifier::FileExtension,
            );
        }

        #[test]
        fn parses_groupers_by_counter() {
            // No short option

            // Long
            parses(
                &vec!["app", "-w", "--counter"],
                |gbo: GroupByOptions| gbo.grouping,
                GroupingSpecifier::Counter,
            );
        }

        #[test]
        fn parses_output_null_separators() {
            // No short option

            // Long
            parses(
                &vec!["app", "--print0", "-f1"],
                |gbo: GroupByOptions| gbo.output.separator,
                Separator::Null,
            );
        }

        #[test]
        fn parses_output_space_separators() {
            // No short option

            // Long
            parses(
                &vec!["app", "--printspace", "-f1"],
                |gbo: GroupByOptions| gbo.output.separator,
                Separator::Space,
            );
        }

        #[test]
        fn parses_output_default_separators() {
            parses(
                &vec!["app", "-f1"],
                |gbo: GroupByOptions| gbo.output.separator,
                Separator::Line,
            );
        }

        #[test]
        fn parses_output_only_group_names() {
            // No short option

            // Long
            parses(
                &vec!["app", "--only-group-names", "-f1"],
                |gbo: GroupByOptions| gbo.output.only_group_names,
                true,
            );

            // When not specified
            parses(
                &vec!["app", "-f1"],
                |gbo: GroupByOptions| gbo.output.only_group_names,
                false,
            );
        }

        #[test]
        fn parses_output_run_command() {
            // Short
            parses(
                &vec!["app", "-c", "tail | head", "-f1"],
                |gbo: GroupByOptions| gbo.output.run_command,
                Some("tail | head".to_string()),
            );
            // No long option

            // When not specified
            parses(
                &vec!["app", "-f1"],
                |gbo: GroupByOptions| gbo.output.run_command,
                None,
            );
        }

        #[test]
        fn parses_output_sequential() {
            // No short option

            // Long
            parses(
                &vec!["app", "--sequential", "-f1"],
                |gbo: GroupByOptions| gbo.output.parallel,
                false,
            );
            parses(
                &vec!["app", "-f1"],
                |gbo: GroupByOptions| gbo.output.parallel,
                true,
            );
        }

        #[test]
        fn parses_output_no_headers() {
            // No short option

            // Long
            parses(
                &vec!["app", "-f1"],
                |gbo: GroupByOptions| gbo.output.headers,
                true,
            );
            parses(
                &vec!["app", "--no-headers", "-f1"],
                |gbo: GroupByOptions| gbo.output.headers,
                false,
            );
        }

        // TODO Write missing test parses_stats
    }

    #[cfg(test)]
    mod parse_capture_group {
        use super::*;

        #[test]
        fn returns_default_if_option_is_missing() {
            let clap = cb()
                .groupers_by_regex()
                .grouper_options_capture_group()
                .command;
            let args = vec!["appname", "--regex", "xeger--"];
            let matches = clap.get_matches_from(args);
            let result = parse_capture_group(&matches);
            assert_eq!(CaptureGroup::Number(0), result);
        }

        #[test]
        fn returns_number_on_success() {
            let clap = cb()
                .groupers_by_regex()
                .grouper_options_capture_group()
                .command;
            let args = vec!["appname", "--regex", "xeger--", "--capture-group", "4"];
            let matches = clap.get_matches_from(args);
            let result = parse_capture_group(&matches);
            assert_eq!(CaptureGroup::Number(4), result);
        }

        #[test]
        fn returns_string_on_failure() {
            let clap = cb()
                .groupers_by_regex()
                .grouper_options_capture_group()
                .command;
            let args = vec!["appname", "--regex", "xeger--", "--capture-group", "four"];
            let matches = clap.get_matches_from(args);
            let result = parse_capture_group(&matches);
            assert_eq!(CaptureGroup::Name("four".to_string()), result);
        }

        #[test]
        fn returns_string_on_number_embedded_in_text() {
            let clap = cb()
                .groupers_by_regex()
                .grouper_options_capture_group()
                .command;
            let args = vec!["appname", "--regex", "xeger--", "--capture-group", "20four"];
            let matches = clap.get_matches_from(args);
            let result = parse_capture_group(&matches);
            assert_eq!(CaptureGroup::Name("20four".to_string()), result);
        }
    }

    #[cfg(test)]
    mod parse_numeric_value {
        use super::*;

        #[test]
        fn returns_number() {
            let clap = cb().groupers_by_first_chars().command;
            let args = vec!["appname", "-f", "4"];
            let matches = clap.get_matches_from(args);
            assert_eq!(4, parse_numeric_value(&matches, "groupers_by_first_chars"));
        }

        #[test]
        #[should_panic]
        fn panics_on_failed_parse() {
            let clap = cb().groupers_by_first_chars().command;
            let args = vec!["appname", "-f", "four"];
            let matches = clap.get_matches_from(args);
            parse_numeric_value::<usize>(&matches, "groupers_by_first_chars");
        }
    }

    #[cfg(test)]
    mod parse_regex_value {
        use super::*;

        #[test]
        fn returns_matching_regex() {
            let clap = CommandBuilder::new(command!()).groupers_by_regex().command;
            let args = vec!["appname", "-r", "(foo)?bar"];
            let matches = clap.get_matches_from(args);
            let re = parse_regex_value(&matches, "groupers_by_regex");
            assert!(re.is_match("bar"));
            assert!(re.is_match("foobar"));
            assert!(!re.is_match("soap"));
        }

        #[test]
        #[should_panic(expected = "unclosed group")]
        fn panics_on_invalid_regex() {
            let clap = CommandBuilder::new(command!()).groupers_by_regex().command;
            let invalid_args = vec!["appname", "-r", "(foo"];
            let matches = clap.get_matches_from(invalid_args);
            parse_regex_value(&matches, "groupers_by_regex"); // Should panic.
        }
    }
}
