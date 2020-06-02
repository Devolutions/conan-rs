#![allow(unused_doc_comments)]

extern crate regex;
extern crate which;
extern crate indexmap;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate lazy_static;

// conan.cmake wrapper reference
// https://github.com/conan-io/cmake-conan/blob/develop/conan.cmake

use std::fmt;
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command};

use regex::Regex;
use indexmap::IndexMap;
use serde::{Serialize, Deserialize};

/**
 * conan detection
 */

lazy_static! {
    static ref REGEX_CONAN_VERSION: Regex = Regex::new(r"version (\d+)\.(\d+).(\d+)$").unwrap();
}

pub fn find_program() -> Option<PathBuf> {
    if let Ok(conan) = env::var("CONAN") {
        return Some(PathBuf::from(conan));
    }
    which::which("conan").ok()
}

pub fn find_version() -> Option<String> {
    let conan_program = find_program()?;
    let conan_program = conan_program.as_path().to_str().unwrap().to_string();

    let output = Command::new(&conan_program)
        .arg("--version")
        .output();

    // $ conan --version
    // Conan version 1.14.3

    if let Ok(output) = output {
        let output_stdout = String::from_utf8(output.stdout).unwrap();
        let captures = REGEX_CONAN_VERSION.captures(output_stdout.as_str().trim()).unwrap();

        let version_major = captures[1].parse::<u8>().unwrap();
        let version_minor = captures[2].parse::<u8>().unwrap();
        let version_micro = captures[3].parse::<u8>().unwrap();

        let version = format!("{}.{}.{}", version_major, version_minor, version_micro);

        return Some(version);
    }

    None
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
    description: Option<String>,
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
    cxxflags: Option<Vec<String>>,
    sharedlinkflags: Vec<String>,
    exelinkflags: Vec<String>,
    cppflags: Option<Vec<String>>,
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
    arch: Option<String>,
    arch_build: Option<String>,
    build_type: Option<String>,
    compiler: Option<String>,
    #[serde(rename = "compiler.libcxx")]
    compiler_libcxx: Option<String>,
    #[serde(rename = "compiler.version")]
    compiler_version: Option<String>,
    os: Option<String>,
    os_build: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct BuildInfo {
    dependencies: Vec<BuildDependency>,
    settings: BuildSettings,
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

            for include_path in &dependency.include_paths {
                println!("cargo:include={}", include_path);
            }

            println!("cargo:rerun-if-env-changed=CONAN");
        }
    }
}


#[test]
fn test_conan_build_info() {
    let build_info = BuildInfo::from_str(include_str!("../test/conanbuildinfo1.json")).unwrap();

    let openssl = build_info.get_dependency("openssl").unwrap();
    assert_eq!(openssl.get_binary_dir(), None);
    let openssl_dir = openssl.get_root_dir().unwrap();
    let openssl_lib_dir = openssl.get_library_dir().unwrap();
    let openssl_inc_dir = openssl.get_include_dir().unwrap();
    assert_eq!(openssl_dir, "/home/awake/.conan/data/openssl/1.1.1b-2/devolutions/stable/package/de9c231f84c85def9df09875e1785a1319fa8cb6");
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

    let build_info = BuildInfo::from_str(include_str!("../test/conanbuildinfo2.json")).unwrap();

    let curl = build_info.get_dependency("curl").unwrap();
    assert_eq!(curl.version, "7.58.0");

    let mbedtls = build_info.get_dependency("mbedtls").unwrap();
    assert_eq!(mbedtls.libs, ["mbedtls", "mbedcrypto", "mbedx509"]);

    let dependencies = build_info.dependencies();
    assert_eq!(dependencies.len(), 2);

    let build_info = BuildInfo::from_str(include_str!("../test/conanbuildinfo3.json")).unwrap();

    let dependencies = build_info.dependencies();
    assert_eq!(dependencies.len(), 2);

    let settings = build_info.settings;
    assert_eq!(settings.compiler, Some("Visual Studio".to_string()));

    let build_info = BuildInfo::from_str(include_str!("../test/conanbuildinfo4.json")).unwrap();
    let dependencies = build_info.dependencies();
    assert_eq!(dependencies.len(), 2);

    let settings = build_info.settings;
    assert_eq!(settings.compiler, Some("clang".to_string()));
}

#[test]
fn test_cargo_build_info() {
    let build_info = BuildInfo::from_str(include_str!("../test/conanbuildinfo1.json")).unwrap();
    build_info.cargo_emit();
}

#[derive(Clone,PartialEq)]
pub enum BuildType {
    None,
    Debug,
    Release,
    RelWithDebInfo,
    MinSizeRel,
}

impl BuildType {
    pub fn as_str(&self) -> &str {
        match self {
            &BuildType::None => "None",
            &BuildType::Debug => "Debug",
            &BuildType::Release => "Release",
            &BuildType::RelWithDebInfo => "RelWithDebInfo",
            &BuildType::MinSizeRel => "MinSizeRel",
        }
    }
}

#[derive(Clone,PartialEq)]
pub enum BuildPolicy {
    Never,
    Always,
    Missing,
    Outdated,
}

pub struct InstallCommand<'a> {
    profile: Option<&'a str>,
    remote: Option<&'a str>,
    build_type: Option<BuildType>,
    build_policy: Option<BuildPolicy>,
    recipe_path: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    update_check: bool,
}

pub struct InstallCommandBuilder<'a> {
    profile: Option<&'a str>,
    remote: Option<&'a str>,
    build_type: Option<BuildType>,
    build_policy: Option<BuildPolicy>,
    recipe_path: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    update_check: bool,
}

impl<'a> InstallCommandBuilder<'a> {
    pub fn new() -> InstallCommandBuilder<'a> {
        InstallCommandBuilder {
            profile: None,
            remote: None,
            build_type: None,
            build_policy: None,
            recipe_path: None,
            output_dir: None,
            update_check: false,
        }
    }

    pub fn with_profile(mut self, profile: &'a str) -> Self {
        self.profile = Some(profile);
        self
    }

    pub fn with_remote(mut self, remote: &'a str) -> Self {
        self.remote = Some(remote);
        self
    }

    pub fn build_type(mut self, build_type: BuildType) -> Self {
        self.build_type = Some(build_type);
        self
    }

    pub fn build_policy(mut self, build_policy: BuildPolicy) -> Self {
        self.build_policy = Some(build_policy);
        self
    }

    pub fn recipe_path(mut self, recipe_path: &Path) -> Self {
        self.recipe_path = Some(recipe_path.to_path_buf());
        self
    }

    pub fn output_dir(mut self, output_dir: &Path) -> Self {
        self.output_dir = Some(output_dir.to_path_buf());
        self
    }

    pub fn update_check(mut self) -> Self {
        self.update_check = true;
        self
    }

    fn detect_build_type(&self) -> Option<BuildType> {
        if self.build_type.is_some() {
            return self.build_type.clone();
        } else if let Ok(profile) = env::var("PROFILE") {
            return match profile.as_str() {
                "debug" => Some(BuildType::Debug),
                "release" => Some(BuildType::Release),
                _ => None,
            }
        }
        None
    }

    pub fn build(self) -> InstallCommand<'a> {
        InstallCommand {
            profile: self.profile,
            remote: self.remote,
            build_type: self.detect_build_type(),
            build_policy: self.build_policy,
            recipe_path: self.recipe_path,
            output_dir: self.output_dir,
            update_check: self.update_check,
        }
    }
}

impl<'a> InstallCommand<'a> {
    pub fn args(&self) -> Vec<String> {
        let mut args: Vec<&str> = Vec::new();
        let mut settings: IndexMap<&str, &str> = IndexMap::new();
        let mut settings_kv: Vec<String> = Vec::new();
        let output_dir = self.output_dir();

        args.push("install");
        args.extend(&["-g", "json"]);

        if let Some(profile) = &self.profile {
            args.extend(&["-pr", profile]);
        }

        if let Some(remote) = &self.remote {
            args.extend(&["-r", remote]);
        }

        if self.update_check {
            args.push("-u");
        }

        if let Some(build_policy) = &self.build_policy {
            match build_policy {
                BuildPolicy::Never => { args.extend(&["-b", "never"]); },
                BuildPolicy::Always => { args.extend(&["-b"]); },
                BuildPolicy::Missing => { args.extend(&["-b", "missing"]); },
                BuildPolicy::Outdated => { args.extend(&["-b", "outdated"]); },
            }
        }

        if let Some(build_type) = &self.build_type {
            settings.insert("build_type", build_type.as_str());
        }

        if let Some(output_dir) = &output_dir {
            let current_dir = env::current_dir().unwrap().to_path_buf();
            if output_dir != &current_dir {
                args.extend(&["-if", output_dir.to_str().unwrap()]);
            }
        }

        for (key, val) in settings.iter() {
            settings_kv.push(format!("{}={}", key, val));
        }

        for kv in settings_kv.iter() {
            args.extend(&["-s", kv])
        }

        if let Some(recipe_path) = &self.recipe_path {
            args.push(recipe_path.to_str().unwrap());
        }

        args.iter().map(|x| x.to_string()).collect()
    }

    pub fn output_dir(&self) -> Option<PathBuf> {
        if let Some(output_dir) = &self.output_dir {
            return Some(output_dir.to_path_buf());
        } else if let Ok(output_dir) = env::var("OUT_DIR") {
            return Some(PathBuf::from(output_dir));
        } else if let Ok(output_dir) = env::current_dir() {
            return Some(output_dir.to_path_buf());
        }
        None
    }

    pub fn output_file(&self) -> Option<PathBuf> {
        let mut output_file = self.output_dir()?;
        output_file.push("conanbuildinfo.json");
        Some(output_file)
    }

    pub fn generate(&self) -> Option<BuildInfo> {
        let args = self.args();
        let program = find_program()?;
        let output_file = self.output_file()?;
        let mut command = Command::new(program);
        if let Ok(_) = command.args(args).status() {
            BuildInfo::from_file(output_file.as_path())
        } else {
            None
        }
    }
}

#[test]
fn test_install_builder() {
    let command = InstallCommandBuilder::new()
        .with_profile("linux-x86_64")
        .build_type(BuildType::Release)
        .build_policy(BuildPolicy::Missing)
        .build();
    assert_eq!(command.args(), ["install", "-g", "json", "-pr", "linux-x86_64", "-b", "missing", "-s", "build_type=Release"]);
}
