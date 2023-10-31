#[cfg(test)]
mod test;

use lazy_static::lazy_static;
use regex::Regex;
use std::env;
use std::path::PathBuf;
use std::process::Command;

lazy_static! {
    static ref REGEX_CONAN_VERSION: Regex = Regex::new(r"version (\d+)\.(\d+).(\d+)$").unwrap();
}

pub fn find_program() -> Option<PathBuf> {
    if let Ok(conan) = env::var("CONAN") {
        return Some(PathBuf::from(conan));
    }
    which::which("conan").ok()
}

// NOTE: Will be used in the future
#[allow(dead_code)]
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
