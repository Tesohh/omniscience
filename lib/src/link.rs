use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Link {
    pub from: String,
    pub to: String,
    pub heading_path: Option<String>,
    pub alias: Option<String>,
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

        [[link]]
        from = "id1"
        to = "id2"
        heading_path = "linky"
        alias = "link23 reference"
        "#;

        let db: Db = toml::from_str(raw).unwrap();
        assert_eq!(
            db.links,
            [
                Link {
                    from: "id1".into(),
                    to: "id2".into(),
                    heading_path: None,
                    alias: None,
                },
                Link {
                    from: "id1".into(),
                    to: "id2".into(),
                    heading_path: Some("linky".into()),
                    alias: Some("link23 reference".into()),
                }
            ]
        )
    }
}
