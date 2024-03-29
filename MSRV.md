# Minimum Supported Rust Version

| `onlyargs` version | `rustc` version |
|--------------------|-----------------|
| (unreleased)       | `1.62.0`        |
| `0.2.0`            | `1.62.0`        |
| `0.1.3`            | `1.62.0`        |
| `0.1.2`            | `1.62.0`        |
| `0.1.1`            | `1.62.0`        |
| `0.1.0`            | `1.62.0`        |

## Policy

The table above will be kept up-to-date in lock-step with CI on the main branch in GitHub. It may contain information about unreleased and yanked versions. It is the user's responsibility to consult with the [`onlyargs` versions page](https://crates.io/crates/onlyargs/versions) on `crates.io` to verify version status.

The MSRV will be chosen as the minimum version of `rustc` that can successfully pass CI, including documentation, lints, and all examples. For this reason, the minimum version _supported_ may be higher than the minimum version _required_ to compile the `onlyargs` crate itself. See `Cargo.toml` for the minimal Rust version required to build the crate alone.
