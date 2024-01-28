<p align="center">
    <img src=https://blog.conan.io/assets/conan_cargo.png width=138/>
</p>

<h1 align="center">conan-rs</h1>

<p align="center"><strong>A Rust wrapper of the conan C/C++ package manager (conan.io) to simplify usage in build scripts</strong></p>

<div align="center">
    <a href="https://crates.io/crates/conan" target="_blank">
    <img src="https://img.shields.io/crates/v/conan"></a>
    <a href="https://docs.rs/conan" target="_blank">
    <img src="https://img.shields.io/docsrs/conan"></a>
    <a href="https://github.com/Devolutions/conan-rs" target="_blank">
    <img alt="GitHub Repo stars" src="https://img.shields.io/github/stars/Devolutions/conan-rs?style=social">
</div>

## TLDR

Add conan to the build-dependencies section:

```bash
cargo add conan --build
```

Modify the project `build.rs` script to invoke cargo and emit the conan build
information automatically. Using conan profiles with names derived from the
cargo target information is recommended:

NOTE: The conan executable is assumed to be `conan` unless the `CONAN`
environment variable is set.

```rust
use std::path::Path;
use std::env;

use conan::*;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let conan_profile = format!("{}-{}", target_os, target_arch);

    let command = InstallCommandBuilder::new()
        .with_profile(&conan_profile)
        .build_policy(BuildPolicy::Missing)
        .with_option("sign", "True")
        .recipe_path(Path::new("conanfile.txt"))
        .build();

    if let Some(build_info) = command.generate() {
        println!("using conan build info");
        build_info.cargo_emit();
    }

    let build_command = BuildCommandBuilder::new()
        .with_recipe_path(PathBuf::from("../../../conanfile.py"))
        .with_build_path(PathBuf::from("../../../build/"))
        .build();

    if let Some(exit_status) = build_command.run() {
        println!("conan build exited with {}", exit_status);
    }
}
```

The simplest approach is to add a conanfile.txt file alongside build.rs:

```
[requires]
openssl/1.1.1l@devolutions/stable
```

To test if the conan packages are properly imported, run `cargo -vv build`, and
look for output similar to this:

```bash
[conan-test 0.1.0] using conan build info
[conan-test 0.1.0] cargo:rustc-link-search=native=/Users/mamoreau/.conan/data/openssl/1.1.1l/devolutions/stable/package/ce597277d61571523403b5b500bda70acd77cd8a/lib
[conan-test 0.1.0] cargo:rustc-link-lib=crypto
[conan-test 0.1.0] cargo:rustc-link-lib=ssl
[conan-test 0.1.0] cargo:include=/Users/mamoreau/.conan/data/openssl/1.1.1l/devolutions/stable/package/ce597277d61571523403b5b500bda70acd77cd8a/include
[conan-test 0.1.0] cargo:rerun-if-env-changed=CONAN
```

This sample conan recipe is available
[here](https://github.com/Devolutions/conan-public), even if it is not available
in a public conan repository.

## Documentation

### Conan Install

The `InstallCommand` struct represents the "conan install" command, facilitating
package installation and dependency management in Rust projects.
`InstallCommandBuilder` provides a fluent API for constructing an
`InstallCommand`.

### Example

```rust
use conan::{InstallCommandBuilder, BuildPolicy};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let install_command = InstallCommandBuilder::new()
        .with_profile("default")
        .build_policy(BuildPolicy::Missing)
        .recipe_path(Path::new("conanfile.txt"))
        .output_dir(Path::new("output_directory"))
        .build();

    if install_command.generate().is_some() {
        println!("Packages installed successfully!");
    } else {
        println!("Failed to install packages.");
    }

    Ok(())
}
```

In this example, `InstallCommandBuilder` configures the Conan install command
with a profile, build policy, recipe file path, and output directory.
`generate()` executes the command, returning `Some(BuildInfo)` on success or
`None` on failure.

### Conan Build

The `BuildCommand` struct represents the "conan build" command, facilitating the
build process of Conan packages in Rust projects. `BuildCommandBuilder` provides
a fluent API to construct a `BuildCommand`.

### Example

```rust
use conan::BuildCommandBuilder;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build_command = BuildCommandBuilder::new()
        .with_recipe_path(PathBuf::from("conanfile.py"))
        .with_build_path(PathBuf::from("build"))
        .should_configure(true)
        .should_build(true)
        .should_install(true)
        .build();

    match build_command.run() {
        Some(status) if status.success() => println!("Build succeeded!"),
        _ => println!("Build failed."),
    }

    Ok(())
}
```

In this example, _BuildCommandBuilder_ is used to configure the Conan build
command with paths and options. _run()_ executes the command, returning
_Some(ExitStatus)_ on success or _None_ on failure.

### Conan Package

The `PackageCommand` struct represents the "conan package" command and is used
for creating packages. The `PackageCommandBuilder` provides a fluent API for
constructing a `PackageCommand`.

The `ConanPackage` struct provides functionality for managing Conan packages
that generate C++ libraries, and it aids in linking these libraries with Rust.

#### Example Usage:

```rust
use conan::{PackageCommandBuilder, PackageComman, ConanPackage};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let package_command = PackageCommandBuilder::new()
        .with_recipe_path(PathBuf::from("conanfile.py"))
        .build();

    if let Some(status) = package_command.run() {
        if !status.success() {
            println!("Package command failed.");
            return Ok(());
        }
    }

    let conan_package = ConanPackage::new(PathBuf::from("./package/"));
    conan_package.emit_cargo_libs_linkage(PathBuf::from("lib"))?;

    Ok(())
}
```

## Use Case: Integrating Rust into a legacy c++/conan1 codebase

Integrating Rust into a legacy C++ codebase can be a strategic move to leverage
Rust's memory safety features while maintaining existing C++ functionality. In
this guide, we will explore how to integrate Rust into a legacy C++/Conan
codebase using `conan-rs` and `autocxx`.

### Existing C++ Conan Codebase Structure

Your existing C++ codebase with Conan and CMake might look like this:

```
.
├── build
│   ├── bin
│   │   └── target_bin
│   ├── lib
│   │   ├── lib1.a
│   │   ├── lib2.a
│   │   ├── lib3.so
│   │   ├── lib4.so
│   │   ├── ...
│   │   └── libn.a
├── CMakeLists.txt
├── conanfile.py
├── include
│   └── ...
├── profiles
│   ├── ...
├── src
│   ├── target_bin
│   │   ├── ...
│   ├── lib1
│   │   ├── CMakeLists.txt
│   │   ├── include
│   │   │   └── ...
│   │   ├── src
│   │   │   └── ...
│   ├── ...
│   ├── libn
│   │   ├── CMakeLists.txt
│   │   ├── include
│   │   │   └── ...
│   │   ├── src
│   │   │   └── ...
```

Make sure that after a build the build dir look like this(Your configuration may
vary):

```
├── build
│   ├── bin
│   │   └── target_bin
│   ├── lib
│   │   ├── lib1.a
│   │   ├── lib2.a
│   │   ├── lib3.so
│   │   ├── lib4.so
│   │   ├── ...
│   │   └── libn.a
```

Also, the `package()` method in your conanfile should organize your libs and
associated includes in a config akin to:

```
package
├── conaninfo.txt
├── conanmanifest.txt
├── include
└── lib
├── lib1.a
├── ...
└── libn.so
```

### Creating the Rust "Bridge" Crate

Create a Rust library crate within the codebase to act as the "bridge" between
the C++ and Rust code:

```
.
├── build.rs
├── Cargo.lock
├── Cargo.toml
└── src
├── lib.rs
└── main.rs
```

### Setting Up Dependencies

Install `conan-rs` and `autocxx`:

```bash
cargo add conan-rs autocxx --build
cargo add autocxx
```

### Setting the build script:

In your crate's build script (`build.rs`), configure the integration:

```rust
use conan::{
    BuildCommandBuilder, BuildPolicy, ConanPackage, InstallCommandBuilder, PackageCommandBuilder,
};
use std::env;
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../path/to/your/conanfile.py");
    println!("cargo:rerun-if-changed=../../path/to/your/build/directory");

    let out_dir = env::var("OUT_DIR").map(PathBuf::from).unwrap_or_else(|_| {
        eprintln!("Error: OUT_DIR environment variable is not set");
        process::exit(1);
    });

    println!("OUT_DIR: {:?}", out_dir);

    let conan_profile = env::var("CONAN_PROFILE").unwrap_or_else(|_| "default".to_string());
    let install_command = InstallCommandBuilder::new()
        .with_profile(&conan_profile)
        .with_remote("your_remote")
        .build_policy(BuildPolicy::Missing)
        .with_profile("../../path/to/your/conan/profile")
        .recipe_path(Path::new("../../path/to/your/conanfile.py"))
        .output_dir(Path::new("../../path/to/your/build/directory"))
        .with_options(&["option1=True", "option2=True"])
        .update_check()
        .build();

    if let Some(build_info) = install_command.generate() {
        println!("using conan build info");
        build_info.cargo_emit();
    } else {
        eprintln!("Error: failed to run conan install");
        process::exit(1);
    }

    BuildCommandBuilder::new()
        .with_recipe_path(PathBuf::from("../../path/to/your/conanfile.py"))
        .with_build_path(PathBuf::from("../../path/to/your/build/directory"))
        .build()
        .run()
        .unwrap_or_else(|| {
            eprintln!("Error: Unable to run conan build");
            process::exit(1);
        });

    let package_command = PackageCommandBuilder::new()
        .with_recipe_path(PathBuf::from("../../path/to/your/conanfile.py"))
        .with_build_path(PathBuf::from("../../path/to/your/build/directory"))
        .with_package_path(out_dir.clone())
        .build();

    if let Some(exit_status) = package_command.run() {
        println!("conan package exited with {}", exit_status);
    }

    let conan_package = ConanPackage::new(out_dir.clone());
    if let Err(err) = conan_package.emit_cargo_libs_linkage("lib".into()) {
        eprintln!("Error: Unable to emit cargo linkage: {:?}", err);
        process::exit(1);
    }

    let include_path = out_dir.join("include");
    let mut builder = autocxx_build::Builder::new("src/lib.rs", &[include_path])
        .build()
        .unwrap_or_else(|err| {
            eprintln!("Error: Unable to generate bindings: {:?}", err);
            process::exit(1);
        });

    builder.flag_if_supported("-std=c++14").compile("foo_bar");
    println!("cargo:rerun-if-changed=src/main.rs");
}
```

### Using C++ Libraries in Rust

Finally, use the C++ libraries in `lib.rs`:

```rust
use autocxx::prelude::*;

include_cpp! {
    #include "path/to/header.h"
    safety!(unsafe_ffi)
    generate!("FunctionFromCpp")
}

pub fn use_cpp_function() {
    let result = ffi::FunctionFromCpp();
    // Use result as needed
}
```
