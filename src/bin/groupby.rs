use groupby::command_line;
use std::collections::BTreeMap;
use std::io;

fn main() {
    // Parse command-line arguments into GroupByOptions struct.
    let options = command_line::parse(command_line::args());

    // Choose which GroupedCollection implementation we're going to use.
    let mut map = BTreeMap::<String, Vec<String>>::new();

    // Process stdin, building a GroupedCollection.
    let stdin = io::stdin();
    command_line::build_groups(stdin.lock(), &mut map, &options);

    // If requested, run commands over the GroupedCollection and return a map of the commands'
    // captured standard outputs.
    let command_results = command_line::run_command(&map, &options.output);

    // Write the final results, per the user's options, to standard output.
    command_line::write_results(io::stdout(), &map, &command_results, &options.output);
}
