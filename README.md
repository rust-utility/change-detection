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
