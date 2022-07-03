# GroupBy

GroupBy is a CLI application and supporting library for grouping inputs (especially text) by common keys, i.e. partitioning them into equivalence classes.

The `groupby` program is a terminal filter that groups lines (or other tokens) of text according to various matchers: first/last `n` characters, file extension, regular expression, etc. It can also run shell commands over groups, acting as a kind of splitting terminal filter.

The GroupBy library provides support for grouping items of arbitrary types by common keys. It currently only provides methods to group strings but is designed to work over arbitrary types. It also provides extensive support for terminal applications that wish to borrow some or all of this functionality.

## Whitepaper

This project started as a term project at Portland State University. One of the original deliverables was a whitepaper, available in this repository as [WHITEPAPER.pdf](https://github.com/edev/groupby/blob/master/WHITEPAPER.pdf). It provides a detailed discussion of the problem this project aims to solve. (Note that performance figures and other details may have changed. The paper as a whole, however, is still accurate and a worthwhile read.)

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

* `-c "xargs ..."` invokes the quoted command sequence *using the current user's current shell* once for each group. It runs these command sequences in parallel across all available CPU cores, storing their standard outputs. After all commands have finished, `groupby` prints the standard output for each group under a header with the group's name.

* `xargs` reads from standard input, building up arguments to pass to a further command. It passes those arguments to `wc | tail -n 1`.

* Since `xargs` delimits arguments in its standard input by whitespace (e.g. space characters, newlines, etc.), it won't correctly parse our note file paths, which contain spaces. As a workaround, it provides the `-0` flag to delimit arguments by null characters instead of whitespace. The `find` command has a `-print0` option to delimit its output entries by null characters instead of newlines; `groupy` mimics this behavior with its `--print0` option for the same reason.

* `wc` counts the lines, words, and characters, then passes that information on to the `tail` filter, which (when used with the `-n 1` option) outputs only the summary line from `wc`.

## Getting help

To see the currently supported options, run `groupby -h` to see a short listing of options or `groupby --help` to see a full listing.

## License

This project is released under the "MIT license". Please see the file [LICENSE](https://github.com/edev/groupby/blob/master/LICENSE) in this distribution for license terms.
