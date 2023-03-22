extern crate serde;
extern crate serde_json;

use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

fn deserialize_optional_string_or_string_vec<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct JsonStringOrStringVecVisitor;

    impl<'de> Visitor<'de> for JsonStringOrStringVecVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("null, a string, or an array of strings")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            Ok(Some(v.to_owned()))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
            deserializer.deserialize_any(JsonStringOrStringVecVisitor)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut concatenated = String::new();
            while let Some(elem) = seq.next_element::<String>()? {
                concatenated += &elem;
            }

            Ok(Some(concatenated))
        }
    }

    deserializer.deserialize_option(JsonStringOrStringVecVisitor)
}

#[derive(Serialize, Deserialize)]
pub struct BuildDependency {
    pub(crate) version: String,
    #[serde(deserialize_with = "deserialize_optional_string_or_string_vec")]
    pub(crate) description: Option<String>,
    pub(crate) rootpath: String,
    pub(crate) sysroot: String,
    pub(crate) include_paths: Vec<String>,
    pub(crate) lib_paths: Vec<String>,
    pub(crate) bin_paths: Vec<String>,
    pub(crate) build_paths: Vec<String>,
    pub(crate) res_paths: Vec<String>,
    pub(crate) libs: Vec<String>,
    pub(crate) system_libs: Option<Vec<String>>,
    pub(crate) defines: Vec<String>,
    pub(crate) cflags: Vec<String>,
    pub(crate) cxxflags: Option<Vec<String>>,
    pub(crate) sharedlinkflags: Vec<String>,
    pub(crate) exelinkflags: Vec<String>,
    pub(crate) cppflags: Option<Vec<String>>,
    pub(crate) name: String,
}

impl BuildDependency {
    pub fn get_root_dir(&self) -> Option<&str> {
        Some(self.rootpath.as_str())
    }

    pub fn get_library_dir(&self) -> Option<&str> {
        self.lib_paths.get(0).map(|x| &**x)
    }

    pub fn get_include_dir(&self) -> Option<&str> {
        self.include_paths.get(0).map(|x| &**x)
    }

    pub fn get_binary_dir(&self) -> Option<&str> {
        self.bin_paths.get(0).map(|x| &**x)
    }
}
