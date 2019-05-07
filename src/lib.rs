
extern crate regex;
extern crate which;

#[macro_use]
extern crate lazy_static;

// conan.cmake wrapper reference
// https://github.com/conan-io/cmake-conan/blob/develop/conan.cmake

use std::fmt;
use std::path::PathBuf;
use std::process::Command;

use regex::Regex;

/**
 * conan detection
 */

#[derive(Clone)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub micro: u8,
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.micro)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.micro)
    }
}

lazy_static! {
    static ref REGEX_CONAN_VERSION: Regex = Regex::new(r"version (\d+)\.(\d+).(\d+)$").unwrap();
}

pub fn get_conan_path() -> Option<PathBuf> {
    which::which("conan").ok()
}

pub fn get_conan_version() -> Option<Version> {
    let output = Command::new("conan")
        .arg("--version")
        .output()
        .expect("failed to execute conan");

    // $ conan --version
    // Conan version 1.14.3

    let output_stdout = String::from_utf8(output.stdout).unwrap();
    let captures = REGEX_CONAN_VERSION.captures(output_stdout.as_str().trim()).unwrap();

    let version = Version {
        major: captures[1].parse::<u8>().unwrap(),
        minor: captures[2].parse::<u8>().unwrap(),
        micro: captures[3].parse::<u8>().unwrap(),
    };

    Some(version)
}

#[test]
fn test_conan_path() {
    if let Some(path) = get_conan_path() {
        println!("Conan path: {}", path.to_str().unwrap());
    }
}

#[test]
fn test_conan_version() {
    if let Some(version) = get_conan_version() {
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
