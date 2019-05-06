
use std::process::Command;

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
