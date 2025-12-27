use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum NodeKind {
    #[serde(rename = "file")]
    File,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
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
pub struct Db {
    #[serde(rename = "node")]
    pub nodes: Vec<Node>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
