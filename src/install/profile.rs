use super::ConanInstallError;
use std::process::Command;

#[allow(dead_code)]
pub fn get_profile_list() -> Result<Vec<String>, ConanInstallError> {
    let output = Command::new("conan")
        .arg("profile")
        .arg("list")
        .output()
        .map_err(ConanInstallError::ConanInstallFailed)?;

    let output_stdout = String::from_utf8(output.stdout).map_err(ConanInstallError::Utf8Error)?;

    Ok(output_stdout.lines().map(|x| x.to_string()).collect())
}
