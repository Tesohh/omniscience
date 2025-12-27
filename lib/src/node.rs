use std::path::{Path, PathBuf};

use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{config::Config, link, node};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Id(pub String);

impl Id {
    pub fn new() -> Self {
        let date = chrono::Local::now();
        // YYYYMMDDHHMM
        Self(date.format("%Y%m%d%H%m").to_string())
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Id(value)
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Id(String::from(value))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct File {
    pub id: String,
    pub path: PathBuf,
}

impl File {
    pub fn into_node(self, names: Vec<String>, tags: Vec<String>) -> Node {
        Node {
            id: Id(self.id),
            path: self.path,
            kind: NodeKind::File,
            names,
            tags,
        }
    }
}

/// the nodes database found in `nodes.toml`
/// this is **NOT** the ultimate source of truth for nodes.
/// it's just the "user facing" nodes database.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UserDb {
    #[serde(rename = "file")]
    pub files: Vec<File>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum NodeKind {
    #[serde(rename = "file")]
    File,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
/// Fully resolved node,
/// made by taking a `File` or (in future) other kinds of nodes,
/// finding names and tags and putting them in here.
pub struct Node {
    pub id: Id,
    pub path: PathBuf,
    pub kind: NodeKind,
    #[serde(default)]
    pub names: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
/// The nodes database found in `build/nodes.toml`.
/// which will contain everything from `nodes.toml` + additional metadata found from files (eg. tags)
/// *this is the ultimate source of truth for nodes.*
pub struct Db {
    #[serde(rename = "node")]
    pub nodes: Vec<Node>,
}

#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error("node at {0} is untracked or does not exist")]
    #[diagnostic(help("add the file to the node database tracking it with TODO"))]
    UntrackedNode(PathBuf),

    #[error("node with name {0} not found")]
    NameNotFound(String),

    #[error("duplicate name {0}")]
    #[diagnostic(help("try specifying a path for your link"))]
    DuplicateName(String),

    #[error("empty path in FilePart::PathAndName")]
    EmptyPath,
}

impl Db {
    /// Finds the id of a node from a system path
    pub fn find_abs(&self, path: &Path, _: &Config) -> Result<&'_ Node, Error> {
        // TODO: consider canonicalizing
        match self
            .nodes
            .binary_search_by_key(&path, |node| node.path.as_path())
        {
            Ok(index) => Ok(&self.nodes[index]), // WARNING: this should never crash but you know...
            Err(_) => Err(Error::UntrackedNode(path.to_path_buf())),
        }
    }

    /// Finds the id of a node from a FilePart
    pub fn find_from_filepart(
        &self,
        part: &link::FilePart,
        config: &Config,
    ) -> Result<&'_ Node, Error> {
        match part {
            link::FilePart::Name(name) => {
                let found: Vec<&node::Node> = self
                    .nodes
                    .iter()
                    .filter(|node| node.names.contains(name))
                    .collect();
                if found.len() > 1 {
                    Err(Error::DuplicateName(name.clone()))
                } else if found.is_empty() {
                    Err(Error::NameNotFound(name.clone()))
                } else {
                    Ok(found[0])
                }
            }
            link::FilePart::PathAndName(fake_path, name) => {
                let path: PathBuf = if !fake_path.is_empty() {
                    if let Some(target) = config.dir_aliases.get(&fake_path[0]) {
                        target.components().map(|c| c.as_os_str()).collect()
                    } else {
                        fake_path.join("/").into()
                    }
                } else {
                    return Err(Error::EmptyPath);
                };

                let found: Vec<&node::Node> = self
                    .nodes
                    .iter()
                    .filter(|node| node.names.contains(name))
                    .filter(|node| node.path.starts_with(&path))
                    .collect();
                if found.len() > 1 {
                    Err(Error::DuplicateName(name.clone()))
                } else if found.is_empty() {
                    Err(Error::NameNotFound(name.clone()))
                } else {
                    Ok(found[0])
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::config::Project;

    use super::*;

    #[test]
    fn test_find_by_name() {
        let db = Db {
            nodes: vec![
                Node {
                    id: "id1".into(),
                    path: "linear-algebra".into(),
                    kind: NodeKind::File,
                    names: vec!["vector".into()],
                    tags: vec![],
                },
                Node {
                    id: "id2".into(),
                    path: "programming/rust".into(),
                    kind: NodeKind::File,
                    names: vec!["borrow-checker".into(), "borrow".into()],
                    tags: vec![],
                },
            ],
        };

        let config = Config {
            project: Project {
                name: String::from("project"),
            },
            dir_aliases: HashMap::from([("linalg".into(), "linear-algebra".into())]),
        };

        let found = db
            .find_from_filepart(&link::FilePart::Name("vector".into()), &config)
            .unwrap();
        assert_eq!(found.id, "id1".into());

        let found = db
            .find_from_filepart(&link::FilePart::Name("borrow".into()), &config)
            .unwrap();
        assert_eq!(found.id, "id2".into());
    }

    #[test]
    fn test_find_by_name_with_path() {
        let db = Db {
            nodes: vec![
                Node {
                    id: "id1".into(),
                    path: "linear-algebra".into(),
                    kind: NodeKind::File,
                    names: vec!["vector".into()],
                    tags: vec![],
                },
                Node {
                    id: "id2".into(),
                    path: "programming/rust".into(),
                    kind: NodeKind::File,
                    names: vec!["vector".into()],
                    tags: vec![],
                },
            ],
        };

        let config = Config {
            project: Project {
                name: String::from("project"),
            },
            dir_aliases: HashMap::from([("linalg".into(), "linear-algebra".into())]),
        };

        let found = db
            .find_from_filepart(
                &link::FilePart::PathAndName(vec!["linalg".into()], "vector".into()),
                &config,
            )
            .unwrap();
        assert_eq!(found.id, "id1".into());

        let found = db
            .find_from_filepart(
                &link::FilePart::PathAndName(vec!["programming".into()], "vector".into()),
                &config,
            )
            .unwrap();
        assert_eq!(found.id, "id2".into());

        let found = db
            .find_from_filepart(
                &link::FilePart::PathAndName(vec!["programming/rust".into()], "vector".into()),
                &config,
            )
            .unwrap();
        assert_eq!(found.id, "id2".into());
    }

    #[test]
    #[should_panic]
    fn test_find_by_name_fail() {
        let db = Db {
            nodes: vec![
                Node {
                    id: "id1".into(),
                    path: "linear-algebra".into(),
                    kind: NodeKind::File,
                    names: vec!["vector".into()],
                    tags: vec![],
                },
                Node {
                    id: "id2".into(),
                    path: "programming/rust".into(),
                    kind: NodeKind::File,
                    names: vec!["vector".into()],
                    tags: vec![],
                },
            ],
        };

        let config = Config {
            project: Project {
                name: String::from("project"),
            },
            dir_aliases: HashMap::new(),
        };

        db.find_from_filepart(&link::FilePart::Name("vector".into()), &config)
            .unwrap();
    }

    #[test]
    fn test_user_nodes_db_parsing() {
        let raw = r#"
        [[file]]
        id = "id1"
        path = "cs/c/matrix.md"

        [[file]]
        id = "id2"
        path = "cs/discrete-math/proofs/proof-by-induction.typ"
        "#;

        let db: UserDb = toml::from_str(raw).unwrap();
        assert_eq!(
            db.files,
            [
                File {
                    id: "id1".into(),
                    path: "cs/c/matrix.md".into()
                },
                File {
                    id: "id2".into(),
                    path: "cs/discrete-math/proofs/proof-by-induction.typ".into()
                }
            ]
        )
    }

    #[test]
    fn test_nodes_db_parsing() {
        let raw = r#"
        [[node]]
        id = "id1"
        path = "cs/c/matrix.md"
        kind = "file"
        names = ["matrix"]
        tags = ["programming"]

        [[node]]
        id = "id2"
        path = "cs/discrete-math/proofs/proof-by-induction.typ"
        kind = "file"
        names = ["proof-by-induction", "induction"]"#;

        let db: Db = toml::from_str(raw).unwrap();

        assert_eq!(
            db.nodes,
            [
                Node {
                    id: Id("id1".into()),
                    path: "cs/c/matrix.md".into(),
                    kind: NodeKind::File,
                    names: vec!["matrix".into()],
                    tags: vec!["programming".into()]
                },
                Node {
                    id: Id("id2".into()),
                    path: "cs/discrete-math/proofs/proof-by-induction.typ".into(),
                    kind: NodeKind::File,
                    names: vec!["proof-by-induction".into(), "induction".into()],
                    tags: vec![]
                }
            ]
        )
    }
}
