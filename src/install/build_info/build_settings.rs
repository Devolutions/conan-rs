use std::env;

use serde::{Deserialize, Serialize};

/// Conan build type
#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum BuildType {
    None,
    Debug,
    Release,
    RelWithDebInfo,
    MinSizeRel,
}

impl ToString for BuildType {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl BuildType {
    pub fn as_str(&self) -> &str {
        match self {
            &BuildType::None => "None",
            &BuildType::Debug => "Debug",
            &BuildType::Release => "Release",
            &BuildType::RelWithDebInfo => "RelWithDebInfo",
            &BuildType::MinSizeRel => "MinSizeRel",
        }
    }
}

/// Conan build settings
#[derive(Serialize, Deserialize)]
pub struct BuildSettings {
    pub(crate) arch: Option<String>,
    pub(crate) arch_build: Option<String>,
    pub(crate) build_type: Option<String>,
    pub(crate) compiler: Option<String>,
    #[serde(rename = "compiler.libcxx")]
    pub(crate) compiler_libcxx: Option<String>,
    #[serde(rename = "compiler.runtime")]
    pub(crate) compiler_runtime: Option<String>,
    #[serde(rename = "compiler.version")]
    pub(crate) compiler_version: Option<String>,
    pub(crate) os: Option<String>,
    pub(crate) os_build: Option<String>,
}

impl Default for BuildSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildSettings {
    pub fn new() -> Self {
        Self {
            arch: None,
            arch_build: None,
            build_type: None,
            compiler: None,
            compiler_libcxx: None,
            compiler_runtime: None,
            compiler_version: None,
            os: None,
            os_build: None,
        }
    }

    pub fn args(&self) -> Vec<String> {
        let mut settings = Vec::new();

        if let Some(arch) = &self.arch {
            settings.push(format!("{}={}", "arch", arch));
        }

        if let Some(arch_build) = &self.arch_build {
            settings.push(format!("{}={}", "arch_build", arch_build));
        }

        if let Some(build_type) = self.build_type.clone().or_else(|| self.detect_build_type()) {
            settings.push(format!("{}={}", "build_type", build_type));
        }

        if let Some(compiler) = &self.compiler {
            settings.push(format!("{}={}", "compiler", compiler));
        }

        if let Some(compiler_libcxx) = &self.compiler_libcxx {
            settings.push(format!("{}={}", "compiler.libcxx", compiler_libcxx));
        }

        if let Some(compiler_runtime) = &self.compiler_runtime {
            settings.push(format!("{}={}", "compiler.runtime", compiler_runtime));
        }

        if let Some(compiler_version) = &self.compiler_version {
            settings.push(format!("{}={}", "compiler.version", compiler_version));
        }

        if let Some(os) = &self.os {
            settings.push(format!("{}={}", "os", os));
        }

        if let Some(os_build) = &self.os_build {
            settings.push(format!("{}={}", "os_build", os_build));
        }

        settings
            .iter()
            .map(|x| ["-s".to_string(), x.clone()])
            .collect::<Vec<[String; 2]>>()
            .concat()
    }

    pub fn arch(mut self, arch: String) -> Self {
        self.arch = Some(arch);
        self
    }

    pub fn arch_build(mut self, arch_build: String) -> Self {
        self.arch_build = Some(arch_build);
        self
    }

    pub fn build_type<T: ToString>(mut self, build_type: T) -> Self {
        self.build_type = Some(build_type.to_string());
        self
    }

    fn detect_build_type(&self) -> Option<String> {
        if self.build_type.is_some() {
            return self.build_type.clone();
        } else if let Ok(profile) = env::var("PROFILE") {
            return match profile.as_str() {
                "debug" => Some("Debug".into()),
                "release" => Some("Release".into()),
                _ => None,
            };
        }
        None
    }

    pub fn compiler(mut self, compiler: String) -> Self {
        self.compiler = Some(compiler);
        self
    }

    pub fn compiler_libcxx(mut self, compiler_libcxx: String) -> Self {
        self.compiler_libcxx = Some(compiler_libcxx);
        self
    }

    pub fn compiler_runtime(mut self, compiler_runtime: String) -> Self {
        self.compiler_runtime = Some(compiler_runtime);
        self
    }

    pub fn compiler_version(mut self, compiler_version: String) -> Self {
        self.compiler_version = Some(compiler_version);
        self
    }

    pub fn os(mut self, os: String) -> Self {
        self.os = Some(os);
        self
    }

    pub fn os_build(mut self, os_build: String) -> Self {
        self.os_build = Some(os_build);
        self
    }
}
