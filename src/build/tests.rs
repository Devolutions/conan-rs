use super::*;

#[test]
fn test_default_build_command() {
    let build_command = BuildCommand::default();
    let args = build_command.args().unwrap();

    let expected = vec!["build", "."];

    assert_eq!(args, expected);
}

#[test]
fn test_custom_build_command() {
    let build_command = BuildCommandBuilder::new()
        .with_recipe_path(PathBuf::from("./recipe"))
        .with_build_path(PathBuf::from("./build"))
        .with_install_path(PathBuf::from("./install"))
        .with_package_path(PathBuf::from("./package"))
        .with_source_path(PathBuf::from("./source"))
        .should_configure(true)
        .should_build(true)
        .should_install(true)
        .build();

    let args = build_command.args().unwrap();

    let expected = vec![
        "build",
        "./recipe",
        "--build-folder",
        "./build",
        "--install-folder",
        "./install",
        "--package-folder",
        "./package",
        "--source-folder",
        "./source",
        "--configure",
        "--build",
        "--install",
    ];

    assert_eq!(args, expected);
}
