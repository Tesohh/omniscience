use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, DefaultHasher},
    path::{Path, PathBuf},
};

use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

static OMNI_TOML: &str = "omni.toml";

/// config contained in `omni.toml`,
/// which also counts as project root.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub project: Project,

    // we need a non-random hasher because wasi doesn't support having a random seed
    #[serde(default)]
    pub dir_aliases: HashMap<String, PathBuf, BuildHasherDefault<DefaultHasher>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub name: String,
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
pub fn find_project_root(pwd: impl AsRef<Path>) -> Result<PathBuf, Error> {
    let mut current = pwd.as_ref().canonicalize()?;
    if current.is_file() {
        return Err(Error::PwdIsAFile);
    }

    loop {
        let target = current.join(OMNI_TOML);
        if target.exists() {
            return Ok(current);
        }

        match current.parent() {
            Some(parent) if parent != Path::new("") => current = parent.to_path_buf(),
            _ => break,
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

        [dir_aliases]
        linalg = "Linear Algebra"
        "#;

        let config: Config = toml::from_str(raw_toml).unwrap();
        assert_eq!(
            config,
            Config {
                project: Project {
                    name: "my_proj".into(),
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
                },
                dir_aliases: HashMap::new()
            }
        )
    }

    #[test]
    fn test_project_root_resolution_fail() {
        let mut path = tempdir().unwrap().path().to_path_buf();
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
        let root = tempdir().unwrap().path().canonicalize().unwrap();
        let mut path = root.clone();
        path.push("notes/linalg/deep/nested");

        std::fs::create_dir_all(&path).unwrap();
        std::fs::File::create(root.join(OMNI_TOML)).unwrap();

        assert_eq!(find_project_root(&path).unwrap(), root);
    }

    #[test]
    fn test_project_root_resolution_same_path() {
        let temp = tempdir().unwrap(); // needs to be given a binding or else it would be dropped
        let root = temp.path().canonicalize().unwrap();
        std::fs::File::create(root.join(OMNI_TOML)).unwrap();
        assert_eq!(find_project_root(&root).unwrap(), root);
    }
}
