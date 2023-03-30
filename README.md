# `onlyargs`

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

Here are some non-scientific benchmarks showing compile-time on my relatively modern AMD Ryzen 9 5900X machine running Windows 11, using Rust `1.70.0-nightly (17c116721 2023-03-29)` and `rust-lld` as the linker:

<details><summary>Debug builds (clean)</summary>

```bash
$ hyperfine -w 1 -p 'cargo clean' 'cargo build'
Benchmark 1: cargo build
  Time (mean ± σ):     200.8 ms ±   5.0 ms    [User: 43.8 ms, System: 29.7 ms]
  Range (min … max):   191.5 ms … 209.6 ms    10 runs
```

```bash
$ hyperfine -w 1 -p 'cargo clean' 'cargo build --example basic'
Benchmark 1: cargo build --example basic
  Time (mean ± σ):     384.6 ms ±   4.2 ms    [User: 107.8 ms, System: 94.7 ms]
  Range (min … max):   375.3 ms … 388.9 ms    10 runs
```

```bash
$ hyperfine -w 1 -p 'cargo clean' 'cargo build --example full'
Benchmark 1: cargo build --example full
  Time (mean ± σ):     487.9 ms ±  11.1 ms    [User: 132.8 ms, System: 66.2 ms]
  Range (min … max):   478.5 ms … 509.3 ms    10 runs
```

</details>

<details><summary>Debug builds (incremental)</summary>

```bash
$ hyperfine -w 1 -p 'touch src/lib.rs' 'cargo build'
Benchmark 1: cargo build
  Time (mean ± σ):     142.6 ms ±   6.1 ms    [User: 22.5 ms, System: 17.5 ms]
  Range (min … max):   136.9 ms … 162.4 ms    18 runs
```

```bash
$ hyperfine -w 1 -p 'touch examples/basic.rs' 'cargo build --example basic'
Benchmark 1: cargo build --example basic
  Time (mean ± σ):     228.5 ms ±   9.6 ms    [User: 31.2 ms, System: 31.2 ms]
  Range (min … max):   218.3 ms … 247.9 ms    12 runs
```

```bash
$ hyperfine -w 1 -p 'touch examples/full.rs' 'cargo build --example full'
Benchmark 1: cargo build --example full
  Time (mean ± σ):     295.1 ms ±   7.5 ms    [User: 57.8 ms, System: 59.4 ms]
  Range (min … max):   286.6 ms … 309.7 ms    10 runs
```

</details>

<details><summary>Release builds (clean)</summary>

```bash
$ hyperfine -w 1 -p 'cargo clean' 'cargo build --release'
Benchmark 1: cargo build --release
  Time (mean ± σ):     202.0 ms ±  12.8 ms    [User: 57.8 ms, System: 16.7 ms]
  Range (min … max):   181.8 ms … 222.8 ms    10 runs
```

```bash
$ hyperfine -w 1 -p 'cargo clean' 'cargo build --release --example basic'
Benchmark 1: cargo build --release --example basic
  Time (mean ± σ):     381.6 ms ±  13.4 ms    [User: 87.5 ms, System: 45.3 ms]
  Range (min … max):   367.0 ms … 402.3 ms    10 runs
```

```bash
$ hyperfine -w 1 -p 'cargo clean' 'cargo build --release --example full'
Benchmark 1: cargo build --release --example full
  Time (mean ± σ):     501.4 ms ±   8.9 ms    [User: 181.2 ms, System: 71.9 ms]
  Range (min … max):   491.1 ms … 520.7 ms    10 runs
```

</details>

<details><summary>Release builds (incremental)</summary>

```bash
$ hyperfine -w 1 -p 'touch src/lib.rs' 'cargo build --release'
Benchmark 1: cargo build --release
  Time (mean ± σ):     155.9 ms ±   4.8 ms    [User: 43.2 ms, System: 17.5 ms]
  Range (min … max):   149.8 ms … 166.0 ms    17 runs
```

```bash
$ hyperfine -w 1 -p 'touch examples/basic.rs' 'cargo build --release --example basic'
Benchmark 1: cargo build --release --example basic
  Time (mean ± σ):     245.0 ms ±  12.1 ms    [User: 56.2 ms, System: 26.6 ms]
  Range (min … max):   234.6 ms … 269.7 ms    10 runs
```

```bash
$ hyperfine -w 1 -p 'touch examples/full.rs' 'cargo build --release --example full'
Benchmark 1: cargo build --release --example full
  Time (mean ± σ):     369.4 ms ±   8.0 ms    [User: 148.4 ms, System: 45.3 ms]
  Range (min … max):   360.8 ms … 381.7 ms    10 runs
```

</details>

Worst case is about half a second on this setup.

### Convenience

Argument parsing is dead simple (assuming your preferred DSL is opinionated and no-nonsense). There is no reason to overcomplicate it by supporting multiple forms like `--argument 123` and `--argument=123` or `-a 123` and `-a123`. _Just pick one!_

The provided examples use the former in both cases: `--argument 123` and `-a 123` are accepted for arguments with a value. Supporting both long and short argument names is just a pattern!

```rust
Some("--argument") | Some("-a")
```

It is fairly straightforward to derive an implementation with a proc_macro. (`onlyargs_derive` is a work in progress.)

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
  -u --username <name>  Your username.
  -o --output [path]    Output file path.

Options:
  -h --help     Show this help message.
  -V --version  Show the application version.

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
  -u --username <name>  Your username.
  -o --output [path]    Output file path.

Options:
  -h --help     Show this help message.
  -V --version  Show the application version.

Numbers:
  A list of numbers to sum.

Error: Argument parsing error
  Caused by: Int parsing error for argument `<POSITIONAL>`: value="--output"
  Caused by: invalid digit found in string
error: process didn't exit successfully: `target\debug\examples\full.exe -u parasyte -- 1 2 3 --output sums.txt` (exit code: 1)
```
