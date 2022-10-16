#![allow(unused_doc_comments)]

pub mod build_info;

extern crate regex;
extern crate which;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate lazy_static;

// conan.cmake wrapper reference
// https://github.com/conan-io/cmake-conan/blob/develop/conan.cmake

use std::env;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;

use build_info::{build_settings::BuildSettings, BuildInfo};

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

    let output = Command::new(&conan_program).arg("--version").output();

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
            url: captures[2].to_string(),
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

#[derive(Clone, PartialEq)]
pub enum BuildPolicy {
    Never,
    Always,
    Missing,
    Outdated,
}

pub struct InstallCommand<'a> {
    profile: Option<&'a str>,
    remote: Option<&'a str>,
    build_settings: Option<BuildSettings>,
    build_policy: Option<BuildPolicy>,
    recipe_path: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    update_check: bool,
}

pub struct InstallCommandBuilder<'a> {
    profile: Option<&'a str>,
    remote: Option<&'a str>,
    build_settings: Option<BuildSettings>,
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
            build_settings: None,
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

    pub fn build_settings(mut self, build_settings: BuildSettings) -> Self {
        self.build_settings = Some(build_settings);
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

    pub fn build(self) -> InstallCommand<'a> {
        InstallCommand {
            profile: self.profile,
            remote: self.remote,
            build_settings: self.build_settings,
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
                BuildPolicy::Never => {
                    args.extend(&["-b", "never"]);
                }
                BuildPolicy::Always => {
                    args.extend(&["-b"]);
                }
                BuildPolicy::Missing => {
                    args.extend(&["-b", "missing"]);
                }
                BuildPolicy::Outdated => {
                    args.extend(&["-b", "outdated"]);
                }
            }
        }

        let output_dir = self.output_dir();
        if let Some(output_dir) = &output_dir {
            let current_dir = env::current_dir().unwrap().to_path_buf();
            if output_dir != &current_dir {
                args.extend(&["-if", output_dir.to_str().unwrap()]);
            }
        }

        let settings;
        if let Some(build_settings) = &self.build_settings {
            settings = build_settings.args();
            args.extend(settings.iter().map(String::as_str));
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
    use build_info::build_settings::{BuildType};

    let build_settings = BuildSettings::new().build_type(BuildType::Release);
    let command = InstallCommandBuilder::new()
        .with_profile("linux-x86_64")
        .build_settings(build_settings)
        .build_policy(BuildPolicy::Missing)
        .build();
    assert_eq!(
        command.args(),
        [
            "install",
            "-g",
            "json",
            "-pr",
            "linux-x86_64",
            "-b",
            "missing",
            "-s",
            "build_type=Release"
        ]
    );
}
