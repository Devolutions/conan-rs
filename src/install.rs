#[cfg(test)]
mod test;

pub mod build_info;

mod profile;
mod remote;

use crate::util::find_program;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

use build_info::{build_settings::BuildSettings, BuildInfo};

#[derive(Debug, Error)]
pub enum ConanInstallError {
    #[error("Conan not found")]
    ConanNotFound,
    #[error("Failed to execute Conan: {0}")]
    ConanInstallFailed(#[from] std::io::Error),
    #[error("Install directory not found")]
    ConanInstallDirNotFound,
    #[error("Failed to convert output to UTF-8: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    Other(String),
}

/// Conan build policy
#[derive(Clone, PartialEq)]
pub enum BuildPolicy {
    Never,
    Always,
    Missing,
    Outdated,
}

/// "conan install" command runner
pub struct InstallCommand<'a> {
    profile_host: Option<&'a str>,
    profile_build: Option<&'a str>,
    remote: Option<&'a str>,
    build_settings: BuildSettings,
    build_options: Option<Vec<&'a str>>,
    build_policy: Option<BuildPolicy>,
    recipe_path: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    update_check: bool,
}

/// "conan install" command arguments builder
#[derive(Default)]
pub struct InstallCommandBuilder<'a> {
    profile_host: Option<&'a str>,
    profile_build: Option<&'a str>,
    remote: Option<&'a str>,
    build_settings: Option<BuildSettings>,
    build_options: Option<Vec<&'a str>>,
    build_policy: Option<BuildPolicy>,
    recipe_path: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    update_check: bool,
}

impl<'a> InstallCommandBuilder<'a> {
    pub fn new() -> InstallCommandBuilder<'a> {
        InstallCommandBuilder::default()
    }

    /// Apply the specified profile to the host machine.
    pub fn with_profile(self, profile: &'a str) -> Self {
        self.with_host_profile(profile)
    }

    /// Apply the specified profile to the host machine.
    pub fn with_host_profile(mut self, profile: &'a str) -> Self {
        self.profile_host = Some(profile);
        self
    }

    /// Apply the specified profile to the build machine.
    pub fn with_build_profile(mut self, profile: &'a str) -> Self {
        self.profile_build = Some(profile);
        self
    }

    pub fn with_remote(mut self, remote: &'a str) -> Self {
        self.remote = Some(remote);
        self
    }

    pub fn build_settings(mut self, build_settings: BuildSettings) -> Self {
        self.build_settings = Some(build_settings);
        self
    }

    pub fn build_policy(mut self, build_policy: BuildPolicy) -> Self {
        self.build_policy = Some(build_policy);
        self
    }

    pub fn with_options(mut self, opts: &[&'a str]) -> Self {
        if self.build_options.is_none() {
            self.build_options = Some(Vec::new());
        }
        // NOTE: Here self.build_options is guaranteed to be Some
        self.build_options.as_mut().unwrap().extend(opts);
        self
    }

    pub fn recipe_path(mut self, recipe_path: &Path) -> Self {
        self.recipe_path = Some(recipe_path.to_path_buf());
        self
    }

    pub fn output_dir(mut self, output_dir: &Path) -> Self {
        self.output_dir = Some(output_dir.to_path_buf());
        self
    }

    pub fn update_check(mut self) -> Self {
        self.update_check = true;
        self
    }

    pub fn build(self) -> InstallCommand<'a> {
        InstallCommand {
            profile_host: self.profile_host,
            profile_build: self.profile_build,
            remote: self.remote,
            build_settings: self.build_settings.unwrap_or_default(),
            build_options: self.build_options,
            build_policy: self.build_policy,
            recipe_path: self.recipe_path,
            output_dir: self.output_dir,
            update_check: self.update_check,
        }
    }
}

impl<'a> InstallCommand<'a> {
    pub fn args(&self) -> Result<Vec<String>, ConanInstallError> {
        let mut args: Vec<&str> = Vec::new();

        args.push("install");
        args.extend(&["-g", "json"]);

        if let Some(profile) = &self.profile_host {
            args.extend(&["--profile:host", profile]);
        }

        if let Some(profile) = &self.profile_build {
            args.extend(&["--profile:build", profile]);
        }

        if let Some(remote) = &self.remote {
            args.extend(&["-r", remote]);
        }

        if self.update_check {
            args.push("-u");
        }

        if let Some(build_policy) = &self.build_policy {
            match build_policy {
                BuildPolicy::Never => {
                    args.extend(&["-b", "never"]);
                }
                BuildPolicy::Always => {
                    args.extend(&["-b"]);
                }
                BuildPolicy::Missing => {
                    args.extend(&["-b", "missing"]);
                }
                BuildPolicy::Outdated => {
                    args.extend(&["-b", "outdated"]);
                }
            }
        }

        if let Some(build_options) = &self.build_options {
            args.extend(build_options.iter().map(|x| ["-o", *x]).flatten());
        }

        let output_dir = self.output_dir();
        if let Some(output_dir) = &output_dir {
            let current_dir = env::current_dir()?.to_path_buf();
            if output_dir != &current_dir {
                args.extend(&["-if", output_dir.to_str().unwrap()]);
                args.extend(&["-of", output_dir.to_str().unwrap()]);
            }
        }

        let build_settings_args = self.build_settings.args();
        args.extend(build_settings_args.iter().map(String::as_str));

        if let Some(recipe_path) = &self.recipe_path {
            args.push(recipe_path.to_str().unwrap());
        }

        Ok(args.iter().map(|x| x.to_string()).collect())
    }

    pub fn output_dir(&self) -> Option<PathBuf> {
        self.output_dir
            .clone()
            .or_else(|| env::var("OUT_DIR").ok().map(PathBuf::from))
            .or_else(|| env::current_dir().ok())
    }

    pub fn output_file(&self) -> Option<PathBuf> {
        let mut output_file = self.output_dir()?;
        output_file.push("conanbuildinfo.json");
        Some(output_file)
    }

    pub fn generate(&self) -> Option<BuildInfo> {
        let args = self.args().ok()?;
        let program = find_program()?;
        let output_file = self.output_file()?;
        let mut command = Command::new(program);
        if command.args(args).status().is_ok() {
            BuildInfo::from_file(output_file.as_path())
        } else {
            None
        }
    }

    pub fn generate_if_no_buildinfo(&self) -> Option<BuildInfo> {
        BuildInfo::from_file(self.output_file()?.as_path()).or_else(|| self.generate())
    }
}
