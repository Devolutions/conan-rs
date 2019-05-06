
use std::path::PathBuf;
use std::process::Command;

use regex::Regex;

// https://github.com/conan-io/cmake-conan/blob/develop/conan.cmake

lazy_static! {
    static ref REGEX_CONAN_VERSION: Regex = Regex::new(r"version (\d+)\.(\d+).(\d+)$").unwrap();
}

pub fn get_conan_path() -> PathBuf {
    which::which("conan").unwrap()
}

pub fn get_conan_version() -> String {
	let output = Command::new("conan")
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

    format!("{}.{}.{}", version_major.to_string(), version_minor.to_string(), version_micro.to_string())
}

#[test]
fn test_conan_detect() {
    let conan_path = get_conan_path();
    println!("Conan path: {}", conan_path.to_str().unwrap());
}

#[test]
fn test_conan_version() {
	let version = get_conan_version();
	println!("Conan version: {}", version);
}
