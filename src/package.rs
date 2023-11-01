#[cfg(test)]
mod tests;

use super::util::find_program;
use std::fs;
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

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid file name: {0}")]
    InvalidFileName(String),

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

    pub fn emit_cargo_libs_linkage(&self, libs_dir: PathBuf) -> Result<(), ConanPackageError> {
        let libs_dir_path = self.path.join(libs_dir);

        let entries = fs::read_dir(libs_dir_path.clone())?;

        for entry in entries {
            let lib_path = entry?.path();
            if lib_path.is_file() {
                let lib_name = lib_path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| ConanPackageError::InvalidFileName(lib_path.display().to_string()))?;

                let lib_name = if lib_name.starts_with("lib") {
                    &lib_name[3..]
                } else {
                    lib_name
                };

                if let Some(lib_suffix) = lib_path.extension().and_then(|s| s.to_str()) {
                    let lib_type = match lib_suffix {
                        "so" | "dll" | "dylib" => "dylib",
                        "a" | "lib" => "static",
                        _ => continue,
                    };
                    println!("cargo:rustc-link-lib={}={}", lib_type, lib_name);
                }
            }
        }

        println!("cargo:rustc-link-search=native={}", libs_dir_path.display());

        Ok(())
    }
}
