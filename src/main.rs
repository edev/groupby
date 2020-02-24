use clap::{App, Arg};
use std::process::{Command, Stdio};

// The environment variable that stores the name of the current shell.
const SHELL_VAR: &str = "SHELL";

// For plumbing the basic shell I/O, we'll read all the arguments
// and pass them along to the user's default shell as sh -c <args>
fn main() {
    // Use clap to parse command-line arguments.
    let matches = App::new("Groupby")
        .author("Dylan Laufenberg <dylan.laufenberg@gmail.com>")
        .about(
            "Reads lines from standard input and groups them by common \
             substrings. Prints the resulting groups to standard output.",
        )
        .arg(
            Arg::with_name("first_chars")
                .short("f")
                .value_name("n")
                .help("Group by equivalence on the first n characters.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("run_command")
                .short("c")
                .value_name("cmd")
                .help(
                    "Execute command cmd for each group, passing the group via \
                  standard input, one match per line.",
                )
                .takes_value(true),
        )
        .get_matches();

    if let Some(n) = matches.value_of("first_chars") {
        println!("NYI: -f {}", n);
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
