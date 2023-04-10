[![Crates.io](https://img.shields.io/crates/v/onlyargs)](https://crates.io/crates/onlyargs "Crates.io version")
[![Documentation](https://img.shields.io/docsrs/onlyargs)](https://docs.rs/onlyargs "Documentation")
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![GitHub actions](https://img.shields.io/github/actions/workflow/status/parasyte/onlyargs/ci.yml?branch=main)](https://github.com/parasyte/onlyargs/actions "CI")
[![GitHub activity](https://img.shields.io/github/last-commit/parasyte/onlyargs)](https://github.com/parasyte/onlyargs/commits "Commit activity")
[![GitHub Sponsors](https://img.shields.io/github/sponsors/parasyte)](https://github.com/sponsors/parasyte "Sponsors")

Only argument parsing! Nothing more.

## Why?

There's an [argument parsing crate for everyone](https://github.com/rosetta-rs/argparse-rosetta-rs). So why write another?

`onlyargs` is an example of extreme minimalism! The only thing it provides is a trait and some utility functions; you're expected to do the actual work to implement it for your CLI argument struct. But don't let that scare you away! The parser implementation in the `full` example (see below) is only around 50 lines! (Most of the file is boilerplate.)

The goals of this parser are correctness, fast compile times, and convenience.

### Correctness

- The main parsing loop uses `OsString`, meaning that invalid UTF-8 can be accepted as an argument.
- Arguments can either be stored directly as an `OsString` or converted to a `PathBuf` with no extra cost. Easily access your [mojibake](https://en.wikipedia.org/wiki/Mojibake) file systems!
- Conversions from `OsString` are handled by your parser implementation. It's only as correct as you want it to be!
- Play with the examples. Try to break it. Have fun!

### Fast compile times

See [`myn` benchmark results](https://github.com/parasyte/myn/blob/main/benchmarks.md).

### Convenience

Argument parsing is dead simple (assuming your preferred DSL is opinionated and no-nonsense). There is no reason to overcomplicate it by supporting multiple forms like `--argument 123` and `--argument=123` or `-a 123` and `-a123`. _Just pick one!_

The provided examples use the former in both cases: `--argument 123` and `-a 123` are accepted for arguments with a value. Supporting both long and short argument names is just a pattern!

```rust
Some("--argument") | Some("-a")
```

It is fairly straightforward to derive an implementation with a proc_macro. See [`onlyargs_derive`](./onlyargs_derive) for an example.

## Examples

### [`basic`](./examples/basic.rs) example

This only parses `--help|-h` and `--version` arguments. It's just here to introduce you to the trait.

#### Run with no args

```bash
$ cargo run --example basic
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target\debug\examples\basic.exe`
Arguments parsed successfully!
```

#### Run with `-h` arg

```bash
$ cargo run --example basic -- -h
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target\debug\examples\basic.exe -h`
onlyargs v0.1.0
Obsessively tiny argument parsing
```

#### Run with `--version` arg

```bash
$ cargo run --example basic -- --version
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target\debug\examples\basic.exe --version`
onlyargs v0.1.0
```


### [`full`](./examples/full.rs) example

This is a dumb "calculator" app with a full argument parser. It isn't realistic to demonstrate every possible way an argument parser could work, but this shows how to parse strings, integers, paths, optional args, and positional args.

#### Run with no args

```bash
$ cargo run --example full
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target\debug\examples\full.exe`
onlyargs v0.1.0
Obsessively tiny argument parsing

A basic argument parsing example with `onlyargs`.
Sums a list of numbers and writes the result to a file or standard output.

Usage:
  full [flags] [options] [numbers...]

Flags:
  -h --help     Show this help message.
  -V --version  Show the application version.

Options:
  -u --username <name>  Your username.
  -o --output [path]    Output file path.

Numbers:
  A list of numbers to sum.

Error: Argument parsing error
  Caused by: Missing required argument `--username`
error: process didn't exit successfully: `target\debug\examples\full.exe` (exit code: 1)
```

#### Run with `-u` argument

```bash
$ cargo run --example full -- -u parasyte
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target\debug\examples\full.exe -u parasyte`
Hello, parasyte!

[examples\full.rs:187] args = Args {
    username: "parasyte",
    output: None,
    numbers: [],
    help: false,
    version: false,
}
```

#### Run with positional arguments

```bash
$ cargo run --example full -- -u parasyte 1 2 3
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target\debug\examples\full.exe -u parasyte 1 2 3`
Hello, parasyte!
The sum of 1 + 2 + 3 is 6

[examples\full.rs:188] args = Args {
    username: "parasyte",
    output: None,
    numbers: [
        1,
        2,
        3,
    ],
    help: false,
    version: false,
}
```

#### Run with `--output` argument

```bash
$ cargo run --example full -- -u parasyte 1 2 3 --output sums.txt
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target\debug\examples\full.exe -u parasyte 1 2 3 --output sums.txt`
Hello, parasyte!
Sums written to "sums.txt"

[examples\full.rs:188] args = Args {
    username: "parasyte",
    output: Some(
        "sums.txt",
    ),
    numbers: [
        1,
        2,
        3,
    ],
    help: false,
    version: false,
}

$ cat sums.txt
The sum of 1 + 2 + 3 is 6
```

#### Use the `--` sentinel to try dumping all trailing args to the `numbers` field

```bash
$ cargo run --example full -- -u parasyte -- 1 2 3 --output sums.txt
    Finished dev [unoptimized + debuginfo] target(s) in 0.00s
     Running `target\debug\examples\full.exe -u parasyte -- 1 2 3 --output sums.txt`
onlyargs v0.1.0
Obsessively tiny argument parsing

A basic argument parsing example with `onlyargs`.
Sums a list of numbers and writes the result to a file or standard output.

Usage:
  full [flags] [options] [numbers...]

Flags:
  -h --help     Show this help message.
  -V --version  Show the application version.

Options:
  -u --username <name>  Your username.
  -o --output [path]    Output file path.

Numbers:
  A list of numbers to sum.

Error: Argument parsing error
  Caused by: Int parsing error for argument `<POSITIONAL>`: value="--output"
  Caused by: invalid digit found in string
error: process didn't exit successfully: `target\debug\examples\full.exe -u parasyte -- 1 2 3 --output sums.txt` (exit code: 1)
```
