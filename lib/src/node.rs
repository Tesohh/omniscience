use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct File {
    pub id: String,
    pub path: PathBuf,
}

impl File {
    pub fn into_node(self, names: Vec<String>, tags: Vec<String>) -> Node {
        Node {
            id: self.id,
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
    pub id: String,
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

#[cfg(test)]
mod tests {
    use super::*;

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
                    id: "id1".into(),
                    path: "cs/c/matrix.md".into(),
                    kind: NodeKind::File,
                    names: vec!["matrix".into()],
                    tags: vec!["programming".into()]
                },
                Node {
                    id: "id2".into(),
                    path: "cs/discrete-math/proofs/proof-by-induction.typ".into(),
                    kind: NodeKind::File,
                    names: vec!["proof-by-induction".into(), "induction".into()],
                    tags: vec![]
                }
            ]
        )
    }
}
