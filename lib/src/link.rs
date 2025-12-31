use std::path::PathBuf;

use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{config::Config, node};

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
pub enum Error {
    #[error("{0}")]
    NodeDbError(#[from] node::Error),

    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
}

impl UnresolvedLink {
    /// consumes `self` to try and resolve the link.
    pub fn try_resolve(self, config: &Config, nodes: &node::Db) -> Result<Link, Error> {
        let from = nodes.find_abs(&self.from, config)?;

        let to_target = match nodes.find_from_filepart(&self.file_part, config) {
            Ok(node) => To::Id(node.id.clone()),
            Err(node::Error::NameNotFound(_)) => To::Ghost(self.file_part),
            Err(err) => return Err(err.into()),
        };

        Ok(Link {
            from: from.id.clone(),
            to: to_target,
            location: None, // TODO:
            alias: self.alias,
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
    use std::collections::HashMap;

    use crate::config::Project;

    use super::*;

    fn get_db() -> node::Db {
        node::Db {
            nodes: vec![
                node::Node {
                    id: "id1".into(),
                    path: "linear-algebra/vector.typ".into(),
                    kind: node::NodeKind::File,
                    title: "Vector".into(),
                    names: vec!["vector".into()],
                    tags: vec![],
                },
                node::Node {
                    id: "id2".into(),
                    path: "programming/rust/vector.typ".into(),
                    kind: node::NodeKind::File,
                    title: "Vector".into(),
                    names: vec!["vector".into()],
                    tags: vec![],
                },
            ],
        }
    }

    fn get_config() -> Config {
        Config {
            project: Project {
                name: String::from("project"),
            },
            dir_aliases: HashMap::from([("linalg".into(), "linear-algebra".into())]),
        }
    }

    #[test]
    fn test_link_resolving() {
        let db = get_db();
        let config = get_config();

        let link = UnresolvedLink {
            from: "linear-algebra/vector.typ".into(),
            file_part: FilePart::PathAndName(
                vec!["programming".into(), "rust".into()],
                "vector".into(),
            ),
            heading_part: None,
            alias: Some("alias".into()),
        };

        assert_eq!(
            link.try_resolve(&config, &db).unwrap(),
            Link {
                from: "id1".into(),
                to: To::Id("id2".into()),
                location: None,
                alias: Some("alias".into()),
            },
        );
    }

    #[test]
    fn test_link_resolve_to_ghost() {
        let db = get_db();
        let config = get_config();

        let link = UnresolvedLink {
            from: "linear-algebra/vector.typ".into(),
            file_part: FilePart::Name("matrix".into()),
            heading_part: None,
            alias: None,
        };

        assert_eq!(
            link.try_resolve(&config, &db).unwrap(),
            Link {
                from: "id1".into(),
                to: To::Ghost(FilePart::Name("matrix".into())),
                location: None,
                alias: None,
            },
        );
    }

    #[test]
    #[should_panic]
    fn test_link_resolve_fail() {
        let db = get_db();
        let config = get_config();

        let link = UnresolvedLink {
            from: "linear-algebra/matrix.typ".into(),
            file_part: FilePart::Name("vector".into()),
            heading_part: None,
            alias: None,
        };

        link.try_resolve(&config, &db).unwrap();
    }

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

        let expect = indoc::indoc!(
            r#"[[link]]
            from = "id1"
            label = "addition"
            alias = "matrix addition"

            [link.to]
            id = "id2"

            [[link]]
            from = "id1"
            heading_path = [
                "operations",
                "addition",
            ]
            alias = "perform an addition"

            [link.to]
            id = "id2"

            [[link]]
            from = "id1"

            [link.to.ghost]
            path_and_name = [
                ["linalg"],
                "matrix",
            ]
            "#
        );

        println!("{}", toml::to_string_pretty(&db).unwrap());
        assert_eq!(toml::to_string_pretty(&db).unwrap(), expect)
    }

    #[test]
    fn test_links_db_parsing() {
        let raw = r#"
        [[link]]
        from = "id1"
        label = "addition"
        alias = "matrix addition"

        [link.to]
        id = "id2"

        [[link]]
        from = "id1"
        heading_path = [
            "operations",
            "addition",
        ]
        alias = "perform an addition"

        [link.to]
        id = "id1"

        [[link]]
        type = "ghost"
        from = "id1"

        [link.to.ghost]
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
                    location: None,
                    alias: None,
                },
            ]
        )
    }
}
