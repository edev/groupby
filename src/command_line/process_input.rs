//! Parses an input stream into a [GroupedCollection].
//!
//! Provides functions to parse an input stream, obeying options in [GroupByOptions], and add
//! parsed tokens into a [GroupedCollection].
//!
//! # Examples
//!
//! ```
//! use groupby::command_line::options::*;
//! use groupby::command_line::process_input::*;
//! use std::collections::HashMap;
//! use std::io::BufReader;
//!
//! let input = BufReader::new("I have some words for you".as_bytes());
//! let mut map = HashMap::new();
//! let options = GroupByOptions {
//!     input: InputOptions {
//!         separator: Separator::Space,
//!     },
//!     grouping: GroupingSpecifier::FirstChars(1),
//!     output: OutputOptions {
//!         separator: Separator::Line,
//!         only_group_names: false,
//!         run_command: None,
//!     },
//! };
//!
//! process_input(input, &mut map, &options);
//! assert_eq!(map.get(&"w".to_string()), Some(&vec!["words".to_string()]));
//! ```

// TODO Use Rayon (?) to provide a parallel input processing function.

use crate::command_line::options::*;
use crate::grouped_collections::GroupedCollection;
use crate::groupers::string::Runner;
use std::io::BufRead;

/// Single-threaded input processing.
pub fn process_input<I, Map>(input: I, map: &mut Map, options: &GroupByOptions)
where
    I: BufRead,
    Map: for<'s> GroupedCollection<'s, String, String, Vec<String>>,
{
    let mut runner = Runner::new(map, &options.grouping);
    match options.input.separator {
        Separator::Null => {
            // Split on null characters and process every resulting token.
            // Note: UTF-8 is designed so the only code point with a null byte is NUL itself,
            // so we won't split a UTF-8 code point by splitting our byte stream before parsing
            // to a String value.
            for result in input.split(0) {
                let token = result.unwrap();
                let token = String::from_utf8(token).unwrap();
                runner.run(token);
            }
        }
        Separator::Space => {
            // Split on whitespace and process every resulting token.
            for line in input.lines() {
                let line = line.unwrap();
                for word in line.split(char::is_whitespace) {
                    // Skip reapted whitespace; split will go character-by-character, so it will
                    // return every second whitespace character in a sequence, which we don't want.
                    if word.chars().all(char::is_whitespace) {
                        continue;
                    }
                    runner.run(word.to_string());
                }
            }
        }
        Separator::Line => {
            // Process each line as a single token.
            for line in input.lines() {
                let line = line.unwrap();
                runner.run(line.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod process_input {
        use super::*;
        use crate::command_line::options::*;
        use crate::grouped_collections::fake_map::*;
        use std::io::BufReader;

        fn works_with(
            input_separator: Separator,
            input: &'static str,
            expected: Vec<&'static str>,
        ) {
            let input: BufReader<&[u8]> = BufReader::new(input.as_bytes());
            let mut map = FakeMap::new();

            // Only input and grouping are relevant; output is unused.
            let options = GroupByOptions {
                input: InputOptions {
                    separator: input_separator,
                },
                grouping: GroupingSpecifier::FirstChars(2000),
                output: OutputOptions {
                    separator: Separator::Line,
                    only_group_names: false,
                    run_command: None,
                },
            };

            process_input(input, &mut map, &options);
            assert_eq!(
                *map.calls(),
                expected
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
            );
        }

        #[test]
        fn works_with_line_separators() {
            works_with(
                Separator::Line,
                "1\n2\n3\n4",
                vec!["1:1", "2:2", "3:3", "4:4"],
            );
        }

        #[test]
        fn works_with_space_separators() {
            works_with(
                Separator::Space,
                "1 2  3     4", // One space, two spaces, and a larger, odd number of spaces.
                vec!["1:1", "2:2", "3:3", "4:4"],
            );
        }

        #[test]
        fn works_with_null_separators() {
            works_with(
                Separator::Null,
                "1\02\03\04",
                vec!["1:1", "2:2", "3:3", "4:4"],
            );
        }
    }
}
