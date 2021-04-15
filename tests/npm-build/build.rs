use change_detection::ChangeDetection;
use std::{env, fs, io::Result, path::Path};

fn main() -> Result<()> {
    ChangeDetection::path("build.rs")
        .path("fixtures-04")
        .generate();

    let out_dir = env::var("OUT_DIR").unwrap();
    let generated_file = Path::new(&out_dir).join("generated.in");

    let version = if generated_file.exists() {
        fs::read_to_string(&generated_file)?
            .parse::<usize>()
            .unwrap()
    } else {
        0
    } + 1;

    fs::write(generated_file, version.to_string())?;
    fs::write(
        "fixtures-04/package-lock.json",
        r#"{"version":"0.1.0"}"#,
    )?;

    Ok(())
}
