use std::path::{Path, PathBuf};

use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{config::Config, node};

// #[derive(Debug, Deserialize, Serialize, PartialEq)]
// #[serde(rename_all = "snake_case", tag = "type")]
// pub enum Link {
//     Resolved(ResolvedLink),
//     Ghost(GhostLink),
// }

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum To {
    Id(node::Id),
    Ghost(FilePart),
}

/// Fully resolved link.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Link {
    pub from: node::Id,
    pub to: To,
    #[serde(flatten)]
    pub location: Option<Location>,
    pub alias: Option<String>,
}

/// Represents a location inside the Node.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Location {
    /// stable and unique identifier inside the Node
    /// like a Typst label
    /// always preferred if possible.
    Label(String),

    /// the full path to a heading with no skips, eg. #operations#addition
    /// may not always resolve.
    HeadingPath(Vec<String>),
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FilePart {
    /// Matches one name under any directory recursively.
    Name(String),
    /// Matches one name under some directory path.
    ///
    /// eg.
    /// c/
    ///   dsa/
    ///     matrix.typ
    ///
    /// linalg/
    ///   matrix.typ
    ///
    /// (["c"], "matrix.typ") would match the first matrix.typ.
    /// WARNING: this may contain aliases that need to be resolved first.
    PathAndName(Vec<String>, String),
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HeadingPart {
    /// Matches one heading under any heading path.
    /// In typst, this also matches labels.
    Heading(String),
    /// Matches one heading under some heading path.
    ///
    /// eg.
    /// = Topic 1
    /// == Subtopic 1
    /// === Lorem
    ///
    /// = Topic 2
    /// == Subtopic 2
    /// === Lorem
    ///
    /// (["topic-2"], "lorem") would match the second lorem.
    PathAndHeading(Vec<String>, String),
}

/// Generic form of an unresolved link, which is pretty much what we get straight out of the user.
pub struct UnresolvedLink {
    pub from: PathBuf,
    pub file_part: FilePart,
    pub heading_part: Option<HeadingPart>,
    pub alias: Option<String>,
}

#[derive(Error, Debug, Diagnostic)]
pub enum Error<'a> {
    #[error("node at {0} is untracked")]
    #[diagnostic(help("add the file to the node database tracking it with TODO"))]
    UntrackedNode(&'a Path),

    #[error("duplicate name {0}")]
    #[diagnostic(help("try specifying a path for your link"))]
    DuplicateName(&'a str),

    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
}

impl UnresolvedLink {
    fn resolve(&self, config: &Config, nodes: node::Db) -> Result<Link, Error<'_>> {
        // get the id for from
        let from_id = match nodes
            .nodes
            .binary_search_by_key(&self.from.as_path(), |node| node.path.as_path())
        {
            Ok(index) => &nodes.nodes[index].id, // WARNING: this should never crash but you know...
            Err(_) => return Err(Error::UntrackedNode(&self.from)),
        };

        let to_id = match &self.file_part {
            FilePart::Name(name) => {
                let found: Vec<&node::Node> = nodes
                    .nodes
                    .iter()
                    .filter(|node| node.names.contains(name))
                    .collect();
                if found.len() > 1 {
                    return Err(Error::DuplicateName(name));
                } else {
                }
            }
            FilePart::PathAndName(path, name) => todo!(),
        };

        Ok(Link {
            from: from_id.clone(),
            to: todo!(),
            location: todo!(),
            alias: todo!(),
        })
    }
}

/// The links database found in `build/links.toml`.
/// It is not meant to be touched by the users!
#[derive(Debug, Deserialize, Serialize)]
pub struct Db {
    #[serde(rename = "link")]
    pub links: Vec<Link>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_links_db_serializing() {
        let db = Db {
            links: vec![
                Link {
                    from: "id1".into(),
                    to: To::Id("id2".into()),
                    location: Some(Location::Label("addition".into())),
                    alias: Some("matrix addition".into()),
                },
                Link {
                    from: "id1".into(),
                    to: To::Id("id2".into()),
                    location: Some(Location::HeadingPath(vec![
                        "operations".into(),
                        "addition".into(),
                    ])),
                    alias: Some("perform an addition".into()),
                },
                Link {
                    from: "id1".into(),
                    to: To::Ghost(FilePart::PathAndName(
                        vec!["linalg".into()],
                        "matrix".into(),
                    )),
                    location: None,
                    alias: None,
                },
            ],
        };

        let expect = r#"[[link]]
type = "resolved"
from = "id1"
to = "id2"
label = "addition"
alias = "matrix addition"

[[link]]
type = "resolved"
from = "id1"
to = "id2"
heading_path = [
    "operations",
    "addition",
]
alias = "perform an addition"

[[link]]
type = "ghost"
from = "id1"
path_and_name = [
    ["linalg"],
    "matrix",
]
"#;

        println!("{}", toml::to_string_pretty(&db).unwrap());
        assert_eq!(toml::to_string_pretty(&db).unwrap(), expect)
    }

    #[test]
    fn test_links_db_parsing() {
        let raw = r#"
        [[link]]
        type = "resolved"
        from = "id1"
        to = "id2"
        label = "addition"
        alias = "matrix addition"

        [[link]]
        type = "resolved"
        from = "id1"
        to = "id1"
        heading_path = [
            "operations",
            "addition",
        ]
        alias = "perform an addition"

        [[link]]
        type = "ghost"
        from = "id1"
        name = "vector"
        "#;

        let db: Db = toml::from_str(raw).unwrap();
        assert_eq!(
            db.links,
            [
                Link {
                    from: "id1".into(),
                    to: To::Id("id2".into()),
                    location: Some(Location::Label("addition".into())),
                    alias: Some("matrix addition".into()),
                },
                Link {
                    from: "id1".into(),
                    to: To::Id("id1".into()),
                    location: Some(Location::HeadingPath(vec![
                        "operations".into(),
                        "addition".into(),
                    ])),
                    alias: Some("perform an addition".into()),
                },
                Link {
                    from: "id1".into(),
                    to: To::Ghost(FilePart::Name("vector".into())),
                    location: Some(Location::HeadingPath(vec![
                        "operations".into(),
                        "addition".into(),
                    ])),
                    alias: Some("perform an addition".into()),
                },
            ]
        )
    }
}
