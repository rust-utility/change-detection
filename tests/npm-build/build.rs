use change_detection::{
    path_matchers::{equal, func, PathMatcherExt},
    ChangeDetection,
};
use std::{
    env, fs,
    io::Result,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let web_pathbuf = PathBuf::from("web");

    ChangeDetection::path_exclude(
        web_pathbuf.clone(),
        equal(web_pathbuf.clone())
            .or(equal(web_pathbuf.join("package-lock.json")))
            .or(func(move |p| {
                p.starts_with(web_pathbuf.join("dist"))
                    || (p.is_file() && p.parent() != Some(web_pathbuf.as_path()))
            })),
    )
    .generate();

    let out_dir = env::var("OUT_DIR").unwrap();
    let generated_file = Path::new(&out_dir).join("generated.in");

    let version = 1 + if generated_file.exists() {
        fs::read_to_string(&generated_file)?
            .parse::<usize>()
            .unwrap()
    } else {
        0
    };

    fs::write(generated_file, version.to_string())?;

    fs::write("web/package-lock.json", r#"{"version":"0.1.0"}"#)?;
    fs::write("web/dist/app/index.js", r#"let a = 1;"#)?;

    Ok(())
}
