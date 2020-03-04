# Project write-up

*Team members: Dylan Laufenberg*

This project consists of a library for grouping items of arbitrary types and an application that uses the library to group lines of text input according to user-specified parameters, optionally running a shell command on each group.

## Related work

To the best of my knowledge, there are no similar projects publicly available in the POSIX ecosystem. The `find` utility is invaluable for searching directory paths but cannot group its results. The `xargs` utility blurs the definition of a terminal filter in similar ways to GroupBy. `sort`, `uniq`, and `awk` can perform some similar functions, but none of the above are able to divide lines of input into groups and invoke a command sequence once for each group, passing that group's full text as standard input. To the best of my knowledge, prior to the development of GroupBy,there was no way to perform this task automatically without writing code.

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

## Reflections

Generally, everything went swimmingly and works well. I feel good about writing this particular application in Rust; to me, it feels like a good fit. I particularly appreciate that I won't need to manage an interpreter installation or install libraries to use GroupBy on any of my systems.

Clap has great documentation. I especially appreciated the respectful comparisons to other systems. Unfortunately, I was disappointed to find that this seemingly heavyweight tool lacked the depth of options that I needed. I'm disappointed with the current state of GroupBy's help output, and I intend to search for an alternative to Clap in the future. I was not able to match my initial prototype's help output with Clap.

The ultimate triumph for me, personally, is that I finally have a way to automatically group text files by the years embedded in their filenames and count the lines, words, and characters in each group. I have an actively growing collection of about 2.5 million words across a thousand or more files, and I have wanted for years to build an application to serve this exact purpose. I'm also honestly thrilled to have built a Linux filter that cooperates and fits in well with `find`, `wc`, `xargs`, and so on! I feel the strangest sense of accomplishment as a Linux power user.

My biggest lesson learned was actually domain-specific: I learned that almost everything I could reasonably want to have as a command-line option for grouping text could be expressed as a regular expression. I would like to add a dozen or more matchers and groupers for common groupings, but in the most technical sense, they won't add any new features to GroupBy. In fact, in this sense, two of my three matchers are redundant, though I maintain it's a lot more convenient and accessible to write `-f 3` than `^.{3}` to match the first three characters of a line.

I also learned more about Unicode and its interaction with regular expressions. I have never before given thought to supporting arbitrary languages in my regular expressions, so this project has opened my eyes.

