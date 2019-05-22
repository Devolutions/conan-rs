# conan-rs

A Rust wrapper of the conan C/C++ package manager (conan.io) to simplify usage in build scripts.

```toml
# Cargo.toml
[build-dependencies]
conan = "0.1"
```

The conan executable is assumed to be `conan` unless the `CONAN` environment variable is set.

```rust
extern crate conan;
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
        return;
    }
}
```
