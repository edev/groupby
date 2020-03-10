use clap::{crate_authors, crate_version, App, Arg, ArgGroup};
use groupby::*;
use regex::Regex;
use std::io;
use std::io::{BufRead, BufWriter, Write};
use std::process::{Command, Stdio};

// The environment variable that stores the name of the current shell.
const SHELL_VAR: &str = "SHELL";

fn main() {
    let mut grouped_collection = GroupedCollection::<String, String>::new();
    let options = args();

    process_input(&mut grouped_collection, &options);
    output_results(&grouped_collection, &options);
}

// Type definitions for handling command-line arguments.

// An option that is either set or not set, i.e. a flag.
#[derive(PartialEq, Copy, Clone)]
enum Flag {
    Set,
    Unset,
}

// Note: optional arguments that take a value are Option types.

// Input options.
struct InputOptions {
    split_on_whitespace: Flag,
}

// Grouping options. These are mutually exclusive, and exactly one must be set.
enum GroupingOptions {
    ByFirstChars(usize),
    ByLastChars(usize),
    ByRegex(Regex),
}

// Output options. None, any, or all may be set.
struct OutputOptions {
    null_separators: Flag,
    space_separators: Flag,
    only_group_names: Flag,
    run_command: Option<String>,
}

// The options struct that holds all of these options.
// Note: for safety, users are strongly recommended to own such a struct immutably.
struct GroupByOptions {
    input: InputOptions,
    grouping: GroupingOptions,
    output: OutputOptions,
}

// Use clap to parse command-line arguments.
fn args<'a>() -> GroupByOptions {
    let matches = App::new("Groupby")
        // Basic app info.
        .author(crate_authors!())
        .version(crate_version!())
        .long_about(
            "Reads lines from standard input and groups them by common substrings. Prints the resulting groups to standard output unless -c is used.\n\
             \n\
             One and only one grouping option must be specified.",
        )

        // Input arguments.
        .arg(
            Arg::with_name("InputSplitOnWhitespace")
                .short("w")
                .help("Group words instead of lines; that is, split input on whitespace.")
        )

        // Grouping arguments.
        .arg(
            Arg::with_name("GroupByFirstChars")
                .short("f")
                .value_name("n")
                .help("Group by equivalence on the first n characters.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("GroupByLastChars")
                .short("l")
                .value_name("n")
                .help("Group by equivalence on the last n characters.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("GroupByRegex")
                .short("r")
                .long("regex")
                .value_name("pattern")
                .help(
                    "Group by equivalence on the first match against the specified regex pattern. \
                     If capture groups are present, group by equivalence on the first capture \
                     group. If a line does not match, it is stored in the blank group, \"\"."
                 )
                .takes_value(true)
        )

        // Add grouping arguments to a Clap ArgGroup.
        .group(
            ArgGroup::with_name("grouping")
                .required(true)
                .args(&[
                    "GroupByFirstChars",
                    "GroupByLastChars",
                    "GroupByRegex"
                ]),
        )

        // Output arguments.
        .arg(
            Arg::with_name("OutputNullSeparators")
                .long("print0")
                .help("When outputting lines, separate them with a null character rather than a newline. This option is meant for compatibility with xargs -0.")
        )
        .arg(
            Arg::with_name("OutputSpaceSeparators")
                .long("printspace")
                .help("When outputting lines, separate them with a space rather than a newline.")
        )
        .arg(
            Arg::with_name("OutputOnlyGroupNames")
                .long("matches")
                .help("Instead of outputting lines, output the matched text that forms each group.")
        )
        .arg(
            Arg::with_name("OutputRunCommand")
                .short("c")
                .value_name("cmd")
                .help("Execute command cmd for each group, passing the group via standard input, one match per line.")
                .takes_value(true),
        )

        // Retrieve the actual arguments provided by the user.
        .get_matches();

    // Now process the arguments and construct the object to return.
    GroupByOptions {
        input: InputOptions {
            split_on_whitespace: match matches.is_present("InputSplitOnWhitespace") {
                true => Flag::Set,
                false => Flag::Unset,
            },
        },
        grouping: if let Some(n) = matches.value_of("GroupByFirstChars") {
            match n.parse::<usize>() {
                Ok(n) => GroupingOptions::ByFirstChars(n),
                Err(_) => {
                    eprintln!("Error: {} is not a whole number.", n);
                    std::process::exit(1);
                }
            }
        } else if let Some(n) = matches.value_of("GroupByLastChars") {
            match n.parse::<usize>() {
                Ok(n) => GroupingOptions::ByLastChars(n),
                Err(_) => {
                    eprintln!("Error: {} is not a whole number.", n);
                    std::process::exit(1);
                }
            }
        } else if let Some(pattern) = matches.value_of("GroupByRegex") {
            match Regex::new(pattern) {
                Ok(re) => GroupingOptions::ByRegex(re),
                Err(e) => {
                    eprintln!("{}", e); // The provided messages are actually really good.
                    std::process::exit(1);
                }
            }
        } else {
            panic!("No grouping option was specified, but the argument parser didn't catch the issue. Please report this!")
        },
        output: OutputOptions {
            null_separators: match matches.is_present("OutputNullSeparators") {
                true => Flag::Set,
                false => Flag::Unset,
            },
            space_separators: match matches.is_present("OutputSpaceSeparators") {
                true => Flag::Set,
                false => Flag::Unset,
            },
            only_group_names: match matches.is_present("OutputOnlyGroupNames") {
                true => Flag::Set,
                false => Flag::Unset,
            },
            run_command: if let Some(cmd) = matches.value_of("OutputRunCommand") {
                Some(cmd.to_string())
            } else {
                None
            },
        },
    }
}

fn process_input(
    grouped_collection: &mut GroupedCollection<String, String>,
    options: &GroupByOptions,
) {
    // Process input.

    // Extract the grouping function to use so that we only perform this logic once
    // rather than for each line.
    let mut grouping_function: Box<dyn FnMut(String)> = match &options.grouping {
        // Moving/reference notes:
        // TLDR: Everything we move is a reference, so no meaningful ownership changes occur.
        // Details:
        // 1. We need to use "move" closures to move the borrowed references n into the closure,
        //    or else they will go out of scope.
        // 2. grouped_collection is intentionally a mutable reference specifically intended to be
        //    moved into whichever closure we construct.
        // 3. We move re into the ByRegex closure, but it, too, is a reference, so we are not
        //    partially moving anything out of GroupByOptions.
        GroupingOptions::ByFirstChars(n) => {
            Box::new(move |s| grouped_collection.group_by_first_chars(s, *n))
        }
        GroupingOptions::ByLastChars(n) => {
            Box::new(move |s| grouped_collection.group_by_last_chars(s, *n))
        }
        GroupingOptions::ByRegex(re) => Box::new(move |s| grouped_collection.group_by_regex(s, re)),
    };

    // Process each line of input.
    let stdin = io::stdin();
    if options.input.split_on_whitespace == Flag::Set {
        // Split on whitespace and process every resulting token.
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            for word in line.split(char::is_whitespace) {
                // Skip whitespace; split will go character-by-character, so it will catch every
                // other whitespace character, which we don't want.
                if word.chars().all(|c| c.is_whitespace()) {
                    continue;
                }
                grouping_function(word.to_string());
            }
        }
    } else {
        // Process each line as a single token.
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            grouping_function(line.clone());
        }
    }
}

fn output_results(
    grouped_collection: &GroupedCollection<String, String>,
    options: &GroupByOptions,
) {
    // Determine what line separator the user wants.
    let line_separator = if options.output.null_separators == Flag::Set {
        "\0"
    } else if options.output.space_separators == Flag::Set {
        " "
    } else {
        "\n"
    };

    // Generate the required outputs.
    if let Some(cmd) = &options.output.run_command {
        // Retrieve the current shell for later use (if needed).
        let shell = match std::env::var(SHELL_VAR) {
            Ok(shell) => shell,
            Err(e) => {
                eprintln!(
                    "Couldn't retrieve environment variable {}: {}",
                    SHELL_VAR, e
                );
                std::process::exit(1);
            }
        };

        for (key, values) in grouped_collection.iter() {
            if options.output.only_group_names == Flag::Unset {
                print_group_header(key);
            }

            // Invoke a new shell and run it with the provided arguments.
            // Note that we actually explicitly invoke a shell because the shell is
            // responsible for parsing the command string, which might (very likely)
            // have pipes, etc. This also frees the user to use whatever shell they
            // might prefer and to use its features (at least in theory).
            let shell_args = ["-c", &cmd];
            let mut child = Command::new(&shell)
                .args(&shell_args)
                .stdin(Stdio::piped())
                .stdout(Stdio::inherit())
                .spawn()
                .expect("Shell command failed.");
            {
                let mut writer = BufWriter::new(child.stdin.as_mut().unwrap());
                if options.output.only_group_names == Flag::Set {
                    writer.write_all(key.as_bytes()).unwrap();
                    writer.write_all(line_separator.as_bytes()).unwrap();
                } else {
                    for line in values.iter() {
                        writer.write_all(line.as_bytes()).unwrap();
                        writer.write_all(line_separator.as_bytes()).unwrap();
                    }
                }
                writer.flush().unwrap();
            }
            child.wait().unwrap();
        }
    } else {
        // Default behavior: print to standard output.
        for (key, values) in grouped_collection.iter() {
            if options.output.only_group_names == Flag::Set {
                print!("{}{}", key, line_separator);
            } else {
                print_group_header(key);
                for line in values.iter() {
                    print!("{}{}", line, line_separator);
                }
            }
        }
    }
}

fn print_group_header(key: &str) {
    println!("\n{}:", key);
}
