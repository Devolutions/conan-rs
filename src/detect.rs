
use std::fmt;
use std::path::PathBuf;
use std::process::Command;

use regex::Regex;

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
fn test_conan_detect() {
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
