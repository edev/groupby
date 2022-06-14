#![allow(dead_code)]

use crate::command_line::run_command::*;
use std::collections::BTreeMap;

// Returns a ShellCommandOptions for use in run* tests.
pub fn options<'a>(only_group_names: bool) -> ShellCommandOptions<'a> {
    ShellCommandOptions {
        shell: current_shell(),
        shell_args: shell_args("cat"),
        line_separator: "   ".to_string(),
        only_group_names,
    }
}

pub fn map() -> BTreeMap<String, Vec<String>> {
    let mut map = BTreeMap::new();
    map.insert(
        "Dogs".to_string(),
        vec!["Lassy".to_string(), "Buddy".to_string()],
    );
    map.insert(
        "Cats".to_string(),
        vec!["Meowser".to_string(), "Mittens".to_string()],
    );
    map
}

pub fn results<'a>() -> BTreeMap<&'a String, Vec<u8>> {
    BTreeMap::new()
}

pub fn expected_results<'a>(
    map: &'a BTreeMap<String, Vec<String>>,
    separator: &str,
    only_group_names: bool,
) -> BTreeMap<&'a String, Vec<u8>> {
    let mut expected = BTreeMap::new();
    for (key, vector) in map.iter() {
        if only_group_names {
            // Group name plus separator.
            let value = key.to_owned() + separator;

            // Convert to Vec<u8> and insert.
            let value = value.as_bytes().to_vec();
            expected.insert(key, value);
        } else {
            expected.insert(
                key,
                vector
                    .iter()
                    .map(|s| s.to_owned() + separator)
                    .collect::<String>()
                    .as_bytes()
                    .to_vec(),
            );
        }
    }
    expected
}
