# conan-rs

A Rust wrapper of the conan C/C++ package manager (conan.io) to simplify usage in build scripts.

The conan executable is assumed to be `conan` unless the `CONAN` environment variable is set.

Add conan to the Cargo.toml build-dependencies section:

```toml
# Cargo.toml
[build-dependencies]
conan = "0.3"
```

Modify the project build.rs script to invoke cargo and emit the conan build information automatically.

Using conan profiles with names derived from the cargo target information is recommended:

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
        .recipe_path(Path::new("conanfile.txt"))
        .build();

    if let Some(build_info) = command.generate() {
        println!("using conan build info");
        build_info.cargo_emit();
    }
}
```

The simplest approach is to add a conanfile.txt file alongside build.rs:

```
[requires]
openssl/1.1.1l@devolutions/stable
```

To test if the conan packages are properly imported, run `cargo -vv build`, and look for output similar to this:

```bash
[conan-test 0.1.0] using conan build info
[conan-test 0.1.0] cargo:rustc-link-search=native=/Users/mamoreau/.conan/data/openssl/1.1.1l/devolutions/stable/package/ce597277d61571523403b5b500bda70acd77cd8a/lib
[conan-test 0.1.0] cargo:rustc-link-lib=crypto
[conan-test 0.1.0] cargo:rustc-link-lib=ssl
[conan-test 0.1.0] cargo:include=/Users/mamoreau/.conan/data/openssl/1.1.1l/devolutions/stable/package/ce597277d61571523403b5b500bda70acd77cd8a/include
[conan-test 0.1.0] cargo:rerun-if-env-changed=CONAN
```

This sample conan recipe is available [here](https://github.com/Devolutions/conan-public), even if it is not available in a public conan repository.
