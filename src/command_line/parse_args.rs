//! Parses args from [args](mod@super::args) into [GroupByOptions](super::options::GroupByOptions).

use crate::command_line::options::*;
use clap::{ArgMatches, Command};
use num::Num;
use regex::Regex;
use std::process;
use std::str::FromStr;

/// Converts a clap::Command into a [GroupByOptions].
pub fn parse(command: Command<'static>) -> GroupByOptions {
    let matches = command.get_matches();

    // Note: clap knows to validate groupings, e.g. "exactly one" or "zero or one" of a given
    // group. The logic below does not need to check for this.

    // Parse input options.
    let input = InputOptions {
        separator: if matches.is_present("input_split_on_whitespace") {
            Separator::Space
        } else if matches.is_present("input_split_on_null") {
            Separator::Null
        } else {
            Separator::Line
        },
    };

    // Parse grouping specifier.
    let grouping = if matches.is_present("group_by_first_chars") {
        let n = parse_numeric_value(&matches, "group_by_first_chars");
        GroupingSpecifier::FirstChars(n)
    } else if matches.is_present("group_by_last_chars") {
        let n = parse_numeric_value(&matches, "group_by_last_chars");
        GroupingSpecifier::LastChars(n)
    } else if matches.is_present("group_by_regex") {
        let re = parse_regex_value(&matches, "group_by_regex");
        GroupingSpecifier::Regex(re)
    } else {
        panic!(
            "No grouping option was specified, but the argument parser didn't catch \
            the issue. Please report this!"
        );
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

        // Parse output only_group_names.
        let only_group_names = matches.is_present("output_only_group_names");

        // Unfortunately, ArgMatches::value_of() returns Option<&str>, but we need
        // Option<String>, so we can't just unwrap.
        let run_command = matches.value_of("output_run_command").map(str::to_string);

        output = OutputOptions {
            separator,
            only_group_names,
            run_command,
        };
    }

    GroupByOptions {
        input,
        grouping,
        output,
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
            eprintln!("Expected a number, but got: {}", s);
            process::exit(1);
        }
    }
}

// Parses a regex value; expects taht the key is present and has a value.
fn parse_regex_value(matches: &ArgMatches, key: &str) -> Regex {
    let pattern = matches.value_of(key).unwrap();
    match Regex::new(pattern) {
        Ok(re) => re,
        Err(e) => {
            eprintln!("{}", e); // The provided messages are actually really good.
            std::process::exit(1);
        }
    }
}
