use onlyargs::{CliError, OnlyArgs as _};
use onlyargs_derive::OnlyArgs;
use std::{ffi::OsString, path::PathBuf};

#[test]
fn test_multivalue_paths() -> Result<(), CliError> {
    #[derive(Debug, OnlyArgs)]
    struct Args {
        path: Vec<PathBuf>,
    }

    let args = Args::parse(
        [
            "--path",
            "/tmp/hello",
            "--path",
            "/var/run/test.pid",
            "--path",
            "./foo/bar with spaces/",
        ]
        .into_iter()
        .map(OsString::from)
        .collect(),
    )?;

    assert_eq!(
        args.path,
        [
            PathBuf::from("/tmp/hello"),
            PathBuf::from("/var/run/test.pid"),
            PathBuf::from("./foo/bar with spaces/"),
        ]
    );

    Ok(())
}

#[test]
fn test_multivalue_with_positional() -> Result<(), CliError> {
    #[derive(Debug, OnlyArgs)]
    struct Args {
        names: Vec<String>,

        #[positional]
        rest: Vec<String>,
    }

    let args = Args::parse(
        ["--names", "Alice", "--names", "Bob", "Carol", "David"]
            .into_iter()
            .map(OsString::from)
            .collect(),
    )?;

    assert_eq!(args.names, ["Alice", "Bob"]);
    assert_eq!(args.rest, ["Carol", "David"]);

    Ok(())
}

#[test]
fn test_required_multivalue() -> Result<(), CliError> {
    #[derive(Debug, OnlyArgs)]
    struct Args {
        #[required]
        names: Vec<String>,
    }

    // Empty `--names` is not allowed.
    assert!(matches!(
        Args::parse(vec![]),
        Err(CliError::MissingRequired(name)) if name == "--names",
    ));

    // At least one `--names` is required.
    let args = Args::parse(
        ["--names", "Alice"]
            .into_iter()
            .map(OsString::from)
            .collect(),
    )?;

    assert_eq!(args.names, ["Alice"]);

    Ok(())
}
