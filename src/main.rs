use clap::{crate_authors, crate_version, App, Arg, ArgGroup};
use groupby::*;
use std::io;
use std::io::BufRead;
use std::process::{Command, Stdio};

// The environment variable that stores the name of the current shell.
const SHELL_VAR: &str = "SHELL";

fn main() {
    let mut grouped_collection = GroupedCollection::new();

    // Use clap to parse command-line arguments.
    let matches = App::new("Groupby")
        .author(crate_authors!())
        .version(crate_version!())
        .long_about(
            "Reads lines from standard input and groups them by common substrings. Prints the resulting groups to standard output.\n\
             \n\
             One and only one grouping option must be specified.",
        )
        // Grouping arguments.
        .arg(
            Arg::with_name("first_chars")
                .short("f")
                .value_name("n")
                .help("Group by equivalence on the first n characters.")
                .takes_value(true),
        )
        // Add grouping arguments to a Clap ArgGroup.
        .group(
            ArgGroup::with_name("grouping")
                .required(true)
                .args(&[
                    "first_chars",
                ]),
        )
        // Output arguments.
        .arg(
            Arg::with_name("run_command")
                .short("c")
                .value_name("cmd")
                .help("Execute command cmd for each group, passing the group via standard input, one match per line.")
                .takes_value(true),
        )
        .get_matches();

    // Extract the grouping function to use so that we only perform this logic once (rather than
    // for each line).
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
    } else {
        panic!("No grouping operation was specified, but Clap didn't catch it. Please report this error!");
    };

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        grouping_function(line.clone());
    }

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

        // For testing: simply run the command as shell -c <args>
        //
        // Invoke a new shell and run it with the provided arguments.
        // Note that we actually explicitly invoke a shell because the shell is
        // responsible for parsing the command string, which might (very likely)
        // have pipes, etc. This also frees the user to use whatever shell they
        // might prefer and to use its features (at least in theory).
        let shell_args = ["-c", &cmd];
        Command::new(shell)
            .args(&shell_args)
            .stdout(Stdio::inherit())
            .output()
            .expect("Shell command failed.");
    }
}
