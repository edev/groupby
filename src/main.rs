use std::process::{Command, Stdio};

fn main() {
    // For plumbing the basic shell I/O, we'll read all the arguments
    // and pass them along to the user's default shell as sh -c <args>
    const SHELL_VAR: &str = "SHELL";
    let args: Vec<String> = std::env::args().collect();
    let cmd = args[1..].join(" ");
    let shell_args = ["-c", &cmd];
    println!("Parsed command: {}", cmd);
    match std::env::var(SHELL_VAR) {
        Ok(shell) => {
            println!("Found shell: {}", shell);
            Command::new(shell)
                .args(&shell_args)
                .stdout(Stdio::inherit())
                .output()
                .expect("Shell command failed.");
            println!("Shell invocation complete.");
        }
        Err(e) => eprintln!(
            "Couldn't retrieve environment variable {}: {}",
            SHELL_VAR, e
        ),
    }
}
