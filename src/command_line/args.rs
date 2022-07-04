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
        .groupers()
        .grouper_options()
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

    /// Adds the overall about message, both short and long forms.
    pub fn about(self) -> Self {
        self.short_about().long_about()
    }

    /// Adds an overall about message for short help.
    pub fn short_about(self) -> Self {
        build!(
            self,
            about,
            "\nReads lines from standard input and groups them by common substrings. By default, \
            prints the resulting groups to standard output."
        )
    }

    /// Adds an overall about message for long help.
    pub fn long_about(self) -> Self {
        build!(
            self,
            long_about,
            "\nReads lines from standard input and groups them by common substrings. By default, \
            prints the resulting groups to standard output.\n\
            \n\
            For example, to group lines in a structured log file by the first 10 characters:

    groupby -f 10 < foo.log\n\
            \n\
            Much more complex use cases are also supported. For instance, to group files in \
            ~/Pictures by file extension (case-sensitive) and print how much disk space each type \
            of file is using:

    find ~/Pictures/ -not -type d -print0 \\
        | groupby -0 --extension --print0 -c \"xargs -0 du -chL | tail -n1\"\n\
            \n\
            Note: the lack of an option to group by the first or last n words is an intional \
            omission. There are many ways to define a word, and when grouping by words, the exact \
            definition matters. To match based on words, please use --regex and supply a \
            definition that works for your use case."
        )
    }

    /// Adds a section for input options.
    pub fn input_split_options(self) -> Self {
        self.input_split_options_heading()
            .input_split_on_whitespace()
            .input_split_on_null()
            .input_split_on_custom()
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

    /// Adds an option to split on a custom string.
    pub fn input_split_on_custom(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("input_split_on_custom")
                .long("split")
                .value_name("delim")
                .takes_value(true)
                .help("Split input on a custom delimiter of your choice.")
        )
    }

    /// Adds the input-splitting options into a group: choose at most one.
    pub fn group_input_split_options(self) -> Self {
        build!(
            self,
            group,
            ArgGroup::new("input_split").args(&[
                "input_split_on_whitespace",
                "input_split_on_null",
                "input_split_on_custom"
            ])
        )
    }

    /// Adds a section for choosing a grouper.
    pub fn groupers(self) -> Self {
        self.groupers_heading()
            .groupers_by_first_chars()
            .groupers_by_last_chars()
            .groupers_by_regex()
            .groupers_by_file_extension()
            .groupers_by_counter()
            .group_groupers()
    }

    /// Adds the grouper heading.
    pub fn groupers_heading(self) -> Self {
        build!(self, next_help_heading, "GROUPERS (choose exactly one)")
    }

    /// Adds an option to specify the [crate::groupers::string::Groupers::group_by_first_chars] grouper.
    pub fn groupers_by_first_chars(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("groupers_by_first_chars")
                .short('f')
                .value_name("n")
                .takes_value(true)
                .help("Group by equivalence on the first n characters.")
        )
    }

    /// Adds an option to specify the [crate::groupers::string::Groupers::group_by_last_chars] grouper.
    pub fn groupers_by_last_chars(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("groupers_by_last_chars")
                .short('l')
                .value_name("n")
                .takes_value(true)
                .help("Group by equivalence on the last n characters.")
        )
    }

    /// Adds an option to specify the [crate::groupers::string::Groupers::group_by_regex] grouper.
    pub fn groupers_by_regex(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("groupers_by_regex")
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

    /// Adds an option to specify the [crate::groupers::string::Groupers::group_by_file_extension]
    /// grouper.
    pub fn groupers_by_file_extension(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("groupers_by_file_extension")
                .long("extension")
                .help("Group by file extension (excluding the leading period).")
                .long_help(
                    "Group by file extension (excluding the leading period). Files with multiple \
                    extensions will match the last extension, e.g. foo.tar.gz will match \"gz\". \
                    Files with only a leading period are considered not to have an extension. So \
                    are files ending in a period. If you need a different definition of a file \
                    extension, please consider using --regex."
                )
        )
    }

    /// Adds an option to specify the [crate::groupers::string::Groupers::group_by_counter] grouper.
    pub fn groupers_by_counter(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("groupers_by_counter")
                .long("counter")
                .help("Place each token in its own, numbered group, starting from 0.")
                .long_help(
                    "Place each token in its own, numbered group, starting from 0. This is useful \
                    for running a command over every token of input, i.e. acting as a splitter \
                    filter."
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
                    "groupers_by_first_chars",
                    "groupers_by_last_chars",
                    "groupers_by_regex",
                    "groupers_by_file_extension",
                    "groupers_by_counter",
                ])
                .required(true)
        )
    }

    /// Adds a section for customizing the behavior of groupers.
    pub fn grouper_options(self) -> Self {
        self.grouper_options_heading()
            .grouper_options_capture_group()
    }

    /// Adds the grouper options heading.
    pub fn grouper_options_heading(self) -> Self {
        build!(self, next_help_heading, "GROUPER OPTIONS")
    }

    /// Adds an option to specify a capture group number or name when using a regex grouper.
    pub fn grouper_options_capture_group(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("grouper_options_capture_group")
                .long("capture-group")
                .help("When used with -r, match a specific capture group by number or name.")
                .long_help(
                    "When used with -r, match a specific capture group by number or name. Group \
                    number 0 matches the entire pattern."
                )
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
            .output_no_headers()
            .output_only_group_names()
            .output_run_command()
            .output_sequential()
            .output_stats()
    }

    /// Adds the general output options header.
    pub fn output_options_header(self) -> Self {
        build!(self, next_help_heading, "GENERAL OUTPUT OPTIONS")
    }

    /// Adds an option to skip outputting group names at final output. No interaction with -c.
    pub fn output_no_headers(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("output_no_headers")
                .long("no-headers")
                .help("At final output, do not print group headers. Does not affect -c.")
                .long_help(
                    "When printing final output, do not print a header before each group. Only print the final output for each group, back-to-back. Groups are still sorted by group name.\n\
                    \n\
                    When used with -c, commands are not affected in any way. The only difference is that final results will be printed back-to-back, with no delimiter between them. This may be useful for chaining terminal filters on this program's stdout."
                )
        )
    }

    /// Adds an option to output only group names, omitting group contents.
    pub fn output_only_group_names(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("output_only_group_names")
                .long("only-group-names")
                .help("Output only group names, omitting group contents.")
                .long_help(
                    "Output only group names, omitting group contents.\n\
                    \n\
                    When used with -c, passes the name of each group to its command instead of \
                    passing the group's contents."
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
                .long("run-command")
                .value_name("cmd")
                .takes_value(true)
                .help(
                    "Execute command cmd for each group, passing the group via stdin."
                )
                .long_help(
                    "Execute cmd as a shell command for each group, passing the group via standard \
                    input, one match per line. Each command runs as a command in the shell \
                    specified by the SHELL variable, just as if you had written $SHELL -c \
                    \"cmd\". After all commands are run, the output for each group's command will \
                    be printed instead of the group's contents.\n\
                    \n\
                    When you use this option, most other output options affect the way each group \
                    is passed to a command's standard input. When printing the outputs of the \
                    commands, groupby resets most output-formatting options to their defaults.\n\
                    \n\
                    The commands are run in parallel and may run in arbitrary order. The commands' \
                    outputs are printed in order by group name."
                )
        )
    }

    /// Adds an option to run commands sequentially rather than in parallel.
    pub fn output_sequential(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("output_sequential")
                .long("sequential")
                .help("When used with -c, run commands in sequence, ordered by group name.")
                .long_help(
                    "When used with -c, run commands in sequence, ordered by group name, using a \
                    single thread. This may be much slower. This option has no effect if used \
                    without -c."
                )
        )
    }

    /// Adds an option to display statistics for each group and for the collection as a whole.
    pub fn output_stats(self) -> Self {
        build!(
            self,
            arg,
            Arg::new("output_stats")
                .long("stats")
                .help("Print statistics about groups alongside normal output.")
                .long_help(
                    "Print an item count for each group, plus statistics about the overall \
                    collection, in addition to any other output (as specified by other options).\n\
                    \n\
                    This option is not affected by -c. When used with -c, the text sent to each \
                    command does not change. The final output is augmented with statistics about \
                    the groups and their contents (not about the commands or their outputs)."
                )
        )
    }
}

/// To hopefully balance simplicity with correctness, since this is heavily hand-crafted by design,
/// we'll test this whole module by simply comparing the output of [args()] to the expected output.
/// If clap proves unstable across versions, then we can use a more advanced approach (such as
/// being more flexible about whitespace). For now, this seems to strike a good balance between
/// simplicity and thorough checks. There's really no use spending the time writing unit tests for
/// every single method in this module, and doing so would be onerous.
#[cfg(test)]
mod args_tests {
    use super::*;

    #[test]
    fn short_help_works() {
        let mut command = args();
        let mut buffer: Vec<u8> = vec![];
        command.write_help(&mut buffer).unwrap();
        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            format!(
                "\
groupby {}
Dylan Laufenberg <dylan.laufenberg@gmail.com>

Reads lines from standard input and groups them by common substrings. By default, prints the
resulting groups to standard output.

USAGE:
    groupby [OPTIONS] <-f <n>|-l <n>|--regex <pattern>|--extension|--counter>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

INPUT-SPLITTING OPTIONS (choose zero or one):
    -0                     Split input by null characters rather than lines.
        --split <delim>    Split input on a custom delimiter of your choice.
    -w                     Group words instead of lines; that is, split input on whitespace.

GROUPERS (choose exactly one):
        --counter            Place each token in its own, numbered group, starting from 0.
        --extension          Group by file extension (excluding the leading period).
    -f <n>                   Group by equivalence on the first n characters.
    -l <n>                   Group by equivalence on the last n characters.
    -r, --regex <pattern>    Group by equivalence on the first match against the specified pattern.

GROUPER OPTIONS:
        --capture-group    When used with -r, match a specific capture group by number or name.

OUTPUT SEPARATOR OPTIONS (choose zero or one):
        --print0        When outputting lines, separate them with a null character, not a newline.
        --printspace    When outputting lines, separate them with a space rather than a newline.

GENERAL OUTPUT OPTIONS:
    -c, --run-command <cmd>    Execute command cmd for each group, passing the group via stdin.
        --no-headers           At final output, do not print group headers. Does not affect -c.
        --only-group-names     Output only group names, omitting group contents.
        --sequential           When used with -c, run commands in sequence, ordered by group name.
        --stats                Print statistics about groups alongside normal output.\n",
                env!("CARGO_PKG_VERSION")
            )
        );
    }

    #[test]
    fn long_help_works() {
        let mut command = args();
        let mut buffer: Vec<u8> = vec![];
        command.write_long_help(&mut buffer).unwrap();
        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            format!(
                "\
groupby {}
Dylan Laufenberg <dylan.laufenberg@gmail.com>

Reads lines from standard input and groups them by common substrings. By default, prints the
resulting groups to standard output.

For example, to group lines in a structured log file by the first 10 characters:

    groupby -f 10 < foo.log

Much more complex use cases are also supported. For instance, to group files in ~/Pictures by file
extension (case-sensitive) and print how much disk space each type of file is using:

    find ~/Pictures/ -not -type d -print0 \\
        | groupby -0 --extension --print0 -c \"xargs -0 du -chL | tail -n1\"

Note: the lack of an option to group by the first or last n words is an intional omission. There are
many ways to define a word, and when grouping by words, the exact definition matters. To match based
on words, please use --regex and supply a definition that works for your use case.

USAGE:
    groupby [OPTIONS] <-f <n>|-l <n>|--regex <pattern>|--extension|--counter>

OPTIONS:
    -h, --help
            Print help information

    -V, --version
            Print version information

INPUT-SPLITTING OPTIONS (choose zero or one):
    -0
            Split input by null characters rather than lines.

        --split <delim>
            Split input on a custom delimiter of your choice.

    -w
            Group words instead of lines; that is, split input on whitespace.

GROUPERS (choose exactly one):
        --counter
            Place each token in its own, numbered group, starting from 0. This is useful for running
            a command over every token of input, i.e. acting as a splitter filter.

        --extension
            Group by file extension (excluding the leading period). Files with multiple extensions
            will match the last extension, e.g. foo.tar.gz will match \"gz\". Files with only a
            leading period are considered not to have an extension. So are files ending in a period.
            If you need a different definition of a file extension, please consider using --regex.

    -f <n>
            Group by equivalence on the first n characters.

    -l <n>
            Group by equivalence on the last n characters.

    -r, --regex <pattern>
            Group by equivalence on the first match against the specified regex pattern. If capture
            groups are present, group by equivalence on the first capture group. If a line does not
            match, it is stored in the blank group, \"\".

GROUPER OPTIONS:
        --capture-group
            When used with -r, match a specific capture group by number or name. Group number 0
            matches the entire pattern.

OUTPUT SEPARATOR OPTIONS (choose zero or one):
        --print0
            When outputting lines, separate them with a null character rather than a newline. This
            option is meant for compatibility with xargs -0.

        --printspace
            When outputting lines, separate them with a space rather than a newline.

GENERAL OUTPUT OPTIONS:
    -c, --run-command <cmd>
            Execute cmd as a shell command for each group, passing the group via standard input, one
            match per line. Each command runs as a command in the shell specified by the SHELL
            variable, just as if you had written $SHELL -c \"cmd\". After all commands are run, the
            output for each group's command will be printed instead of the group's contents.
            
            When you use this option, most other output options affect the way each group is passed
            to a command's standard input. When printing the outputs of the commands, groupby resets
            most output-formatting options to their defaults.
            
            The commands are run in parallel and may run in arbitrary order. The commands' outputs
            are printed in order by group name.

        --no-headers
            When printing final output, do not print a header before each group. Only print the
            final output for each group, back-to-back. Groups are still sorted by group name.
            
            When used with -c, commands are not affected in any way. The only difference is that
            final results will be printed back-to-back, with no delimiter between them. This may be
            useful for chaining terminal filters on this program's stdout.

        --only-group-names
            Output only group names, omitting group contents.
            
            When used with -c, passes the name of each group to its command instead of passing the
            group's contents.

        --sequential
            When used with -c, run commands in sequence, ordered by group name, using a single
            thread. This may be much slower. This option has no effect if used without -c.

        --stats
            Print an item count for each group, plus statistics about the overall collection, in
            addition to any other output (as specified by other options).
            
            This option is not affected by -c. When used with -c, the text sent to each command does
            not change. The final output is augmented with statistics about the groups and their
            contents (not about the commands or their outputs).\n",
                env!("CARGO_PKG_VERSION")
            )
        );
    }
}
