# `onlyargs`

Only argument parsing! Nothing more.

## Why?

There's an [argument parsing crate for everyone](https://github.com/rosetta-rs/argparse-rosetta-rs). So why write another?

`onlyargs` is an example of extreme minimalism! The only thing it provides is a trait, and you're expected to do the actual work to implement it for your CLI argument struct. But don't let that scare you away! The parser implementation in the `full` example (see below) is only around 80 lines! (Most of the file is boilerplate.)

The goals of this parser are correctness, fast compile times, and convenience.

### Correctness

- The main parsing loop uses `OsString`, meaning that invalid UTF-8 can be accepted as an argument.
- Arguments can either be stored directly as an `OsString` or converted to a `PathBuf` with no extra cost. Easily access your [mojibake](https://en.wikipedia.org/wiki/Mojibake) file systems!
- Conversions from `OsString` are handled by your parser implementation. It's only as correct as you want it to be!
- Play with the examples. Try to break it. Have fun!

### Fast compile times

Here are some non-scientific benchmarks showing compile-time on my relatively modern AMD Ryzen 9 5900X machine running Windows 11, using Rust `1.70.0-nightly (1db9c061d 2023-03-21)` and `rust-lld` as the linker:

<details><summary>Debug builds (clean)</summary>

```bash
$ hyperfine -w 1 -p 'cargo clean' 'cargo build'
Benchmark 1: cargo build
  Time (mean ± σ):     199.3 ms ±   4.6 ms    [User: 40.3 ms, System: 35.2 ms]
  Range (min … max):   193.2 ms … 206.6 ms    10 runs
```

```bash
$ hyperfine -w 1 -p 'cargo clean' 'cargo build --example basic'
Benchmark 1: cargo build --example basic
  Time (mean ± σ):     405.3 ms ±  28.9 ms    [User: 87.5 ms, System: 66.6 ms]
  Range (min … max):   383.7 ms … 476.6 ms    10 runs
```

```bash
$ hyperfine -w 1 -p 'cargo clean' 'cargo build --example full'
Benchmark 1: cargo build --example full
  Time (mean ± σ):     514.0 ms ±  16.5 ms    [User: 165.6 ms, System: 85.0 ms]
  Range (min … max):   488.7 ms … 543.5 ms    10 runs
```

</details>

<details><summary>Debug builds (incremental)</summary>

```bash
$ hyperfine -w 1 -p 'touch src/lib.rs' 'cargo build'
Benchmark 1: cargo build
  Time (mean ± σ):     150.2 ms ±   9.8 ms    [User: 33.1 ms, System: 21.1 ms]
  Range (min … max):   141.3 ms … 174.5 ms    17 runs
```

```bash
$ hyperfine -w 1 -p 'touch examples/basic.rs' 'cargo build --example basic'
Benchmark 1: cargo build --example basic
  Time (mean ± σ):     244.2 ms ±  15.4 ms    [User: 38.8 ms, System: 40.4 ms]
  Range (min … max):   226.3 ms … 275.1 ms    12 runs
```

```bash
$ hyperfine -w 1 -p 'touch examples/full.rs' 'cargo build --example full'
Benchmark 1: cargo build --example full
  Time (mean ± σ):     325.1 ms ±  14.5 ms    [User: 60.6 ms, System: 59.4 ms]
  Range (min … max):   302.4 ms … 345.6 ms    10 runs
```

</details>

<details><summary>Release builds (clean)</summary>

```bash
$ hyperfine -w 1 -p 'cargo clean' 'cargo build --release'
Benchmark 1: cargo build --release
  Time (mean ± σ):     206.2 ms ±  23.2 ms    [User: 40.6 ms, System: 15.6 ms]
  Range (min … max):   184.0 ms … 262.5 ms    10 runs
```

```bash
$ hyperfine -w 1 -p 'cargo clean' 'cargo build --release --example basic'
Benchmark 1: cargo build --release --example basic
  Time (mean ± σ):     376.4 ms ±   9.1 ms    [User: 78.1 ms, System: 48.4 ms]
  Range (min … max):   362.9 ms … 395.3 ms    10 runs
```

```bash
$ hyperfine -w 1 -p 'cargo clean' 'cargo build --release --example full'
Benchmark 1: cargo build --release --example full
  Time (mean ± σ):     547.0 ms ±  11.6 ms    [User: 248.1 ms, System: 72.2 ms]
  Range (min … max):   530.1 ms … 566.1 ms    10 runs
```

</details>

<details><summary>Release builds (incremental)</summary>

```bash
$ hyperfine -w 1 -p 'touch src/lib.rs' 'cargo build --release'
Benchmark 1: cargo build --release
  Time (mean ± σ):     156.5 ms ±   9.5 ms    [User: 42.0 ms, System: 26.8 ms]
  Range (min … max):   145.9 ms … 183.0 ms    17 runs
```

```bash
$ hyperfine -w 1 -p 'touch examples/basic.rs' 'cargo build --release --example basic'
Benchmark 1: cargo build --release --example basic
  Time (mean ± σ):     254.3 ms ±   8.4 ms    [User: 54.0 ms, System: 12.6 ms]
  Range (min … max):   241.6 ms … 269.1 ms    11 runs
```

```bash
$ hyperfine -w 1 -p 'touch examples/full.rs' 'cargo build --release --example full'
Benchmark 1: cargo build --release --example full
  Time (mean ± σ):     418.2 ms ±   7.6 ms    [User: 248.4 ms, System: 32.6 ms]
  Range (min … max):   410.0 ms … 435.3 ms    10 runs
```

</details>

Worst case is about half a second on this setup.

### Convenience

Argument parsing is dead simple (assuming your preferred DSL is opinionated and no-nonsense). There is no reason overcomplicating it by supporting multiple forms like `--argument 123` and `--argument=123` or `-a 123` and `-a123`. Just pick one!

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
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target\debug\examples\basic.exe`
Arguments parsed successfully!
```

#### Run with `-h` arg

```bash
$ cargo run --example basic -- -h
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target\debug\examples\basic.exe -h`
onlyargs v0.1.0
Obsessively tiny argument parsing
```

#### Run with `--version` arg

```bash
$ cargo run --example basic -- --version
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target\debug\examples\basic.exe --version`
onlyargs v0.1.0
```


### [`full`](./examples/full.rs) example

This is a dumb "calculator" app with a full argument parser. It isn't realistic to demonstrate every possible way an argument parser could work, but this shows how to parse strings, integers, paths, optional args, and positional args.

#### Run with no args

```bash
$ cargo run --example full
   Compiling onlyargs v0.1.0 (C:\Users\jay\projects\onlyargs)
    Finished dev [unoptimized + debuginfo] target(s) in 0.24s
     Running `target\debug\examples\full.exe`
onlyargs v0.1.0
Obsessively tiny argument parsing

A basic argument parsing example with `onlyargs`.
Sums a list of numbers and writes the result to a file or standard output.

Usage:
  full [flags] [options] [numbers...]

Flags:
  -u, --username <name>  Your username.
  -o, --output [path]    Output file path.

Options:
  -h --help     Show this help message.
  --version     Show the application version.

Numbers:
  A list of numbers to sum.

Error: Argument parsing error
  Caused by: Missing required argument `username`
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
