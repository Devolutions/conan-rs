
use std::fmt;
use std::process::Command;

use regex::Regex;

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
