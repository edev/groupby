//! Building blocks for a command-line interface using [clap](https://github.com/clap-rs/clap/).
//!
//! This module breaks down the command-line interface that the `groupby` binary uses. The
//! intention is to allow other applications that might want to use part or all of this interface
//! to borrow this clap code and receive updates as new features evolve.
//!
//! Each level of organization is presented as one or more functions or methods, so you can choose
//! what you want to borrow. For instance, you can integrate the entirety of the `groupby`
//! interface as a subcommand using the [command()] function.

use clap::{command, Arg, ArgGroup, Command};

type Cmd = Command<'static>;

/// Provides individual methods for adding parts of the `groupby` command-line interface.
///
/// If the methods here were bare functions, calling them would be painful and far from idiomatic.
/// Instead, you can create a CommandBuilder, chain methods on it just like the builder interface
/// of Clap, and then retrive the `command` field once you're done.
pub struct CommandBuilder {
    pub command: Cmd,
}

/// Creates the entire Clap::Command, fully populated and ready to match.
///
/// This is what the `groupby` binary calls.
pub fn args() -> Cmd {
    command(command!())
}

/// Takes a partially built Command and adds `groupby`'s arguments.
pub fn command(command: Cmd) -> Cmd {
    CommandBuilder::new(command)
        .about()
        .input_split_options()
        .grouping_options()
        .output_separator_options()
        .output_options()
        .command
}

// Handles the boilerplate so that CommandBuilder's methods can simply focus on writing clap
// builder logic. For instance, to add an argument, you might write:
//
// pub fn dance(self) -> Self {
//     build!(
//         self,
//         arg,
//         Arg::new("dance")
//             .short('d')
//             .help("Dance like nobody's watching")
//     )
// }
//
// Note: to the best of my knowledge, we can't move the function definition into the macro body and
// still have Cargo generate documentation for the function. Therefore, we'll keep that bit of
// boilerplate, unfortunately.
macro_rules! build {
    ( $self:ident, $method:ident, $($arg:expr),* ) => {
        CommandBuilder {
            command: $self.command.$method($($arg),*),
        }
    }
}

impl CommandBuilder {
    /// A convenience constructor. Feel free to construct the struct directly if you prefer.
    pub fn new(command: Cmd) -> Self {
        CommandBuilder { command }
    }

    /// Adds the overall about message.
    pub fn about(self) -> Self {
        build!(
            self,
            about,
            "\nReads lines from standard input and groups them by common substrings. By default, \
            prints the resulting groups to standard output."
        )
    }

    /// Adds a section for input options.
    pub fn input_split_options(self) -> Self {
        self.input_split_options_heading()
            .input_split_on_whitespace()
            .input_split_on_null()
            .group_input_split_options()
    }

    /// Adds the input options heading.
    pub fn input_split_options_heading(self) -> Self {
        build!(
            self,
            next_help_heading,
            "INPUT-SPLITTING OPTIONS (choose zero or one)"
        )
    }

    /// Adds an option to split input on whitespace.
    pub fn input_split_on_whitespace(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("input_split_on_whitespace")
                .short('w')
                .help("Group words instead of lines; that is, split input on whitespace.")
        )
    }

    /// Adds an option to split input on null characters.
    pub fn input_split_on_null(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("input_split_on_null")
                .short('0')
                .help("Split input by null characters rather than lines.")
        )
    }

    /// Adds the input-splitting options into a group: choose at most one.
    pub fn group_input_split_options(self) -> Self {
        build!(
            self,
            group,
            ArgGroup::new("input_split")
                .args(&["input_split_on_whitespace", "input_split_on_null"])
        )
    }

    /// Adds a section for choosing a grouper.
    pub fn grouping_options(self) -> Self {
        self.grouping_heading()
            .group_by_first_chars()
            .group_by_last_chars()
            .group_by_regex()
            .group_groupers()
    }

    /// Adds the grouper heading.
    pub fn grouping_heading(self) -> Self {
        build!(self, next_help_heading, "GROUPERS (choose exactly one)")
    }

    /// Adds an option to specify the [crate::grouped_collection::GroupedCollection::group_by_first_chars] grouper.
    pub fn group_by_first_chars(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("group_by_first_chars")
                .short('f')
                .value_name("n")
                .takes_value(true)
                .help("Group by equivalence on the first n characters.")
        )
    }

    /// Adds an option to specify the [crate::grouped_collection::GroupedCollection::group_by_last_chars] grouper.
    pub fn group_by_last_chars(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("group_by_last_chars")
                .short('l')
                .value_name("n")
                .takes_value(true)
                .help("Group by equivalence on the last n characters.")
        )
    }

    /// Adds an option to specify the [crate::grouped_collection::GroupedCollection::group_by_regex] grouper.
    pub fn group_by_regex(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("group_by_regex")
                .short('r')
                .long("regex")
                .value_name("pattern")
                .takes_value(true)
                .help("Group by equivalence on the first match against the specified pattern.")
                .long_help(
                    "Group by equivalence on the first match against the specified regex pattern. \
                    If capture groups are present, group by equivalence on the first capture \
                    group. If a line does not match, it is stored in the blank group, \"\"."
                )
        )
    }

    /// Adds the grouper choices into a group: choose exactly one.
    pub fn group_groupers(self) -> Self {
        build!(
            self,
            group,
            ArgGroup::new("groupers")
                .args(&[
                    "group_by_first_chars",
                    "group_by_last_chars",
                    "group_by_regex"
                ])
                .required(true)
        )
    }

    /// Adds a section for output options.
    pub fn output_separator_options(self) -> Self {
        self.output_separator_heading()
            .output_null_separators()
            .output_space_separators()
            .group_output_separator_options()
    }

    /// Adds the output separator heading.
    pub fn output_separator_heading(self) -> Self {
        build!(
            self,
            next_help_heading,
            "OUTPUT SEPARATOR OPTIONS (choose zero or one)"
        )
    }

    /// Adds an option to separate records by null characters on output.
    pub fn output_null_separators(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("output_null_separators")
                .long("print0")
                .help("When outputting lines, separate them with a null character, not a newline.")
                .long_help(
                    "When outputting lines, separate them with a null character rather than a \
                    newline. This option is meant for compatibility with xargs -0."
                )
        )
    }

    /// Adds an option to separate records by spaces on output.
    pub fn output_space_separators(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("output_space_separators")
                .long("printspace")
                .help("When outputting lines, separate them with a space rather than a newline.")
        )
    }

    /// Adds the output separator options into a group: choose zero or one.
    pub fn group_output_separator_options(self) -> Self {
        build!(
            self,
            group,
            ArgGroup::new("output_separators")
                .args(&["output_null_separators", "output_space_separators"])
        )
    }

    /// Adds a section for general output options.
    pub fn output_options(self) -> Self {
        self.output_options_header()
            .output_only_group_names()
            .output_run_command()
    }

    /// Adds the general output options header.
    pub fn output_options_header(self) -> Self {
        build!(self, next_help_heading, "GENERAL OUTPUT OPTIONS")
    }

    /// Adds an option to output only group names, motting group contents.
    pub fn output_only_group_names(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("output_only_group_names").long("matches").help(
                "Instead of outputting lines, output the matched text that forms each group."
            )
        )
    }

    /// Adds an option to run a command over each group.
    pub fn output_run_command(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("output_run_command")
                .short('c')
                .value_name("cmd")
                .takes_value(true)
                .help(
                    "Execute command cmd for each group, passing the group via standard input."
                )
                .long_help(
                    "Execute command cmd for each group, passing the group via standard input, one \
                    match per line."
                )
        )
    }
}
