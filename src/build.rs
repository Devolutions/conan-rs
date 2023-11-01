#[cfg(test)]
mod tests;

use std::{
    path::PathBuf,
    process::{Command, ExitStatus},
};
use thiserror::Error;

use crate::util::find_program;

#[derive(Debug, Error)]
pub enum ConanBuildError {}

/// A command for building a Conan package.
pub struct BuildCommand {
    recipe_path: Option<PathBuf>,
    build_path: Option<PathBuf>,
    install_path: Option<PathBuf>,
    package_path: Option<PathBuf>,
    source_path: Option<PathBuf>,
    should_configure: bool,
    should_build: bool,
    should_install: bool,
}

/// Builder pattern for creating a `BuildCommand`
#[derive(Default)]
pub struct BuildCommandBuilder {
    recipe_path: Option<PathBuf>,
    build_path: Option<PathBuf>,
    install_path: Option<PathBuf>,
    package_path: Option<PathBuf>,
    source_path: Option<PathBuf>,
    should_configure: bool,
    should_build: bool,
    should_install: bool,
}

impl Default for BuildCommand {
    fn default() -> Self {
        BuildCommand {
            recipe_path: Some(PathBuf::from(".")),
            build_path: None,
            install_path: None,
            package_path: None,
            source_path: None,
            should_configure: false,
            should_build: false,
            should_install: false,
        }
    }
}

impl BuildCommandBuilder {
    pub fn new() -> BuildCommandBuilder {
        BuildCommandBuilder::default()
    }

    pub fn with_recipe_path(mut self, recipe_path: PathBuf) -> Self {
        self.recipe_path = Some(recipe_path);
        self
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

    pub fn should_configure(mut self, should_configure: bool) -> Self {
        self.should_configure = should_configure;
        self
    }

    pub fn should_build(mut self, should_build: bool) -> Self {
        self.should_build = should_build;
        self
    }

    pub fn should_install(mut self, should_install: bool) -> Self {
        self.should_install = should_install;
        self
    }

    pub fn build(self) -> BuildCommand {
        BuildCommand {
            recipe_path: self.recipe_path,
            build_path: self.build_path,
            install_path: self.install_path,
            package_path: self.package_path,
            source_path: self.source_path,
            should_configure: self.should_configure,
            should_build: self.should_build,
            should_install: self.should_install,
        }
    }
}

impl BuildCommand {
    pub fn args(&self) -> Result<Vec<String>, ConanBuildError> {
        let mut args: Vec<&str> = Vec::new();

        // NOTE: Here self.recipe_path is guaranteed to be Some
        args.extend(&["build", self.recipe_path.as_ref().unwrap().to_str().unwrap()]);

        if let Some(build_path) = &self.build_path {
            args.extend(&["--build-folder", build_path.to_str().unwrap()]);
        }

        if let Some(install_path) = &self.install_path {
            args.extend(&["--install-folder", install_path.to_str().unwrap()]);
        }

        if let Some(package_path) = &self.package_path {
            args.extend(&["--package-folder", package_path.to_str().unwrap()]);
        }

        if let Some(source_path) = &self.source_path {
            args.extend(&["--source-folder", source_path.to_str().unwrap()]);
        }

        if self.should_configure {
            args.push("--configure");
        }

        if self.should_build {
            args.push("--build");
        }

        if self.should_install {
            args.push("--install");
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
