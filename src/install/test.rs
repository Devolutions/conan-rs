use super::{
    build_info::{build_settings::BuildType, BuildSettings},
    BuildPolicy, InstallCommandBuilder,
};

#[test]
fn test_install_builder() -> Result<(), Box<dyn std::error::Error>> {
    let build_settings = BuildSettings::new().build_type(BuildType::Release);
    let command = InstallCommandBuilder::new()
        .with_profile("linux-x86_64")
        .build_settings(build_settings)
        .build_policy(BuildPolicy::Missing)
        .build();
    assert_eq!(
        command.args()?,
        [
            "install",
            "-g",
            "json",
            "--profile:host",
            "linux-x86_64",
            "-b",
            "missing",
            "-s",
            "build_type=Release"
        ]
    );

    Ok(())
}

#[test]
fn test_install_builder_cross() -> Result<(), Box<dyn std::error::Error>> {
    let build_settings = BuildSettings::new().build_type(BuildType::Debug);
    let command = InstallCommandBuilder::new()
        .with_host_profile("windows-x86_64")
        .with_build_profile("linux-x86_64")
        .build_settings(build_settings)
        .build_policy(BuildPolicy::Always)
        .build();
    assert_eq!(
        command.args()?,
        [
            "install",
            "-g",
            "json",
            "--profile:host",
            "windows-x86_64",
            "--profile:build",
            "linux-x86_64",
            "-b",
            "-s",
            "build_type=Debug"
        ]
    );

    Ok(())
}

#[test]
fn test_install_builder_with_options() -> Result<(), Box<dyn std::error::Error>> {
    let build_settings = BuildSettings::new().build_type(BuildType::Release);
    let command = InstallCommandBuilder::new()
        .with_profile("linux-x86_64")
        .build_settings(build_settings)
        .with_options(&["shared=True", "build_type=Release"])
        .build_policy(BuildPolicy::Missing)
        .build();
    assert_eq!(
        command.args()?,
        [
            "install",
            "-g",
            "json",
            "--profile:host",
            "linux-x86_64",
            "-b",
            "missing",
            "-o",
            "shared=True",
            "-o",
            "build_type=Release",
            "-s",
            "build_type=Release"
        ]
    );

    Ok(())
}
