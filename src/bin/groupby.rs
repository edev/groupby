use groupby::command_line;
use groupby::command_line::options::*;
use groupby::command_line::process_input::*;
use groupby::command_line::run_command::{self, *};
use groupby::grouped_collections::GroupedCollection;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::sync::Mutex;

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

fn output_results<'a, Map>(map: &'a Map, options: &GroupByOptions)
where
    Map: for<'s> GroupedCollection<'s, String, String, Vec<String>>,
    &'a Map: IntoParallelIterator<Item = (&'a String, &'a Vec<String>)>,
{
    // Determine what line separator the user wants.
    let line_separator = options.output.separator.sep();

    // Generate the required outputs.
    if let Some(cmd) = &options.output.run_command {
        // Retrieve the current shell for later use (if needed).
        let shell = std::env::var(SHELL_VAR).unwrap_or_else(|e| {
            eprintln!(
                "Couldn't retrieve environment variable {}: {}",
                SHELL_VAR, e
            );
            std::process::exit(1);
        });

        let results: Mutex<BTreeMap<&str, Vec<u8>>> = Mutex::new(BTreeMap::new());

        // Initialize the shell arguments required to run a command via the current shell.
        // TODO Add a command-line option to specify the exact shell invocation.
        let shell_args = ["-c", cmd];

        map.par_iter().for_each(|(key, values)| {
            // Spawn the new shell process.
            let mut handle = run_command::run(&shell, shell_args, line_separator);

            // Pass along the group's contents (or name, if output.only_group_names) via stdin.
            if options.output.only_group_names {
                handle.stdin.provide(key);
            } else {
                handle.stdin.provide_all(values.iter());
            }

            // Wait for the process to finish, then record its output so we can print it later.
            let output = handle.wait_with_output().unwrap();
            results.report(key, output.stdout);
        });

        // Print the outputs from each group. Note: as long as we use BTreeMap for both results and
        // and map, which we do, the results should be sorted stably by key.
        for (key, output) in results.lock().unwrap().iter() {
            print_group_header(key);
            io::stdout().write_all(output).unwrap();
            print!("{}", line_separator);
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
