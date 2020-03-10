# GroupBy

*Team members: Dylan Laufenberg ([lauf@pdx.edu](mailto:lauf@pdx.edu))*

This project consists of an application and a supporting library.

The GroupBy library provides support for grouping items of arbitrary types by common keys.

The GroupBy application is a terminal filter for POSIX systems that groups lines of text according to their contents. You can specify a portion of a line to examine, and `groupby` will group together any lines of input that have that portion of the line in common.

## Compiling & running

GroupBy is written in the Rust programming language and packaged with Cargo. You will need a recent version of Rust that supports the 2018 edition of the language. One easy way to obtain Rust on your Linux system is to follow the directions on [rustup.rs](rustup.rs).

To compile GroupBy, run:

```
cargo build --release
```

This will compile `groupby` into `target/release`. From there, you can copy it wherever you like, e.g. to `~/bin`. The examples below assume that `groupby` is installed somewhere in your `PATH`.

If you prefer to run it in-place, you can replace `groupby` with `cargo run --release --` in any example below, as long as you are inside GroupBy's directory structure.

## Getting help

To see the currently supported options, run `groupby -h` or `groupby --help`, like so:

```
$ groupby --help
Groupby 0.1.0
Dylan Laufenberg <dylan.laufenberg@gmail.com>
Reads lines from standard input and groups them by common substrings. Prints the resulting groups to standard output
unless -c is used.

One and only one grouping option must be specified.

USAGE:
    groupby [FLAGS] [OPTIONS] <-f <n>|-l <n>|--regex <pattern>>

FLAGS:
    -w                  
            Group words instead of lines; that is, split input on whitespace.

        --print0        
            When outputting lines, separate them with a null character rather than a newline. This option is meant for
            compatibility with xargs -0.
        --matches       
            Instead of outputting lines, output the matched text that forms each group.

        --printspace    
            When outputting lines, separate them with a space rather than a newline.

    -h, --help          
            Prints help information

    -V, --version       
            Prints version information


OPTIONS:
    -f <n>                   
            Group by equivalence on the first n characters.

    -l <n>                   
            Group by equivalence on the last n characters.

    -r, --regex <pattern>    
            Group by equivalence on the first match against the specified regex pattern. If capture groups are present,
            group by equivalence on the first capture group. If a line does not match, it is stored in the blank group,
            "".
    -c <cmd>                 
            Execute command cmd for each group, passing the group via standard input, one match per line.
```

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

We can use `find` to select just the note files and group them by year:

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

## How GroupBy works

The core of the library portion of GroupBy is the `GroupedCollection` struct. At its heart, `GroupedCollection` is a `HashMap` that associates arbitrary keys with vectors of arbitrary values. This struct has a simple API:

* A constructor, `new`, builds an empty `GroupedCollection`.

* `add` adds a new value to the collection. If the collection already knows of the specified key, the value is added to the existing vector. Otherwise, a new key-value pair is stored.

* `get` retrieves a vector of values associated with a key (if any).

* `iter` returns an iterator that yields each key-vector pair in turn.

In order to process a collection, one calls `new`, repeatedly adds to the collection by calling `add`, and then uses `iter` to iterate over rows (e.g. in a `for` loop). This iteration will likely consist of iterating over each vector's elements in turn.

There are currently no methods to remove values from the collection; this would be a useful addition in future work.

### Matchers

A small collection of `String`-specific matchers extracts specified portions of a string. As of this writing, there are convenience matchers for the first `n` characters of a string, the last `n` characters of a string, and a regular expression over a string. For regular expression matches, if any capture groups are present, the first capture group is returned; otherwise, the full match is returned.

### Groupers

Each matcher corresponds to a grouper method that adds a `String` to the `GroupedCollection`, using the matched text as the `String`'s key. Each of these grouper methods, in turn, corresponds to a command-line option in the `groupby` application.

### Reference implementation

The application portion of Groupby follows a simple flow: read command-line arguments, process input, then output results.

It uses Clap to read command-line arguments. Clap appears to lack the ability to designate custom headers and groups in the help text that it generates, which is unsatisfactory for this application. It appears to arbitrarily group options with no arguments as "flags" and options with arguments as "options", offering no other organization options.

In order to (a) leverage Rust's type system to promote correct use of the command-line options within the application's source, (b) facilitate future expansion by organizing the options, (c) abstract away the details of Clap's implementation, and (d) and ease the future transition to a different system, I designed a `GroupByOptions` struct that maps all available command-line options to Rust types.

The input processing function is relatively straightforward. First, it matches the provided `GroupingOptions` enum (contained within the `GroupByOptions` struct) and builds a closure that, when called, correctly invokes the appropriate grouper. Then, it groups either lines or words, depending on the user's command-line arguments.

Finally, the function to output results either prints the requested output (which can be varied via a number of command-line arguments) or invokes a command sequence once for each group, writing the group's text to the command sequence's starting standard input.

### Testing

The library is tested via doctests. The application, due to its I/O-heavy nature, is carefully hand-checked and manually tested.

All tests pass. Neither the compiler nor Clippy generates any warnings.

## Reflections

Generally, everything went swimmingly and works well. I feel good about writing this particular application in Rust; to me, it feels like a good fit. I particularly appreciate that I won't need to manage an interpreter installation or install libraries to use GroupBy on any of my systems.

Clap has great documentation. I especially appreciated the respectful comparisons to other systems. Unfortunately, I was disappointed to find that this seemingly heavyweight tool lacked the depth of options that I needed. I'm disappointed with the current state of GroupBy's help output, and I intend to search for an alternative to Clap in the future. I was not able to match my initial prototype's help output with Clap.

The ultimate triumph for me, personally, is that I finally have a way to automatically group text files by the years embedded in their filenames and count the lines, words, and characters in each group. I have an actively growing collection of about 2.5 million words across a thousand or more files, and I have wanted for years to build an application to serve this exact purpose. I'm also honestly thrilled to have built a Linux filter that cooperates and fits in well with `find`, `wc`, `xargs`, and so on! I feel the strangest sense of accomplishment as a Linux power user.

My biggest lesson learned was actually domain-specific: I learned that almost everything I could reasonably want to have as a command-line option for grouping text could be expressed as a regular expression. I would like to add a dozen or more matchers and groupers for common groupings, but in the most technical sense, they won't add any new features to GroupBy. In fact, in this sense, two of my three matchers are redundant, though I maintain it's a lot more convenient and accessible to write `-f 3` than `^.{3}` to match the first three characters of a line.

I also learned more about Unicode and its interaction with regular expressions. I have never before given thought to supporting arbitrary languages in my regular expressions, so this project has opened my eyes.

## Related work

To the best of my knowledge, there are no similar projects publicly available in the POSIX ecosystem. The `find` utility is invaluable for searching directory paths but cannot group its results. The `xargs` utility blurs the definition of a terminal filter in similar ways to GroupBy. `sort`, `uniq`, and `awk` can perform some similar functions, but none of the above are able to divide lines of input into groups and invoke a command sequence once for each group, passing that group's full text as standard input. To the best of my knowledge, prior to the development of GroupBy,there was no way to perform this task automatically without writing code.

## License

This project is released under the "MIT license". Please see the file [LICENSE](https://github.com/edev/groupby/blob/master/LICENSE) in this distribution for license terms.
