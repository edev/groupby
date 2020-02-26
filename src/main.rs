use clap::{crate_authors, crate_version, App, Arg, ArgGroup};
use groupby::*;
use std::io;
use std::io::{BufRead, BufWriter, Write};
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
            "Reads lines from standard input and groups them by common substrings. Prints the resulting groups to standard output unless -c is used.\n\
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
        .arg(
            Arg::with_name("last_chars")
                .short("l")
                .value_name("n")
                .help("Group by equivalence on the last n characters.")
                .takes_value(true),
        )
        // Add grouping arguments to a Clap ArgGroup.
        .group(
            ArgGroup::with_name("grouping")
                .required(true)
                .args(&[
                    "first_chars",
                    "last_chars"
                ]),
        )
        // Output arguments.
        .arg(
            Arg::with_name("print0")
                .long("print0")
                .help("When outputting lines, separate them with a null character rather than a newline. This option is meant for compatibility with xargs -0.")
        )
        .arg(
            Arg::with_name("run_command")
                .short("c")
                .value_name("cmd")
                .help("Execute command cmd for each group, passing the group via standard input, one match per line.")
                .takes_value(true),
        )
        .get_matches();

    {
        // Extract the grouping function to use so that we only perform this logic once (rather than
        // for each line).
        let mut grouping_function: Box<dyn FnMut(String)> = if let Some(n) =
            matches.value_of("first_chars")
        {
            match n.parse::<usize>() {
                Ok(n) => {
                    let coll = &mut grouped_collection;
                    Box::new(move |s| coll.group_by_first_chars(s, n))
                }
                Err(_) => {
                    eprintln!("Error: {} is not a whole number.", n);
                    std::process::exit(1);
                }
            }
        } else if let Some(n) = matches.value_of("last_chars") {
            match n.parse::<usize>() {
                Ok(n) => {
                    let coll = &mut grouped_collection;
                    Box::new(move |s| coll.group_by_last_chars(s, n))
                }
                Err(_) => {
                    eprintln!("Error: {} is not a whole number.", n);
                    std::process::exit(1);
                }
            }
        } else {
            panic!("No grouping operation was specified, but Clap didn't catch it. Please report this error!");
        };

        // Process each line of input.
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            grouping_function(line.clone());
        }
    }

    // Determine what line separator the user wants.
    let line_separator: &[u8] = if matches.is_present("print0") {
        b"\0"
    } else {
        b"\n"
    };

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
            print_group_header(key);

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
                for line in values.iter() {
                    writer.write(line.as_bytes()).unwrap();
                    writer.write(line_separator).unwrap();
                }
                writer.flush().unwrap();
            }
            child.wait().unwrap();
        }
    } else {
        // Default behavior: print to standard output.
        for (key, values) in grouped_collection.iter() {
            print_group_header(key);
            for line in values.iter() {
                println!("{}", line);
            }
        }
    }
}

fn print_group_header(key: &str) {
    println!("<<< Group: {} >>>", key);
}
