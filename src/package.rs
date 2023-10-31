#[cfg(test)]
mod tests;

use super::util::find_program;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConanPackageError {
    #[error("The recipe path is missing")]
    MissingRecipePath,

    #[error("Invalid Unicode in path")]
    InvalidUnicodeInPath,

    #[error("Conan binary not found")]
    ConanNotFound,

    #[error("Command execution failed")]
    CommandExecutionFailed,

    #[error("Other error: {0}")]
    Other(String),
}

pub struct ConanPackage {
    path: PathBuf,
}

#[derive(Default)]
pub struct PackageCommand {
    build_path: Option<PathBuf>,
    install_path: Option<PathBuf>,
    package_path: Option<PathBuf>,
    source_path: Option<PathBuf>,
    recipe_path: Option<PathBuf>,
}

impl Default for PackageCommandBuilder {
    fn default() -> Self {
        PackageCommandBuilder {
            build_path: None,
            install_path: None,
            package_path: None,
            source_path: None,
            recipe_path: Some(PathBuf::from(".")),
        }
    }
}

pub struct PackageCommandBuilder {
    build_path: Option<PathBuf>,
    install_path: Option<PathBuf>,
    package_path: Option<PathBuf>,
    source_path: Option<PathBuf>,
    recipe_path: Option<PathBuf>,
}

impl PackageCommandBuilder {
    pub fn new() -> Self {
        PackageCommandBuilder::default()
    }

    pub fn with_build_path(mut self, build_path: PathBuf) -> Self {
        self.build_path = Some(build_path);
        self
    }

    pub fn with_install_path(mut self, install_path: PathBuf) -> Self {
        self.install_path = Some(install_path);
        self
    }

    pub fn with_package_path(mut self, package_path: PathBuf) -> Self {
        self.package_path = Some(package_path);
        self
    }

    pub fn with_source_path(mut self, source_path: PathBuf) -> Self {
        self.source_path = Some(source_path);
        self
    }

    pub fn with_recipe_path(mut self, recipe_path: PathBuf) -> Self {
        self.recipe_path = Some(recipe_path);
        self
    }

    pub fn build(self) -> PackageCommand {
        PackageCommand {
            build_path: self.build_path,
            install_path: self.install_path,
            package_path: self.package_path,
            source_path: self.source_path,
            recipe_path: self.recipe_path,
        }
    }
}

impl PackageCommand {
    pub fn args(&self) -> Result<Vec<String>, ConanPackageError> {
        let mut args: Vec<&str> = Vec::new();

        args.extend(&["package", self.recipe_path.as_ref().unwrap().to_str().unwrap()]);

        if let Some(build_path) = &self.build_path {
            args.extend(&[
                "--build-folder",
                build_path.to_str().ok_or(ConanPackageError::InvalidUnicodeInPath)?,
            ]);
        }

        if let Some(install_path) = &self.install_path {
            args.extend(&[
                "--install-folder",
                install_path.to_str().ok_or(ConanPackageError::InvalidUnicodeInPath)?,
            ]);
        }

        if let Some(package_path) = &self.package_path {
            args.extend(&[
                "--package-folder",
                package_path.to_str().ok_or(ConanPackageError::InvalidUnicodeInPath)?,
            ]);
        }

        if let Some(source_path) = &self.source_path {
            args.extend(&[
                "--source-folder",
                source_path.to_str().ok_or(ConanPackageError::InvalidUnicodeInPath)?,
            ]);
        }

        Ok(args.iter().map(|s| s.to_string()).collect())
    }

    pub fn run(&self) -> Option<ExitStatus> {
        let args = self.args().ok()?;
        let conan_bin = find_program()?;
        let mut command = Command::new(conan_bin);
        Some(command.args(args).status().ok()?)
    }
}

impl Default for ConanPackage {
    fn default() -> Self {
        ConanPackage {
            path: PathBuf::from("./package/"),
        }
    }
}

impl ConanPackage {
    pub fn new(path: PathBuf) -> Self {
        ConanPackage { path }
    }

    pub fn emit_cargo_libs_linkage(&self) -> std::io::Result<()> {
        fn emit_link_info(path: &Path) -> std::io::Result<()> {
            if path.is_dir() {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    emit_link_info(&entry.path())?;
                }
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                match ext {
                    "a" | "so" | "dll" | "dylib" => {
                        if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                            // Here we strip the "lib" prefix from library names, which is a common convention.
                            // For example, a file named "libexample.a" would result in "cargo:rustc-link-lib=static:example".
                            let lib_name = if file_stem.starts_with("lib") {
                                &file_stem[3..]
                            } else {
                                file_stem
                            };

                            let kind = if ext == "a" { "static" } else { "dylib" };

                            println!("cargo:rustc-link-lib={}={}", kind, lib_name);
                            println!(
                                "cargo:rustc-link-search=native={}",
                                path.parent().unwrap_or(path).display()
                            );
                        }
                    }
                    _ => {}
                }
            }
            Ok(())
        }

        emit_link_info(&self.path)
    }
}
