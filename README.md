[![Crates.io](https://img.shields.io/crates/v/onlyargs)](https://crates.io/crates/onlyargs "Crates.io version")
[![Documentation](https://img.shields.io/docsrs/onlyargs)](https://docs.rs/onlyargs "Documentation")
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![GitHub actions](https://img.shields.io/github/actions/workflow/status/parasyte/onlyargs/ci.yml?branch=main)](https://github.com/parasyte/onlyargs/actions "CI")
[![GitHub activity](https://img.shields.io/github/last-commit/parasyte/onlyargs)](https://github.com/parasyte/onlyargs/commits "Commit activity")
[![GitHub Sponsors](https://img.shields.io/github/sponsors/parasyte)](https://github.com/sponsors/parasyte "Sponsors")

Only argument parsing! Nothing more.

# Why?

- 100% safe Rust 🦀.
- Correctness: Paths with invalid UTF-8 work correctly on all platforms.
- Fast compile times.
  - See [`myn` benchmark results](https://github.com/parasyte/myn/blob/main/benchmarks.md).
- Convenience: `#[derive(OnlyArgs)]` on a struct and parse CLI arguments from the environment into it with minimal boilerplate.

## MSRV Policy

The Minimum Supported Rust Version for `onlyargs` will always be made available in the [MSRV.md](./MSRV.md) file on GitHub.


# Rationale

There's an [argument parsing crate for everyone](https://github.com/rosetta-rs/argparse-rosetta-rs). So why write another?

`onlyargs` is an example of extreme minimalism! The only thing it provides is a trait and some utility functions; you're expected to do the actual work to implement it for your CLI argument struct. But don't let that scare you away! The parser implementation in the [`full` example](./examples/full.rs) is only around 50 lines! (Most of the file is boilerplate.)

The goals of this parser are correctness, fast compile times, and convenience.

## 100% safe Rust

No shenanigans! The only `unsafe` code is abstracted away in the standard library.

## Correctness

- The main parsing loop uses `OsString` so that invalid UTF-8 can be accepted as an argument.
- Arguments can either be stored directly as an `OsString` or converted to a `PathBuf` with no extra cost. Easily access your [mojibake](https://en.wikipedia.org/wiki/Mojibake) file systems!
- Conversions from `OsString` are handled by your parser implementation. It's only as correct as you want it to be!
- Play with the examples. Try to break it. Have fun!

## Fast compile times

See [`myn` benchmark results](https://github.com/parasyte/myn/blob/main/benchmarks.md).

## Convenience

Argument parsing is dead simple (assuming your preferred DSL is opinionated and no-nonsense). There is no reason to overcomplicate it by supporting multiple forms like `--argument 123` and `--argument=123` or `-a 123` and `-a123`. _Just pick one!_

The provided examples use the former in both cases: `--argument 123` and `-a 123` are accepted for arguments with a value. Supporting both long and short argument names is just a pattern!

```rust
Some("--argument") | Some("-a")
```

It is fairly straightforward to derive an implementation with a proc_macro. Compare the [`derive-example`](./examples/derive/src/main.rs) to the `full` example.
