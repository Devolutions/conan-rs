use super::PackageCommandBuilder;
use std::path::PathBuf;

#[test]
fn test_default_args_generation() {
    let command = PackageCommandBuilder::new()
        .build();
    let args = command.args().expect("Failed to generate args");
    assert_eq!(args, vec!["package", "."]);
}

#[test]
fn test_successful_args_generation() {
    let recipe_path = PathBuf::from("path/to/recipe");
    let build_path = PathBuf::from("path/to/build");
    let install_path = PathBuf::from("path/to/install");
    let package_path = PathBuf::from("path/to/package");
    let source_path = PathBuf::from("path/to/source");

    let command = PackageCommandBuilder::new()
        .with_recipe_path(recipe_path.clone())
        .with_build_path(build_path.clone())
        .with_install_path(install_path.clone())
        .with_package_path(package_path.clone())
        .with_source_path(source_path.clone())
        .build();

    let args = command.args().expect("Failed to generate args");

    assert_eq!(
        args,
        vec![
            "package",
            "path/to/recipe",
            "--build-folder",
            "path/to/build",
            "--install-folder",
            "path/to/install",
            "--package-folder",
            "path/to/package",
            "--source-folder",
            "path/to/source",
        ]
    );
}
