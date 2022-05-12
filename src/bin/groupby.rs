use groupby::command_line::{self, options::*};
use groupby::grouped_collections::GroupedCollection;
use groupby::groupers::string::*;
use std::collections::BTreeMap;
use std::io;
use std::io::{BufRead, BufWriter, Write};
use std::process::{Command, Stdio};

// The environment variable that stores the name of the current shell.
const SHELL_VAR: &str = "SHELL";

fn main() {
    let options = command_line::parse(command_line::args::args());
    let mut map = BTreeMap::<String, Vec<String>>::new();
    let stdin = io::stdin();
    let locked_stdin = stdin.lock();
    process_input(locked_stdin, &mut map, &options);
    output_results(&map, &options);
}

fn process_input<I, Map>(input: I, map: &mut Map, options: &GroupByOptions)
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

fn output_results<Map>(map: &Map, options: &GroupByOptions)
where
    Map: for<'s> GroupedCollection<'s, String, String, Vec<String>>,
{
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

        for (key, values) in map.iter() {
            if !options.output.only_group_names {
                print_group_header(key);
            }

            // Invoke a new shell and run it with the provided arguments.
            // Note that we actually explicitly invoke a shell because the shell is
            // responsible for parsing the command string, which might (very likely)
            // have pipes, etc. This also frees the user to use whatever shell they
            // might prefer and to use its features (at least in theory).
            //
            // TODO Add a command-line option to specify the exact shell invocation.
            let shell_args = ["-c", cmd];
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
        for (key, values) in map.iter() {
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
