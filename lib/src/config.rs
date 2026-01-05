use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use std::hash::{BuildHasherDefault, DefaultHasher};
use thiserror::Error;

static OMNI_TOML: &str = "omni.toml";

/// config contained in `omni.toml`,
/// which also counts as project root.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Config {
    pub project: Project,

    #[serde(default)]
    pub typst: Typst,

    // we need a non-random hasher because wasi doesn't support having a random seed
    #[cfg(target_arch = "wasm32")]
    #[serde(default)]
    pub dir_aliases: HashMap<String, Utf8PathBuf, BuildHasherDefault<DefaultHasher>>,

    #[cfg(not(target_arch = "wasm32"))]
    #[serde(default)]
    pub dir_aliases: HashMap<String, Utf8PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub name: String,

    /// a single directory name where all your content should be stored.
    /// if empty, no "prefix" will be used.
    pub prefix_dir: Option<String>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: String::from("project"),
            prefix_dir: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Typst {
    #[serde(default)]
    pub output_format: TypstOutputFormat,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TypstOutputFormat {
    Html,
    Pdf,
    #[serde(rename = "html+pdf")]
    HtmlAndPdf,
}

impl Default for TypstOutputFormat {
    fn default() -> Self {
        Self::HtmlAndPdf
    }
}

#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error("the pwd provided is not a directory. the developer did something wrong!")]
    PwdIsAFile,
    #[error("no project root found")]
    #[diagnostic(help("try executing this command in a omniscience project directory"))]
    NoProjectRoot,
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
}

/// returns the parent directory that contains omni.toml, if it's found, otherwise `Error::NoProjectRoot`
pub fn find_project_root(pwd: impl AsRef<Utf8Path>) -> Result<Utf8PathBuf, Error> {
    let current = pwd.as_ref().canonicalize_utf8()?;

    for ancestor in current.ancestors() {
        if ancestor.join(OMNI_TOML).exists() {
            return Ok(ancestor.to_path_buf());
        }
    }

    Err(Error::NoProjectRoot)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_config_deserialization_all_specified() {
        let raw_toml = r#"
        [project]
        name = "my_proj"
        prefix_dir = "src"

        [typst]
        output_format = "html+pdf"

        [dir_aliases]
        linalg = "Linear Algebra"
        "#;

        let config: Config = toml::from_str(raw_toml).unwrap();
        assert_eq!(
            config,
            Config {
                project: Project {
                    name: "my_proj".into(),
                    prefix_dir: Some("src".into()),
                },
                typst: Typst {
                    output_format: TypstOutputFormat::HtmlAndPdf,
                },
                dir_aliases: HashMap::from([("linalg".into(), "Linear Algebra".into())])
            }
        )
    }

    #[test]
    fn test_config_deserialization_bare_minimum() {
        let raw_toml = r#"
        [project]
        name = "my_proj"
        "#;

        let config: Config = toml::from_str(raw_toml).unwrap();
        assert_eq!(
            config,
            Config {
                project: Project {
                    name: "my_proj".into(),
                    prefix_dir: None,
                },
                typst: Typst::default(),
                dir_aliases: HashMap::new()
            }
        )
    }

    #[test]
    fn test_project_root_resolution_fail() {
        let mut path = Utf8PathBuf::try_from(tempdir().unwrap().path().to_path_buf()).unwrap();
        path.push("notes/linalg/deep/nested");

        std::fs::create_dir_all(&path).unwrap();
        // just create the path, but never add omni.toml

        assert!(matches!(
            find_project_root(&path),
            Err(Error::NoProjectRoot)
        ));
    }

    #[test]
    fn test_project_root_resolution() {
        let root =
            Utf8PathBuf::try_from(tempdir().unwrap().path().canonicalize().unwrap()).unwrap();
        let mut path = root.clone();
        path.push("notes/linalg/deep/nested");

        std::fs::create_dir_all(&path).unwrap();
        std::fs::File::create(root.join(OMNI_TOML)).unwrap();

        assert_eq!(find_project_root(&path).unwrap(), root);
    }

    #[test]
    fn test_project_root_resolution_same_path() {
        let temp = tempdir().unwrap(); // needs to be given a binding or else it would be dropped
        let root = Utf8PathBuf::try_from(temp.path().canonicalize().unwrap()).unwrap();
        std::fs::File::create(root.join(OMNI_TOML)).unwrap();
        assert_eq!(find_project_root(&root).unwrap(), root);
    }
}
