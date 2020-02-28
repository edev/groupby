use clap::{crate_authors, crate_version, App, Arg, ArgGroup, ArgMatches};
use groupby::*;
use regex::Regex;
use std::io;
use std::io::{BufRead, BufWriter, Write};
use std::process::{Command, Stdio};

// The environment variable that stores the name of the current shell.
const SHELL_VAR: &str = "SHELL";

fn main() {
    let mut grouped_collection = GroupedCollection::<String, String>::new();
    let matches = args();

    process_input(&mut grouped_collection, &matches);
    output_results(&grouped_collection, &matches);
}

// Use clap to parse command-line arguments.
fn args<'a>() -> ArgMatches<'a> {
    App::new("Groupby")
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
            Arg::with_name("split_on_whitespace")
                .short("w")
                .help("Group words instead of lines; that is, split input on whitespace.")
        )

        // Grouping arguments.
        .arg(
            Arg::with_name("first_chars")
                .short("f")
                .value_name("n")
                .help("Group by equivalence on the first n characters.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("last_chars")
                .short("l")
                .value_name("n")
                .help("Group by equivalence on the last n characters.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("regex")
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
                    "first_chars",
                    "last_chars",
                    "regex"
                ]),
        )

        // Output arguments.
        .arg(
            Arg::with_name("print0")
                .long("print0")
                .help("When outputting lines, separate them with a null character rather than a newline. This option is meant for compatibility with xargs -0.")
        )
        .arg(
            Arg::with_name("printspace")
                .long("printspace")
                .help("When outputting lines, separate them with a space rather than a newline.")
        )
        .arg(
            Arg::with_name("only_output_group_names")
                .long("matches")
                .help("Instead of outputting lines, output the matched text that forms each group.")
        )
        .arg(
            Arg::with_name("run_command")
                .short("c")
                .value_name("cmd")
                .help("Execute command cmd for each group, passing the group via standard input, one match per line.")
                .takes_value(true),
        )

        // Retrieve and return the actual arguments provided by the user.
        .get_matches()
}

fn process_input(grouped_collection: &mut GroupedCollection<String, String>, matches: &ArgMatches) {
    // Process input.

    // Extract the grouping function to use so that we only perform this logic once
    // rather than for each line.
    let mut grouping_function: Box<dyn FnMut(String)> = if let Some(n) =
        matches.value_of("first_chars")
    {
        match n.parse::<usize>() {
            Ok(n) => Box::new(move |s| grouped_collection.group_by_first_chars(s, n)),
            Err(_) => {
                eprintln!("Error: {} is not a whole number.", n);
                std::process::exit(1);
            }
        }
    } else if let Some(n) = matches.value_of("last_chars") {
        match n.parse::<usize>() {
            Ok(n) => Box::new(move |s| grouped_collection.group_by_last_chars(s, n)),
            Err(_) => {
                eprintln!("Error: {} is not a whole number.", n);
                std::process::exit(1);
            }
        }
    } else if let Some(pattern) = matches.value_of("regex") {
        // Compile the regex just once, then move it into the closure, where it will be reused
        // to check every line of input.
        match Regex::new(pattern) {
            Ok(re) => Box::new(move |s| grouped_collection.group_by_regex(s, &re)),
            Err(e) => {
                eprintln!("{}", e); // The provided messages are actually really good.
                std::process::exit(1);
            }
        }
    } else {
        panic!("No grouping operation was specified, but Clap didn't catch it. Please report this error!");
    };

    // Process each line of input.
    let stdin = io::stdin();
    if matches.is_present("split_on_whitespace") {
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
        // PRocess each line as a single token.
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            grouping_function(line.clone());
        }
    }
}

fn output_results(grouped_collection: &GroupedCollection<String, String>, matches: &ArgMatches) {
    // Determine what line separator the user wants.
    let line_separator: &[u8] = if matches.is_present("print0") {
        b"\0"
    } else if matches.is_present("printspace") {
        b" "
    } else {
        b"\n"
    };

    // Determine what to print for output.
    let only_output_group_names: bool = matches.is_present("only_output_group_names");

    // Generate the required outputs.
    if let Some(cmd) = matches.value_of("run_command") {
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
            if !only_output_group_names {
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
                if only_output_group_names {
                    writer.write_all(key.as_bytes()).unwrap();
                    writer.write_all(line_separator).unwrap();
                } else {
                    for line in values.iter() {
                        writer.write_all(line.as_bytes()).unwrap();
                        writer.write_all(line_separator).unwrap();
                    }
                }
                writer.flush().unwrap();
            }
            child.wait().unwrap();
        }
    } else {
        // Default behavior: print to standard output.
        for (key, values) in grouped_collection.iter() {
            if only_output_group_names {
                println!("{}", key);
            } else {
                print_group_header(key);
                for line in values.iter() {
                    println!("{}", line);
                }
            }
        }
    }
}

fn print_group_header(key: &str) {
    println!("\n{}:", key);
}
