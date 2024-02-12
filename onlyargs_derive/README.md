[![Crates.io](https://img.shields.io/crates/v/onlyargs_derive)](https://crates.io/crates/onlyargs_derive "Crates.io version")
[![Documentation](https://img.shields.io/docsrs/onlyargs_derive)](https://docs.rs/onlyargs_derive "Documentation")
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

The Minimum Supported Rust Version for `onlyargs` will always be made available in the [MSRV.md](../MSRV.md) file on GitHub.

# Examples

See the [`derive-example`](../examples/full-derive/src/main.rs) for usage.
