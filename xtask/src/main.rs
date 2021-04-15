use anyhow::{anyhow, Result};
use pico_args::Arguments;
use std::{
    env,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

fn cargo() -> String {
    env::var("CARGO").unwrap_or_else(|_| "cargo".into())
}

fn project_root() -> PathBuf {
    Path::new(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(1)
    .unwrap()
    .to_path_buf()
}

fn cargo_clean_release() -> Result<()> {
    let status = Command::new(cargo())
        .args(&["clean", "--release"])
        .current_dir(project_root())
        .status()?;
    if !status.success() {
        return Err(anyhow!("'tests clean' failed"));
    }

    Ok(())
}

fn cargo_tests_npm_build() -> Result<String> {
    let output = Command::new(cargo())
        .args(&["run", "--release"])
        .current_dir(project_root().join("tests/npm-build"))
        .output()?;

    if !output.status.success() {
        io::stderr().write_all(&output.stderr)?;
        return Err(anyhow!("'tests npm-build run' failed"));
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn tests_npm_build() -> Result<()> {
    let run1 = cargo_tests_npm_build()?;
    let run2 = cargo_tests_npm_build()?;

    if run1 != run2 {
        return Err(anyhow!(
            "\
outputs of two sequentional 'npm-build' test runs do not match: {} != {}
This means build.rs was triggered second time but it should not.",
            run1,
            run2
        ));
    }

    Ok(())
}

fn main() -> Result<()> {
    let mut args = Arguments::from_env();
    let subcommand = args.subcommand()?.unwrap_or_default();

    match subcommand.as_str() {
        "tests" => {
            args.finish();

            cargo_clean_release()?;

            tests_npm_build()?;
        }
        _ => {
            eprintln!(
                "\
cargo xtask
Run custom build command.
USAGE:
    cargo xtask <SUBCOMMAND>
SUBCOMMANDS:
    tests"
            );
        }
    }

    Ok(())
}
