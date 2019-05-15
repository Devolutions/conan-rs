
extern crate regex;
extern crate which;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate lazy_static;

// conan.cmake wrapper reference
// https://github.com/conan-io/cmake-conan/blob/develop/conan.cmake

use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;

use serde::{Serialize, Deserialize};

/**
 * conan detection
 */

lazy_static! {
    static ref REGEX_CONAN_VERSION: Regex = Regex::new(r"version (\d+)\.(\d+).(\d+)$").unwrap();
}

pub fn find_program() -> Option<PathBuf> {
    which::which("conan").ok()
}

pub fn find_version() -> Option<String> {
    let conan_program = find_program();

    if conan_program.is_none() {
        return None;
    }

    let conan_program = conan_program.unwrap().as_path().to_str().unwrap().to_string();

    let output = Command::new(&conan_program)
        .arg("--version")
        .output()
        .expect("failed to execute conan");

    // $ conan --version
    // Conan version 1.14.3

    let output_stdout = String::from_utf8(output.stdout).unwrap();
    let captures = REGEX_CONAN_VERSION.captures(output_stdout.as_str().trim()).unwrap();

    let version_major = captures[1].parse::<u8>().unwrap();
    let version_minor = captures[2].parse::<u8>().unwrap();
    let version_micro = captures[3].parse::<u8>().unwrap();

    let version = format!("{}.{}.{}", version_major, version_minor, version_micro);

    Some(version)
}

#[test]
fn test_find_program() {
    if let Some(path) = find_program() {
        println!("Conan path: {}", path.to_str().unwrap());
    }
}

#[test]
fn test_find_version() {
    if let Some(version) = find_version() {
        println!("Conan version: {}", version);
    }
}

/**
 * conan profile
 */

pub fn get_profile_list() -> Vec<String> {
    let output = Command::new("conan")
        .arg("profile")
        .arg("list")
        .output()
        .expect("failed to execute conan");

    // $ conan profile list
    // default

    let output_stdout = String::from_utf8(output.stdout).unwrap();

    let mut list: Vec<String> = Vec::new();

    for line in output_stdout.lines() {
        list.push(line.to_string());
    }

    list
}

#[test]
fn test_conan_profile_list() {
    let conan_profile_list = get_profile_list();
    for conan_profile in conan_profile_list {
        println!("{}", conan_profile);
    }
}

/**
 * conan remote
 */

#[derive(Clone)]
pub struct Remote {
    pub name: String,
    pub url: String,
}

impl fmt::Debug for Remote {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.url)
    }
}

impl fmt::Display for Remote {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.url)
    }
}

lazy_static! {
    static ref REGEX_CONAN_REMOTE: Regex = Regex::new(r"(\S+):\s+(\S+)\s+(.*)").unwrap();
}

pub fn get_remote_list() -> Vec<Remote> {
    let output = Command::new("conan")
        .arg("remote")
        .arg("list")
        .output()
        .expect("failed to execute conan");

    // $ conan remote list
    // conan-center: https://conan.bintray.com [Verify SSL: True]
    // artifactory: https://devolutions.jfrog.io/devolutions/api/conan/conan-local [Verify SSL: True]

    let output_stdout = String::from_utf8(output.stdout).unwrap();

    let mut list: Vec<Remote> = Vec::new();

    for line in output_stdout.lines() {
        let captures = REGEX_CONAN_REMOTE.captures(line.trim()).unwrap();
        let remote = Remote {
            name: captures[1].to_string(),
            url: captures[2].to_string()
        };
        list.push(remote);
    }

    list
}

#[test]
fn test_conan_remote_list() {
    let conan_remote_list = get_remote_list();
    for conan_remote in conan_remote_list {
        println!("{}", conan_remote);
    }
}

// conan build info

#[derive(Serialize, Deserialize)]
pub struct BuildDependency {
    version: String,
    description: String,
    rootpath: String,
    sysroot: String,
    include_paths: Vec<String>,
    lib_paths: Vec<String>,
    bin_paths: Vec<String>,
    build_paths: Vec<String>,
    res_paths: Vec<String>,
    libs: Vec<String>,
    defines: Vec<String>,
    cflags: Vec<String>,
    cxxflags: Vec<String>,
    sharedlinkflags: Vec<String>,
    exelinkflags: Vec<String>,
    cppflags: Vec<String>,
    name: String,
}

impl BuildDependency {
    pub fn get_root_dir(&self) -> Option<&str> {
        Some(self.rootpath.as_str())
    }

    pub fn get_library_dir(&self) -> Option<&str> {
        self.lib_paths.get(0).map(|x| &**x)
    }

    pub fn get_include_dir(&self) -> Option<&str> {
        self.include_paths.get(0).map(|x| &**x)
    }

    pub fn get_binary_dir(&self) -> Option<&str> {
        self.bin_paths.get(0).map(|x| &**x)
    }
}

#[derive(Serialize, Deserialize)]
pub struct BuildSettings {
    arch: String,
    arch_build: String,
    build_type: String,
    compiler: String,
    #[serde(rename = "compiler.libcxx")]
    compiler_libcxx: String,
    #[serde(rename = "compiler.version")]
    compiler_version: String,
    os: String,
    os_build: String,
}

#[derive(Serialize, Deserialize)]
pub struct BuildInfo {
    dependencies: Vec<BuildDependency>,
    settings: BuildSettings,
}

impl BuildInfo {
    pub fn from_str(json: &str) -> Option<Self> {
        serde_json::from_str(&json).ok()
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

    pub fn cargo_emit(&self) {
        for dependency in &self.dependencies {
            for lib_path in &dependency.lib_paths {
                println!("cargo:rustc-link-search=native={}", lib_path);
            }

            for lib in &dependency.libs {
                println!("cargo:rustc-link-lib={}", lib);
            }

            for include_path in &dependency.include_paths {
                println!("cargo:include={}", include_path);
            }
        }
    }
}

#[test]
fn test_conan_build_info() {
    let build_info = BuildInfo::from_str(include_str!("../test/conanbuildinfo.json")).unwrap();

    let openssl = build_info.get_dependency("openssl").unwrap();
    assert_eq!(openssl.get_binary_dir(), None);
    let openssl_dir = openssl.get_root_dir().unwrap();
    let openssl_lib_dir = openssl.get_library_dir().unwrap();
    let openssl_inc_dir = openssl.get_include_dir().unwrap();
    assert_eq!(openssl_dir, "/home/awake/.conan/data/openssl/1.1.1b-2/devolutions/stable/package/de9c231f84c85def9df09875e1785a1319fa8cb6");
    assert_eq!(openssl_lib_dir, "/home/awake/.conan/data/openssl/1.1.1b-2/devolutions/stable/package/de9c231f84c85def9df09875e1785a1319fa8cb6/lib");
    assert_eq!(openssl_inc_dir, "/home/awake/.conan/data/openssl/1.1.1b-2/devolutions/stable/package/de9c231f84c85def9df09875e1785a1319fa8cb6/include");

    let settings = build_info.settings;
    assert_eq!(settings.arch, "x86_64");
    assert_eq!(settings.arch_build, "x86_64");
    assert_eq!(settings.build_type, "Release");
    assert_eq!(settings.compiler, "gcc");
    assert_eq!(settings.compiler_libcxx, "libstdc++");
    assert_eq!(settings.compiler_version, "4.8");
    assert_eq!(settings.os, "Linux");
    assert_eq!(settings.os_build, "Linux");
}

#[test]
fn test_cargo_build_info() {
    let build_info = BuildInfo::from_str(include_str!("../test/conanbuildinfo.json")).unwrap();
    build_info.cargo_emit();
}
