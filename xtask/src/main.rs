use anyhow::{anyhow, Result};
use pico_args::Arguments;
use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    thread::sleep,
    time::Duration,
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

fn tests_npm_build_without_src_changes() -> Result<()> {
    eprint!("tests::npm-build::without_src_changes...");

    cargo_clean_release()?;

    let run1 = cargo_tests_npm_build()?;

    cooldown_between_builds();

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

    eprintln!("ok");

    Ok(())
}

fn tests_npm_build_with_src_changes() -> Result<()> {
    eprint!("tests::npm-build::with_src_changes...");

    cargo_clean_release()?;

    let run1 = cargo_tests_npm_build()?;

    cooldown_between_builds();

    fs::write(
        project_root().join("tests/npm-build/web/src/index.js"),
        r#"let a = 1;"#,
    )?;

    let run2 = cargo_tests_npm_build()?;

    if run1 == run2 {
        return Err(anyhow!(
            "\
outputs of two sequentional 'npm-build' test runs should not match: {} == {}
This means build.rs was not triggered second time but it must.",
            run1,
            run2
        ));
    }

    eprintln!("ok");

    Ok(())
}

fn cooldown_between_builds() {
    sleep(Duration::from_secs(2));
}

fn main() -> Result<()> {
    let mut args = Arguments::from_env();
    let subcommand = args.subcommand()?.unwrap_or_default();

    match subcommand.as_str() {
        "tests" => {
            args.finish();

            tests_npm_build_without_src_changes()?;

            tests_npm_build_with_src_changes()?;
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
