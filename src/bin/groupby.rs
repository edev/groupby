use groupby::command_line;
use std::collections::BTreeMap;
use std::io;

fn main() {
    // Parse command-line arguments into GroupByOptions struct.
    let options = command_line::parse(command_line::args());

    // Choose which GroupedCollection implementation we're going to use.
    let mut map = BTreeMap::<String, Vec<String>>::new();

    // Process stdin, populating map.
    let stdin = io::stdin();
    command_line::build_groups(stdin.lock(), &mut map, &options);

    // Depending on options, either print map directly or run a specified command over each group.
    command_line::output_results(io::stdout(), &map, &options);
}
