use miette::Diagnostic;
use thiserror::Error;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct OmniPath {
    pub path: Vec<String>,
    pub name: String,

    unaliased: bool,
}

#[derive(Error, Diagnostic, Debug)]
pub enum Error {
    #[error("path part of omni path is empty")]
    EmptyPath,
    #[error("omni.toml contains an empty dir_alias")]
    EmptyPathInConfig,
}

impl OmniPath {
    pub fn new(path: Vec<String>, name: String) -> Self {
        Self {
            path,
            name,
            unaliased: false,
        }
    }

    pub fn is_unaliased(&self) -> bool {
        self.unaliased
    }

    pub fn unalias(self, config: &Config) -> Result<Self, Error> {
        if self.unaliased {
            return Ok(self);
        }

        let new_path: Vec<String> = if !self.path.is_empty() {
            if let Some(alias_target) = config.dir_aliases.get(&self.path[0]) {
                if alias_target.as_os_str().is_empty() {
                    return Err(Error::EmptyPathInConfig);
                }

                alias_target
                    .join(self.path[1..].join("/"))
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect()
            } else {
                // no alias found. just return the original path
                self.path
            }
        } else {
            return Err(Error::EmptyPath);
        };

        Ok(Self {
            path: new_path,
            name: self.name,
            unaliased: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::config::Project;

    use super::*;

    #[test]
    fn test_unaliasing() {
        let config = Config {
            project: Project {
                name: "proj".into(),
            },
            dir_aliases: HashMap::from([("linalg".into(), "cs/linear-algebra".into())]),
        };

        let op = OmniPath::new(vec!["linalg".into()], "matrix".into());
        assert_eq!(op.unalias(&config).unwrap().path, ["cs", "linear-algebra"]);

        let op = OmniPath::new(
            vec!["linalg".into(), "spectral-analysis".into()],
            "determinant".into(),
        );
        assert_eq!(
            op.unalias(&config).unwrap().path,
            ["cs", "linear-algebra", "spectral-analysis"]
        );

        let op = OmniPath::new(vec!["cs".into(), "c".into()], "matrix".into());
        assert_eq!(op.unalias(&config).unwrap().path, ["cs", "c"]);
    }

    #[test]
    fn test_double_unaliasing() {
        let config = Config {
            project: Project {
                name: "proj".into(),
            },
            dir_aliases: HashMap::from([("linalg".into(), "cs/linear-algebra".into())]),
        };

        let op = OmniPath::new(
            vec!["linalg".into(), "spectral-analysis".into()],
            "determinant".into(),
        );

        assert_eq!(
            op.clone().unalias(&config).unwrap().path,
            op.unalias(&config).unwrap().unalias(&config).unwrap().path,
        );
    }
}
