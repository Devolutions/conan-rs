#[cfg(test)]
pub mod test;

use super::ConanInstallError;
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;
use std::process::Command;

lazy_static! {
    static ref REGEX_CONAN_REMOTE: Regex = Regex::new(r"(\S+):\s+(\S+)\s+(.*)").unwrap();
}

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

// NOTE: This function will be used later
#[allow(dead_code)]
pub fn get_remote_list() -> Result<Vec<Remote>, ConanInstallError> {
    let output = Command::new("conan")
        .arg("remote")
        .arg("list")
        .output()
        .expect("failed to execute conan");

    let output_stdout = String::from_utf8(output.stdout).map_err(ConanInstallError::Utf8Error)?;

    Ok(output_stdout
        .lines()
        .map(|x| {
            let captures = REGEX_CONAN_REMOTE.captures(x.trim()).unwrap();
            Remote {
                name: captures[1].to_string(),
                url: captures[2].to_string(),
            }
        })
        .collect())
}
