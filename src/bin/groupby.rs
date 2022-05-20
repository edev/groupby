use groupby::command_line;
use groupby::command_line::output_results::output_results;
use groupby::command_line::process_input::process_input;
use std::collections::BTreeMap;
use std::io;

fn main() {
    // Parse command-line arguments into GroupByOptions struct.
    let options = command_line::parse(command_line::args::args());

    // Choose which GroupedCollection implementation we're going to use.
    let mut map = BTreeMap::<String, Vec<String>>::new();

    // Process stdin, populating map.
    let stdin = io::stdin();
    process_input(stdin.lock(), &mut map, &options);

    // Depending on options, either print map directly or run a specified command over each group.
    output_results(io::stdout(), &map, &options);
}
