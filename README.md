# GroupBy

Group lines of input according to their contents.

## Description

GroupBy is a Linux terminal filter for grouping lines of input according to their contents. You can specify a portion of a line to examine, and `groupby` will group together any lines of input that have that portion of the line in common.

## Compiling

GroupBy is written in the Rust programming language and packaged with Cargo. You will need a recent version of Rust that supports the 2018 edition of the language. One easy way to obtain Rust on your Linux system is to follow the directions on [rustup.rs](rustup.rs).

To compile GroupBy, run:

```
cargo build --release
```

This will compile `groupby` into `target/release`. From there, you can copy it wherever you like, e.g. to `~/bin`. The examples below assume that `groupby` is installed somewhere in your `PATH`. (Because of the immature state of this project, no install scripts are provided.)

If you prefer to run it in-place, you can replace `groupby` with `cargo run --release --` in any example below, as long as you are inside GroupBy's directory structure.

## Getting help

To see the currently supported options, run `groupby -h` or `groupby --help`.

## Example: group by prefix

Suppose we have a text file with some course descriptions from [The University of California at Davis](https://www.ucdavis.edu/):

```
$ cat example_input.txt
DES 001—Introduction to Design (4)
DES 014—Design Drawing (4)
DES 015—Form & Color (4)
CRD 001—The Community (4)
CRD 002—Ethnicity & American Communities (4)
CRD 020—Food Systems (4)
ECS 012—Introduction to Media Computation (4)
ECS 015—Introduction to Computers (4)
ECS 020—Discrete Mathematics For Computer Science (4)
```

By providing this file as input to `groupby`, we can group the lines according to their 3-character prefixes:

```
$ groupby -f 3 < example_input.txt

CRD:
CRD 001—The Community (4)
CRD 002—Ethnicity & American Communities (4)
CRD 020—Food Systems (4)

DES:
DES 001—Introduction to Design (4)
DES 014—Design Drawing (4)
DES 015—Form & Color (4)

ECS:
ECS 012—Introduction to Media Computation (4)
ECS 015—Introduction to Computers (4)
ECS 020—Discrete Mathematics For Computer Science (4)
```

## Example: group by regular expression

It's also possible to group by regular expression (regex) matches. (For advanced notes on how this works, see `groupby --help`.) For instance, suppose we have a project directory with lots of different types of files, including some files with notes coded by date:

```
$ ls -1
icon.png
main.rs
...
'Notes 2019-09-01.txt'
'Notes 2019-11-14.txt'
...
'Notes 2020-02-21.txt'
'Notes 2020-02-22.txt'
'Notes 2020-02-23.txt'
...
```

We can use `find` to select just the notes files and group them by year:

```
$ find . -iname 'notes*.txt' | groupby -r '\d{4}'

2019:
./Notes 2019-09-01.txt
./Notes 2019-11-14.txt
...

2020:
./Notes 2020-02-21.txt
./Notes 2020-02-22.txt
./Notes 2020-02-23.txt
...
```

## Example: passing groups to a filter chain

GroupBy's real power is its ability to pass each group to a command sequence. For instance, we can extend the grouping from the previous example to count the lines, words, and characters in each group:

```
$ find . -iname 'notes*.txt' | groupby -r '\d{4}' --print0 -c "xargs -0 wc | tail -n 1"

2019:
  2458 6474 32604 total

2020:
  4735 9719 53609 total
```

Let's discuss what's happening in detail:

* `find . -iname 'notes*.txt'` prints out a list of paths to note files in the current directory and all subdirectories, one file per line.

* `groupby -r '\d{4}'` matches the 4-digit date in each file path and groups paths according to their 4-digit dates. This places the 2019 paths in one group and the 2020 paths in another group.

* `-c "xargs ..."` invokes the quoted command sequence *using the current user's current shell* once for each group. At each invocation, it writes the contents of one group to the command sequence's standard input. So in this example, `xargs` is called twice.

* `xargs` reads from standard input, building up arguments to pass to a further command. It passes those arguments to `wc | tail -n 1`.

* Since `xargs` delimits arguments in standard input by whitespace (e.g. space characters, newlines, etc.), it won't correctly parse our note file paths, which contain spaces. As a workaround, it provides the `-0` flag to delimit arguments by null characters instead of whitespace. The `find` command has a `-print0` option to delimit its output entries by null characters instead of newlines; `groupy` mimics this behavior with its `--print0` option for the same reason.

* `wc` counts the lines, words, and characters, then passes that information on to the `tail` filter, which (when used with the `-n 1` option) outputs only the summary line from `wc`.

## License

This project is released under the "MIT license". Please see the file [LICENSE](https://github.com/edev/groupby/blob/master/LICENSE) in this distribution for license terms.
