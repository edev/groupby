use groupby::command_line::*;
use groupby::*;
use std::io;
use std::io::{BufRead, BufWriter, Write};
use std::process::{Command, Stdio};

// The environment variable that stores the name of the current shell.
const SHELL_VAR: &str = "SHELL";

fn main() {
    let options = command_line::parse(command_line::args::args());
    let mut grouped_collection = GroupedCollection::<String, String>::new();
    process_input(&mut grouped_collection, &options);
    output_results(&grouped_collection, &options);
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
        GroupingSpecifier::FirstChars(n) => {
            Box::new(move |s| grouped_collection.group_by_first_chars(s, *n))
        }
        GroupingSpecifier::LastChars(n) => {
            Box::new(move |s| grouped_collection.group_by_last_chars(s, *n))
        }
        GroupingSpecifier::Regex(re) => Box::new(move |s| grouped_collection.group_by_regex(s, re)),
    };

    // Process each line of input.
    let stdin = io::stdin();
    match options.input.separator {
        Separator::Null => {
            // Split on null characters and process every resulting token.
            for result in stdin.lock().split(0) {
                let token = result.unwrap();
                let token = String::from_utf8(token).unwrap();
                grouping_function(token);
            }
        },
        Separator::Space => {
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
        },
        Separator::Line => {
            // Process each line as a single token.
            for line in stdin.lock().lines() {
                let line = line.unwrap();
                grouping_function(line.clone());
            }
        }
    }
}

fn output_results(
    grouped_collection: &GroupedCollection<String, String>,
    options: &GroupByOptions,
) {
    // Determine what line separator the user wants.
    let line_separator = match options.output.separator {
        Separator::Null => "\0",
        Separator::Space => " ",
        Separator::Line => "\n",
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
            if !options.output.only_group_names {
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
                if options.output.only_group_names {
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
            if options.output.only_group_names {
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
