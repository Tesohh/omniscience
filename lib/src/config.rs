use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
};

use miette::Diagnostic;
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use std::hash::{BuildHasherDefault, DefaultHasher};
use thiserror::Error;

static OMNI_TOML: &str = "omni.toml";

/// config contained in `omni.toml`,
/// which also counts as project root.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub project: Project,

    // we need a non-random hasher because wasi doesn't support having a random seed
    #[cfg(target_arch = "wasm32")]
    #[serde(default)]
    pub dir_aliases: HashMap<String, PathBuf, BuildHasherDefault<DefaultHasher>>,

    #[cfg(not(target_arch = "wasm32"))]
    #[serde(default)]
    pub dir_aliases: HashMap<String, PathBuf>,
}

// #[derive(thiserror::Error, miette::Diagnostic, Debug)]
// pub enum OmniPathError {
//     #[error("path is empty")]
//     PathEmpty,
//     #[error("invalid path")]
//     #[diagnostic(help(
//         "path must be a omni style path (starts from src, can have aliases, no file extension eg. `linalg/matrix`)"
//     ))]
//     InvalidPath,
//     #[error("path contains invalid unicode")]
//     InvalidUnicode,
// }
//
// impl Config {
//     pub fn parse_omni_path(&self, path: impl AsRef<Path>) -> Result<PathBuf, OmniPathError> {
//         let mut path_components = path.as_ref().components();
//         let mut target = PathBuf::new();
//         let mut first = path_components.next().ok_or(OmniPathError::PathEmpty)?;
//
//         // TODO: use a custom OmniPathType similar to FilePart
//
//         if let Component::Normal(str) = first
//             && str == "src"
//         {
//             first = path_components.next().ok_or(OmniPathError::PathEmpty)?;
//         } else {
//             target.push("src");
//         }
//
//         // parse potential alias
//         if let Component::Normal(first) = first {
//             let first = first.to_str().ok_or(OmniPathError::InvalidUnicode)?;
//             match self.dir_aliases.get(first) {
//                 Some(first) => target.push(first),
//                 None => target.push(first),
//             };
//         };
//
//         // add all other components
//         for component in path_components {
//             target.push(component);
//         }
//
//         Ok(target)
//     }
// }

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
