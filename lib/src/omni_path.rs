use camino::{Utf8Path, Utf8PathBuf};
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

    #[error("path is empty while converting a &Path into a OmniPath")]
    EmptyPathInConversionFromPath,

    #[error("omni.toml contains an empty dir_alias")]
    EmptyPathInConfig,

    #[error("omni path must be unaliased before pathizing it.")]
    #[diagnostic(help("this should never happen, report it to the developer"))]
    PathizeNotUnaliased,

    #[error("invalid component in omni path")]
    #[diagnostic(help("omni style paths cannot contain `.` or `..`, start with a `/`..."))]
    InvalidComponent,
}

impl TryInto<Utf8PathBuf> for OmniPath {
    type Error = Error;

    /// tries to convert an OmniPath into a PathBuf.
    /// fails if the OmniPath is not unaliased.
    ///
    /// you might need to add the project root and set an extension later.
    fn try_into(self) -> Result<Utf8PathBuf, Self::Error> {
        if !self.unaliased {
            return Err(Self::Error::PathizeNotUnaliased);
        }

        Ok(Utf8PathBuf::from(self.path.join("/")).join(self.name))
    }
}

impl TryFrom<&Utf8Path> for OmniPath {
    type Error = Error;

    /// tries to convert a PathBuf into a OmniPath.
    /// it is not guaranteed that the OmniPath will be valid of course.
    fn try_from(value: &Utf8Path) -> Result<Self, Self::Error> {
        if value.as_os_str().is_empty() {
            return Err(Error::EmptyPathInConversionFromPath);
        }

        let mut path: Vec<_> = value
            .components()
            .map(|c| match c {
                camino::Utf8Component::Normal(c) => Ok(c.to_string()),
                _ => Err(Error::InvalidComponent),
            })
            .collect::<Result<Vec<_>, Self::Error>>()?;

        let name = path.pop().ok_or(Error::EmptyPathInConversionFromPath)?;

        Ok(OmniPath::new(path, name))
    }
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
                if alias_target.as_str().is_empty() {
                    return Err(Error::EmptyPathInConfig);
                }

                alias_target
                    .join(self.path[1..].join("/"))
                    .components()
                    .map(|c| c.to_string())
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

    pub fn try_from_path(path: impl AsRef<Utf8Path>) -> Result<Self, Error> {
        Self::try_from(path.as_ref())
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

    #[test]
    fn test_tryinto() {
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
            std::convert::TryInto::<Utf8PathBuf>::try_into(op.clone().unalias(&config).unwrap())
                .unwrap(),
            Utf8PathBuf::from("cs/linear-algebra/spectral-analysis/determinant")
        );
    }

    #[test]
    #[should_panic]
    fn test_tryinto_fail() {
        std::convert::TryInto::<Utf8PathBuf>::try_into(OmniPath::new(
            vec!["some".into()],
            "path".into(),
        ))
        .unwrap();
    }

    #[test]
    fn test_tryfrom() {
        let op = OmniPath::try_from(Utf8PathBuf::from("linalg/matrix").as_path()).unwrap();
        assert_eq!(op.path, ["linalg"]);
        assert_eq!(op.name, "matrix");
    }

    #[test]
    #[should_panic]
    fn test_tryfrom_fail() {
        OmniPath::try_from(Utf8PathBuf::from("../linalg/matrix").as_path()).unwrap();
    }
}
