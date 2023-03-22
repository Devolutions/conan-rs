extern crate serde;
extern crate serde_json;

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

#[test]
fn test_conan_build_info() {
    let build_info = BuildInfo::from_str(include_str!("../../test/conanbuildinfo1.json")).unwrap();

    let openssl = build_info.get_dependency("openssl").unwrap();
    assert_eq!(openssl.get_binary_dir(), None);
    let openssl_dir = openssl.get_root_dir().unwrap();
    let openssl_lib_dir = openssl.get_library_dir().unwrap();
    let openssl_inc_dir = openssl.get_include_dir().unwrap();
    assert_eq!(
        openssl_dir,
        "/home/awake/.conan/data/openssl/1.1.1b-2/devolutions/stable/package/de9c231f84c85def9df09875e1785a1319fa8cb6"
    );
    assert_eq!(openssl_lib_dir, "/home/awake/.conan/data/openssl/1.1.1b-2/devolutions/stable/package/de9c231f84c85def9df09875e1785a1319fa8cb6/lib");
    assert_eq!(openssl_inc_dir, "/home/awake/.conan/data/openssl/1.1.1b-2/devolutions/stable/package/de9c231f84c85def9df09875e1785a1319fa8cb6/include");

    let dependencies = build_info.dependencies();
    assert_eq!(dependencies.len(), 1);

    let settings = build_info.settings;
    assert_eq!(settings.arch, Some("x86_64".to_string()));
    assert_eq!(settings.arch_build, Some("x86_64".to_string()));
    assert_eq!(settings.build_type, Some("Release".to_string()));
    assert_eq!(settings.compiler, Some("gcc".to_string()));
    assert_eq!(settings.compiler_libcxx, Some("libstdc++".to_string()));
    assert_eq!(settings.compiler_version, Some("4.8".to_string()));
    assert_eq!(settings.os, Some("Linux".to_string()));
    assert_eq!(settings.os_build, Some("Linux".to_string()));

    let build_info = BuildInfo::from_str(include_str!("../../test/conanbuildinfo2.json")).unwrap();

    let curl = build_info.get_dependency("curl").unwrap();
    assert_eq!(curl.version, "7.58.0");

    let mbedtls = build_info.get_dependency("mbedtls").unwrap();
    assert_eq!(mbedtls.libs, ["mbedtls", "mbedcrypto", "mbedx509"]);

    let dependencies = build_info.dependencies();
    assert_eq!(dependencies.len(), 2);

    let build_info = BuildInfo::from_str(include_str!("../../test/conanbuildinfo3.json")).unwrap();

    let dependencies = build_info.dependencies();
    assert_eq!(dependencies.len(), 2);

    let settings = build_info.settings;
    assert_eq!(settings.compiler, Some("Visual Studio".to_string()));

    let build_info = BuildInfo::from_str(include_str!("../../test/conanbuildinfo4.json")).unwrap();
    let dependencies = build_info.dependencies();
    assert_eq!(dependencies.len(), 2);

    let settings = build_info.settings;
    assert_eq!(settings.compiler, Some("clang".to_string()));
}

#[test]
fn test_conan_build_info_syslibs() {
    let build_info = BuildInfo::from_str(include_str!("../../test/conanbuildinfo5.json")).unwrap();
    let dependencies = build_info.dependencies();
    assert_eq!(dependencies.len(), 10);

    let libsystemd = build_info.get_dependency("libsystemd").unwrap();
    assert_eq!(libsystemd.libs, ["systemd"]);

    let system_libs = libsystemd.system_libs.as_ref().unwrap().as_slice();
    assert_eq!(system_libs, ["rt", "pthread", "dl"]);
}

#[test]
fn test_cargo_build_info() {
    let build_info = BuildInfo::from_str(include_str!("../../test/conanbuildinfo1.json")).unwrap();
    build_info.cargo_emit();
}

#[test]
fn test_cargo_build_info_syslibs() {
    let build_info = BuildInfo::from_str(include_str!("../../test/conanbuildinfo5.json")).unwrap();
    build_info.cargo_emit();
}
