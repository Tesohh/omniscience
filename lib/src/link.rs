use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::node;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
/// Fully resolved link.
pub struct Link {
    pub from: node::Id,
    pub to: node::Id,
    #[serde(flatten)]
    pub location: Option<Location>,
    pub alias: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
/// Represents a location inside the Node.
pub enum Location {
    /// stable and unique identifier inside the Node
    /// like a Typst label
    /// always preferred if possible.
    #[serde(rename = "label")]
    Label(String),

    /// the full path to a heading with no skips, eg. #operations#addition
    /// may not always resolve.
    #[serde(rename = "heading_path")]
    HeadingPath(Vec<String>),
}

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

impl Link {
    // a full link contains all the following:
    // dir_path.name:heading_path.heading[alias]
    // and the possible forms are:
    // file part (mandatory):
    // - name
    // - dir_path.name
    // heading part (optional):
    // - heading
    // - heading_path.heading
    // alias part (optional):
    // - alias
}

#[derive(Debug, Deserialize, Serialize)]
/// The links database found in `build/links.toml`.
/// It is not meant to be touched by the users!
pub struct Db {
    #[serde(rename = "link")]
    pub links: Vec<Link>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_links_db_parsing() {
        let raw = r#"
        [[link]]
        from = "id1"
        to = "id2"
        label = "addition"
        alias = "matrix addition"

        [[link]]
        from = "id1"
        to = "id1"
        heading_path = [
            "operations",
            "addition",
        ]
        alias = "perform an addition"
        "#;

        let db: Db = toml::from_str(raw).unwrap();
        assert_eq!(
            db.links,
            [
                Link {
                    from: "id1".into(),
                    to: "id2".into(),
                    location: Some(Location::Label("addition".into())),
                    alias: Some("matrix addition".into()),
                },
                Link {
                    from: "id1".into(),
                    to: "id1".into(),
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
