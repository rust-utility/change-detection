/*!
# A library to generate change detection instructions during build time

## Legal

Dual-licensed under `MIT` or the [UNLICENSE](http://unlicense.org/).

## Features

Automates task of generating change detection instructions for your static files.

<https://doc.rust-lang.org/cargo/reference/build-scripts.html#change-detection>

## Usage

Add dependency to Cargo.toml:

```toml
[dependencies]
change-detection = "1.1"
```

Add a call to `build.rs`:

```rust
use change_detection::ChangeDetection;

fn main() {
    ChangeDetection::path("src/hello.c").generate();
}
```

This is basically the same, as just write:

```rust
fn main() {
    println!("cargo:rerun-if-changed=src/hello.c");
}
```

You can also use a directory. For example, if your resources are in `static` directory:

```rust
use change_detection::ChangeDetection;

fn main() {
    ChangeDetection::path("static").generate();
}
```

One call to generate can have multiple `path` components:

```rust
use change_detection::ChangeDetection;

fn main() {
    ChangeDetection::path("static")
        .path("another_path")
        .path("build.rs")
        .generate();
}
```

Using `path-matchers` library you can specify include / exclude filters:

```rust
#[cfg(features = "glob")]
use change_detection::{path_matchers::glob, ChangeDetection};

fn main() {
    #[cfg(features = "glob")]
    ChangeDetection::exclude(glob("another_path/**/*.tmp").unwrap())
        .path("static")
        .path("another_path")
        .path("build.rs")
        .generate();
}
```

You can actual generated result with this command:

```bash
find . -name output | xargs cat
```

*/
use ::path_matchers::PathMatcher;
use path_slash::PathExt;
use std::path::{Path, PathBuf};

/// Reexport `path-matchers`.
pub mod path_matchers {
    pub use ::path_matchers::*;
}

/// A change detection entry point.
///
/// Creates a builder to generate change detection instructions.
///
/// # Examples
///
/// ```
/// use change_detection::ChangeDetection;
///
/// fn main() {
///     ChangeDetection::path("src/hello.c").generate();
/// }
/// ```
///
/// This is the same as just write:
///
/// ```ignore
/// fn main() {
///     println!("cargo:rerun-if-changed=src/hello.c");
/// }
/// ```
///
/// You can collect resources from a path:
///
/// ```
/// # use change_detection::ChangeDetection;
///
/// fn main() {
///     ChangeDetection::path("some_path").generate();
/// }
/// ```
///
/// To chain multiple directories and files:
///
/// ```
/// # use change_detection::ChangeDetection;
///
/// fn main() {
///     ChangeDetection::path("src/hello.c")
///         .path("static")
///         .path("build.rs")
///         .generate();
/// }
/// ```
pub struct ChangeDetection;

impl ChangeDetection {
    /// Collects change detection instructions from a `path`.
    ///
    /// A `path` can be a single file or a directory.
    ///
    /// # Examples:
    ///
    /// To generate change instructions for the directory with the name `static`:
    ///
    /// ```
    /// # use change_detection::ChangeDetection;
    /// ChangeDetection::path("static").generate();
    /// ```
    ///
    /// To generate change instructions for the file with the name `build.rs`:
    ///
    /// ```
    /// # use change_detection::ChangeDetection;
    /// ChangeDetection::path("build.rs").generate();
    /// ```
    pub fn path<P>(path: P) -> ChangeDetectionBuilder
    where
        P: AsRef<Path>,
    {
        ChangeDetectionBuilder::default().path(path)
    }

    /// Collects change detection instructions from a `path` applying include `filter`.
    ///
    /// A `path` can be a single file or a directory.
    ///
    /// # Examples:
    ///
    /// To generate change instructions for the directory with the name `static` but only for files ending with `b`:
    ///
    /// ```
    /// # use change_detection::ChangeDetection;
    /// ChangeDetection::path_include("static", |path: &std::path::Path| {
    ///     path.file_name()
    ///         .map(|filename| filename.to_str().unwrap().ends_with("b"))
    ///         .unwrap_or(false)
    /// }).generate();
    /// ```
    pub fn path_include<P, F>(path: P, filter: F) -> ChangeDetectionBuilder
    where
        P: AsRef<Path>,
        F: PathMatcher + 'static,
    {
        ChangeDetectionBuilder::default().path_include(path, filter)
    }

    /// Collects change detection instructions from a `path` applying exclude `filter`.
    ///
    /// A `path` can be a single file or a directory.
    ///
    /// # Examples:
    ///
    /// To generate change instructions for the directory with the name `static` but without files ending with `b`:
    ///
    /// ```
    /// # use change_detection::ChangeDetection;
    /// ChangeDetection::path_exclude("static", |path: &std::path::Path| {
    ///     path.file_name()
    ///         .map(|filename| filename.to_str().unwrap().ends_with("b"))
    ///         .unwrap_or(false)
    /// }).generate();
    /// ```
    pub fn path_exclude<P, F>(path: P, filter: F) -> ChangeDetectionBuilder
    where
        P: AsRef<Path>,
        F: PathMatcher + 'static,
    {
        ChangeDetectionBuilder::default().path_exclude(path, filter)
    }

    /// Collects change detection instructions from a `path` applying `include` and `exclude` filters.
    ///
    /// A `path` can be a single file or a directory.
    ///
    /// # Examples:
    ///
    /// To generate change instructions for the directory with the name `static` including only files starting with `a` but without files ending with `b`:
    ///
    /// ```
    /// # use change_detection::ChangeDetection;
    /// ChangeDetection::path_filter("static", |path: &std::path::Path| {
    ///     path.file_name()
    ///         .map(|filename| filename.to_str().unwrap().starts_with("a"))
    ///         .unwrap_or(false)
    /// }, |path: &std::path::Path| {
    ///     path.file_name()
    ///         .map(|filename| filename.to_str().unwrap().ends_with("b"))
    ///         .unwrap_or(false)
    /// }).generate();
    /// ```
    pub fn path_filter<P, F1, F2>(path: P, include: F1, exclude: F2) -> ChangeDetectionBuilder
    where
        P: AsRef<Path>,
        F1: PathMatcher + 'static,
        F2: PathMatcher + 'static,
    {
        ChangeDetectionBuilder::default().path_filter(path, include, exclude)
    }

    /// Applies a global `include` filter to all paths.
    ///
    /// # Examples:
    ///
    /// To included only files starting with `a` for paths `static1`, `static2` and `static3`:
    ///
    /// ```
    /// # use change_detection::ChangeDetection;
    /// ChangeDetection::include(|path: &std::path::Path| {
    ///         path.file_name()
    ///             .map(|filename| filename.to_str().unwrap().starts_with("a"))
    ///             .unwrap_or(false)
    ///     })
    ///     .path("static1")
    ///     .path("static2")
    ///     .path("static3")
    ///     .generate();
    /// ```
    pub fn include<F>(filter: F) -> ChangeDetectionBuilder
    where
        F: PathMatcher + 'static,
    {
        ChangeDetectionBuilder::default().include(filter)
    }

    /// Applies a global `exclude` filter to all paths.
    ///
    /// # Examples:
    ///
    /// To exclude files starting with `a` for paths `static1`, `static2` and `static3`:
    ///
    /// ```
    /// # use change_detection::ChangeDetection;
    /// ChangeDetection::exclude(|path: &std::path::Path| {
    ///         path.file_name()
    ///             .map(|filename| filename.to_str().unwrap().starts_with("a"))
    ///             .unwrap_or(false)
    ///     })
    ///     .path("static1")
    ///     .path("static2")
    ///     .path("static3")
    ///     .generate();
    /// ```
    pub fn exclude<F>(filter: F) -> ChangeDetectionBuilder
    where
        F: PathMatcher + 'static,
    {
        ChangeDetectionBuilder::default().exclude(filter)
    }

    /// Applies a global `include` and `exclude` filters to all paths.
    ///
    /// # Examples:
    ///
    /// To include files starting with `a` for paths `static1`, `static2` and `static3`, but whose names do not end in `b`:
    ///
    /// ```
    /// # use change_detection::ChangeDetection;
    /// ChangeDetection::filter(|path: &std::path::Path| {
    ///         path.file_name()
    ///             .map(|filename| filename.to_str().unwrap().starts_with("a"))
    ///             .unwrap_or(false)
    ///     }, |path: &std::path::Path| {
    ///         path.file_name()
    ///             .map(|filename| filename.to_str().unwrap().ends_with("b"))
    ///             .unwrap_or(false)
    ///     })
    ///     .path("static1")
    ///     .path("static2")
    ///     .path("static3")
    ///     .generate();
    /// ```
    pub fn filter<F1, F2>(include: F1, exclude: F2) -> ChangeDetectionBuilder
    where
        F1: PathMatcher + 'static,
        F2: PathMatcher + 'static,
    {
        ChangeDetectionBuilder::default()
            .include(include)
            .exclude(exclude)
    }
}

/// A change detection builder.
///
/// A builder to generate change detection instructions.
/// You should not use this directly, use [`ChangeDetection`] as an entry point instead.
#[derive(Default)]
pub struct ChangeDetectionBuilder {
    include: Option<Box<dyn PathMatcher>>,
    exclude: Option<Box<dyn PathMatcher>>,
    paths: Vec<ChangeDetectionPath>,
}

impl ChangeDetectionBuilder {
    /// Collects change detection instructions from a `path`.
    ///
    /// A `path` can be a single file or a directory.
    ///
    /// # Examples:
    ///
    /// To generate change instructions for the directory with the name `static`:
    ///
    /// ```
    /// # use change_detection::ChangeDetectionBuilder;
    /// # let builder = ChangeDetectionBuilder::default();
    /// builder.path("static").generate();
    /// ```
    ///
    /// To generate change instructions for the file with the name `build.rs`:
    ///
    /// ```
    /// # use change_detection::ChangeDetectionBuilder;
    /// # let builder = ChangeDetectionBuilder::default();
    /// builder.path("build.rs").generate();
    /// ```
    pub fn path<P>(mut self, path: P) -> ChangeDetectionBuilder
    where
        P: Into<ChangeDetectionPath>,
    {
        self.paths.push(path.into());
        self
    }

    /// Collects change detection instructions from a `path` applying include `filter`.
    ///
    /// A `path` can be a single file or a directory.
    ///
    /// # Examples:
    ///
    /// To generate change instructions for the directory with the name `static` but only for files ending with `b`:
    ///
    /// ```
    /// # use change_detection::ChangeDetectionBuilder;
    /// # let builder = ChangeDetectionBuilder::default();
    /// builder.path_include("static", |path: &std::path::Path| {
    ///     path.file_name()
    ///         .map(|filename| filename.to_str().unwrap().ends_with("b"))
    ///         .unwrap_or(false)
    /// }).generate();
    /// ```
    pub fn path_include<P, F>(mut self, path: P, filter: F) -> ChangeDetectionBuilder
    where
        P: AsRef<Path>,
        F: PathMatcher + 'static,
    {
        self.paths.push(ChangeDetectionPath::PathInclude(
            path.as_ref().into(),
            Box::new(filter),
        ));
        self
    }

    /// Collects change detection instructions from a `path` applying exclude `filter`.
    ///
    /// A `path` can be a single file or a directory.
    ///
    /// # Examples:
    ///
    /// To generate change instructions for the directory with the name `static` but without files ending with `b`:
    ///
    /// ```
    /// # use change_detection::ChangeDetectionBuilder;
    /// # let builder = ChangeDetectionBuilder::default();
    /// builder.path_exclude("static", |path: &std::path::Path| {
    ///     path.file_name()
    ///         .map(|filename| filename.to_str().unwrap().ends_with("b"))
    ///         .unwrap_or(false)
    /// }).generate();
    /// ```
    pub fn path_exclude<P, F>(mut self, path: P, filter: F) -> ChangeDetectionBuilder
    where
        P: AsRef<Path>,
        F: PathMatcher + 'static,
    {
        self.paths.push(ChangeDetectionPath::PathExclude(
            path.as_ref().into(),
            Box::new(filter),
        ));
        self
    }

    /// Collects change detection instructions from a `path` applying `include` and `exclude` filters.
    ///
    /// A `path` can be a single file or a directory.
    ///
    /// # Examples:
    ///
    /// To generate change instructions for the directory with the name `static` including only files starting with `a` but without files ending with `b`:
    ///
    /// ```
    /// # use change_detection::ChangeDetectionBuilder;
    /// # let builder = ChangeDetectionBuilder::default();
    /// builder.path_filter("static", |path: &std::path::Path| {
    ///     path.file_name()
    ///         .map(|filename| filename.to_str().unwrap().starts_with("a"))
    ///         .unwrap_or(false)
    /// }, |path: &std::path::Path| {
    ///     path.file_name()
    ///         .map(|filename| filename.to_str().unwrap().ends_with("b"))
    ///         .unwrap_or(false)
    /// }).generate();
    /// ```
    pub fn path_filter<P, F1, F2>(
        mut self,
        path: P,
        include: F1,
        exclude: F2,
    ) -> ChangeDetectionBuilder
    where
        P: AsRef<Path>,
        F1: PathMatcher + 'static,
        F2: PathMatcher + 'static,
    {
        self.paths.push(ChangeDetectionPath::PathIncludeExclude {
            path: path.as_ref().into(),
            include: Box::new(include),
            exclude: Box::new(exclude),
        });
        self
    }

    fn include<F>(mut self, filter: F) -> ChangeDetectionBuilder
    where
        F: PathMatcher + 'static,
    {
        self.include = Some(Box::new(filter));
        self
    }

    fn exclude<F>(mut self, filter: F) -> ChangeDetectionBuilder
    where
        F: PathMatcher + 'static,
    {
        self.exclude = Some(Box::new(filter));
        self
    }

    pub fn generate(self) {
        self.generate_extended(print_change_detection_instruction)
    }

    fn generate_extended<F>(self, mut f: F)
    where
        F: FnMut(&Path),
    {
        for path in &self.paths {
            path.generate(&self, &mut f);
        }
    }

    fn filter_include_exclude(&self, path: &Path) -> bool {
        self.include
            .as_ref()
            .map_or(true, |filter| filter.matches(path))
            && self
                .exclude
                .as_ref()
                .map_or(true, |filter| !filter.matches(path))
    }
}

pub enum ChangeDetectionPath {
    Path(PathBuf),
    PathInclude(PathBuf, Box<dyn PathMatcher>),
    PathExclude(PathBuf, Box<dyn PathMatcher>),
    PathIncludeExclude {
        path: PathBuf,
        include: Box<dyn PathMatcher>,
        exclude: Box<dyn PathMatcher>,
    },
}

fn print_change_detection_instruction(path: &Path) {
    println!(
        "cargo:rerun-if-changed={}",
        path.to_slash().expect("can't convert path to utf-8 string")
    );
}

impl ChangeDetectionPath {
    fn collect(&self, builder: &ChangeDetectionBuilder) -> std::io::Result<Vec<PathBuf>> {
        let filter_fn: Box<dyn Fn(&_) -> bool> =
            Box::new(|path: &std::path::Path| builder.filter_include_exclude(path));

        let (path, filter): (&PathBuf, Box<dyn Fn(&_) -> bool>) = match self {
            ChangeDetectionPath::Path(path) => (path, filter_fn),
            ChangeDetectionPath::PathInclude(path, include_filter) => (
                path,
                Box::new(move |p: &Path| filter_fn(p.as_ref()) && include_filter.matches(p)),
            ),
            ChangeDetectionPath::PathExclude(path, exclude_filter) => (
                path,
                Box::new(move |p: &Path| filter_fn(p.as_ref()) && !exclude_filter.matches(p)),
            ),
            ChangeDetectionPath::PathIncludeExclude {
                path,
                include,
                exclude,
            } => (
                path,
                Box::new(move |p: &Path| {
                    filter_fn(p.as_ref()) && include.matches(p) && !exclude.matches(p)
                }),
            ),
        };

        collect_resources(path, &filter)
    }

    fn generate<F>(&self, builder: &ChangeDetectionBuilder, printer: &mut F)
    where
        F: FnMut(&Path),
    {
        for path in self.collect(builder).expect("error collecting resources") {
            printer(path.as_ref());
        }
    }
}

impl<T> From<T> for ChangeDetectionPath
where
    T: AsRef<Path>,
{
    fn from(path: T) -> Self {
        ChangeDetectionPath::Path(path.as_ref().into())
    }
}

fn collect_resources(path: &Path, filter: &dyn PathMatcher) -> std::io::Result<Vec<PathBuf>> {
    let mut result = vec![];

    if !path.exists() {
        return Ok(result);
    }

    result.push(path.into());

    if !path.is_dir() {
        return Ok(result);
    }

    for entry in std::fs::read_dir(&path)? {
        let entry = entry?;
        let path = entry.path();

        if !filter.matches(path.as_ref()) {
            continue;
        }

        if path.is_dir() {
            let nested = collect_resources(path.as_ref(), filter)?;
            result.extend(nested);
        }

        result.push(path);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{ChangeDetection, ChangeDetectionBuilder};
    use std::path::{Path, PathBuf};

    fn assert_change_detection(builder: ChangeDetectionBuilder, expected: &[&str]) {
        let mut result: Vec<PathBuf> = vec![];
        let r = &mut result;

        builder.generate_extended(move |path| r.push(path.into()));

        let mut expected = expected
            .iter()
            .map(|s| PathBuf::from(s))
            .collect::<Vec<_>>();

        expected.sort();
        result.sort();

        assert_eq!(result, expected);
    }

    #[test]
    fn single_file() {
        assert_change_detection(ChangeDetection::path("src/lib.rs"), &["src/lib.rs"]);
    }

    #[test]
    fn single_path() {
        assert_change_detection(ChangeDetection::path("src"), &["src", "src/lib.rs"]);
    }

    #[test]
    fn fixture_01() {
        assert_change_detection(
            ChangeDetection::path("fixtures-01"),
            &[
                "fixtures-01",
                "fixtures-01/a",
                "fixtures-01/ab",
                "fixtures-01/b",
                "fixtures-01/bc",
                "fixtures-01/c",
                "fixtures-01/cd",
            ],
        );
    }

    #[test]
    fn fixture_01_global_include() {
        assert_change_detection(
            ChangeDetection::include(|path: &Path| {
                path.file_name()
                    .map(|filename| filename.to_str().unwrap().ends_with("b"))
                    .unwrap_or(false)
            })
            .path("fixtures-01"),
            &["fixtures-01", "fixtures-01/ab", "fixtures-01/b"],
        );
    }

    #[test]
    fn fixture_01_global_exclude() {
        assert_change_detection(
            ChangeDetection::exclude(|path: &Path| {
                path.file_name()
                    .map(|filename| filename.to_str().unwrap().ends_with("b"))
                    .unwrap_or(false)
            })
            .path("fixtures-01"),
            &[
                "fixtures-01",
                "fixtures-01/a",
                "fixtures-01/bc",
                "fixtures-01/c",
                "fixtures-01/cd",
            ],
        );
    }

    #[test]
    fn fixture_01_global_filter() {
        assert_change_detection(
            ChangeDetection::filter(
                |path: &Path| {
                    path.file_name()
                        .map(|filename| filename.to_str().unwrap().ends_with("b"))
                        .unwrap_or(false)
                },
                |path: &Path| {
                    path.file_name()
                        .map(|filename| filename.to_str().unwrap().starts_with("a"))
                        .unwrap_or(false)
                },
            )
            .path("fixtures-01"),
            &["fixtures-01", "fixtures-01/b"],
        );
    }

    #[test]
    fn fixture_02() {
        assert_change_detection(
            ChangeDetection::path("fixtures-02"),
            &[
                "fixtures-02",
                "fixtures-02/abc",
                "fixtures-02/def",
                "fixtures-02/ghk",
            ],
        );
    }

    #[test]
    fn fixture_03() {
        assert_change_detection(
            ChangeDetection::path("fixtures-03"),
            &[
                "fixtures-03",
                "fixtures-03/hello",
                "fixtures-03/hello.c",
                "fixtures-03/hello.js",
            ],
        );
    }

    #[test]
    fn all_fixtures() {
        assert_change_detection(
            ChangeDetection::path("fixtures-01")
                .path("fixtures-02")
                .path("fixtures-03"),
            &[
                "fixtures-01",
                "fixtures-01/a",
                "fixtures-01/ab",
                "fixtures-01/b",
                "fixtures-01/bc",
                "fixtures-01/c",
                "fixtures-01/cd",
                "fixtures-02",
                "fixtures-02/abc",
                "fixtures-02/def",
                "fixtures-02/ghk",
                "fixtures-03",
                "fixtures-03/hello",
                "fixtures-03/hello.c",
                "fixtures-03/hello.js",
            ],
        );
    }

    #[test]
    #[cfg(feature = "glob")]
    fn path_matchers() {
        use path_matchers::glob;
        assert_change_detection(
            ChangeDetection::include(glob("**/a*").unwrap())
                .path("fixtures-01")
                .path("fixtures-02")
                .path("fixtures-03"),
            &[
                "fixtures-01",
                "fixtures-01/a",
                "fixtures-01/ab",
                "fixtures-02",
                "fixtures-02/abc",
                "fixtures-03",
            ],
        );
    }
}
