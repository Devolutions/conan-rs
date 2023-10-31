#[cfg(test)]
mod test;

pub mod build_dependency;
pub mod build_settings;

use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};

use build_dependency::BuildDependency;
use build_settings::BuildSettings;

// conan build info
#[derive(Serialize, Deserialize)]
pub struct BuildInfo {
    pub(crate) dependencies: Vec<BuildDependency>,
    pub(crate) settings: BuildSettings,
}

impl BuildInfo {
    pub fn from_str(json: &str) -> Option<Self> {
        let result = serde_json::from_str(&json);
        if let Err(error) = result {
            eprintln!("failed to parse conan build info: {:?}", error);
            return None;
        }
        result.ok()
    }

    pub fn from_file(path: &Path) -> Option<Self> {
        if let Ok(json_file) = File::open(path) {
            serde_json::from_reader(&json_file).ok()
        } else {
            None
        }
    }

    pub fn get_dependency(&self, name: &str) -> Option<&BuildDependency> {
        self.dependencies.iter().find(|&x| x.name == name)
    }

    pub fn dependencies(&self) -> &Vec<BuildDependency> {
        &self.dependencies
    }

    pub fn cargo_emit(&self) {
        for dependency in &self.dependencies {
            for lib_path in &dependency.lib_paths {
                println!("cargo:rustc-link-search=native={}", lib_path);
            }

            for lib in &dependency.libs {
                println!("cargo:rustc-link-lib={}", lib);
            }

            if let Some(syslibs) = &dependency.system_libs {
                for syslib in syslibs {
                    println!("cargo:rustc-link-lib={}", syslib);
                }
            }

            for include_path in &dependency.include_paths {
                println!("cargo:include={}", include_path);
            }

            println!("cargo:rerun-if-env-changed=CONAN");
        }
    }
}
