use std::collections::HashMap;

use camino::Utf8Path;
use miette::Diagnostic;
use thiserror::Error;

use crate::{
    build::shallow::{ShallowError, shallow},
    config::Config,
    link, node,
};

#[derive(Debug, Error, Diagnostic)]
pub enum PartialError {
    #[error(transparent)]
    ShallowError(#[from] ShallowError),

    #[error(transparent)]
    NodeError(#[from] node::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    TomlSerializeError(#[from] toml::ser::Error),
}

pub fn partial(
    root: impl AsRef<Utf8Path>,
    config: &Config,
    nodes: &mut node::Db,
    links: &mut link::Db,
    file: &node::File,
    compile: bool,
) -> Result<(), PartialError> {
    // first shallow myself
    shallow(&root, config, nodes, links, file, compile)?;

    let mut dependants: Vec<node::File> = vec![];

    // find all ghosts that would be updated
    // also i might have changed title, which we cannot know.
    // so find all nodes that DONT have an alias and that point to me.

    let mut file_parts_cache: HashMap<link::FilePart, Option<&node::Node>> = HashMap::new();
    let mut node_id_cache: HashMap<node::Id, &node::Node> = HashMap::new();

    for link in &mut links.links {
        if let link::To::Ghost(filepart) = &link.to {
            let node = match file_parts_cache.get(filepart) {
                Some(Some(n)) => *n,
                Some(None) => {
                    continue;
                }
                None => match nodes.find_from_filepart(filepart, config) {
                    Ok(n) => {
                        file_parts_cache.insert(filepart.clone(), Some(n));
                        n
                    }
                    Err(node::Error::NameNotFound(_)) => {
                        file_parts_cache.insert(filepart.clone(), None);
                        continue;
                    }
                    Err(err) => return Err(err.into()),
                },
            };

            if node.id != file.id {
                continue;
            }

            // NOTE: we don't even really need to do this, as it is done later on by shallow.
            link.to = link::To::Id(file.id.clone());

            let other = match node_id_cache.get(&link.from) {
                Some(n) => *n,
                None => {
                    let n = nodes.find_from_id(&link.from, config)?;
                    node_id_cache.insert(link.from.clone(), n);
                    n
                }
            };

            dependants.push(node::File {
                id: other.id.clone(),
                path: other.path.clone(),
            });
        } else if let link::To::Id(maybe_my_id) = &link.to
            && compile
            && link.alias.is_none()
            && maybe_my_id == &file.id
        {
            // someone who links to me has no alias, so we might need to update their titles,
            // but only if we also compile, as this is only a visual change
            let other = match node_id_cache.get(&link.from) {
                Some(n) => *n,
                None => {
                    let n = nodes.find_from_id(&link.from, config)?;
                    node_id_cache.insert(link.from.clone(), n);
                    n
                }
            };

            dependants.push(node::File {
                id: other.id.clone(),
                path: other.path.clone(),
            });
        }
    }

    // SAVEPOINT(nodes): as dependants rely on the new node existing/being changed (if they're typst)
    let new_nodes_toml = toml::to_string(nodes)?;
    std::fs::write(root.as_ref().join("build/nodes.toml"), new_nodes_toml)?;

    for dependant in dependants {
        shallow(&root, config, nodes, links, &dependant, compile)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use camino::Utf8PathBuf;
    use tempfile::tempdir;

    use crate::node::Node;

    use super::*;

    #[test]
    fn test_partial_build_typst() -> Result<(), Box<dyn std::error::Error>> {
        let tempdir = tempdir()?;
        let root = Utf8PathBuf::try_from(tempdir.path().to_path_buf())?.canonicalize_utf8()?;

        let config = Config::default();
        let mut nodes = node::Db {
            nodes: vec![
                Node {
                    id: "id1".into(),
                    path: root.join("vector.typ"),
                    kind: node::NodeKind::File,
                    title: "Vector".into(),
                    names: vec!["vector".into()],
                    tags: vec![],
                    private: false,
                },
                Node {
                    id: "id2".into(),
                    path: root.join("gem.typ"),
                    kind: node::NodeKind::File,
                    title: "Gaussian Elimination".into(),
                    names: vec!["gem".into()],
                    tags: vec![],
                    private: false,
                },
            ],
        };

        let mut links = link::Db {
            links: vec![
                link::Link {
                    from: "id1".into(),
                    to: link::To::Ghost(link::FilePart::Name("matrix".into())),
                    location: None,
                    alias: None,
                },
                link::Link {
                    from: "id2".into(),
                    to: link::To::Ghost(link::FilePart::Name("matrix".into())),
                    location: None,
                    alias: None,
                },
            ],
        };

        let contents = r#"
        #metadata((
            title: "Vector",
            tags: (),
            names: ("vector",),
            private: false
        )) <omni-frontmatter>

        #metadata((
            content: "matrix",
            to: "id3",
            ghost: false,
        )) <omni-link>
        "#;

        std::fs::write(root.join("vector.typ"), contents)?;

        let contents = r#"
        #metadata((
            title: "Gaussian Elimination",
            tags: (),
            names: ("gem",),
            private: false
        )) <omni-frontmatter>

        #metadata((
            content: "matrix",
            to: "id3",
            ghost: false,
        )) <omni-link>
        "#;

        std::fs::write(root.join("gem.typ"), contents)?;

        let contents = r#"
        #metadata((
            title: "Matrix",
            tags: ("linalg", "matrix", "linear"),
            names: ("matrix", "matrices"),
            private: false
        )) <omni-frontmatter>
        
        = Top
        == Mid
        === Bottom
        "#;

        std::fs::write(root.join("matrix.typ"), contents)?;

        let file = node::File {
            id: "id3".into(),
            path: "matrix.typ".into(),
        };

        std::fs::create_dir(root.join("build"))?;

        // We are not interested in compilation in this example
        partial(&root, &config, &mut nodes, &mut links, &file, false)?;

        assert_eq!(
            nodes.nodes,
            vec![
                Node {
                    id: "id1".into(),
                    path: root.join("vector.typ"),
                    kind: node::NodeKind::File,
                    title: "Vector".into(),
                    names: vec!["vector".into()],
                    tags: vec![],
                    private: false,
                },
                Node {
                    id: "id2".into(),
                    path: root.join("gem.typ"),
                    kind: node::NodeKind::File,
                    title: "Gaussian Elimination".into(),
                    names: vec!["gem".into()],
                    tags: vec![],
                    private: false,
                },
                Node {
                    id: file.id,
                    path: root.join(file.path),
                    kind: node::NodeKind::File,
                    title: "Matrix".into(),
                    names: vec!["matrix".into(), "matrices".into()],
                    tags: vec!["linalg".into(), "matrix".into(), "linear".into()],
                    private: false,
                }
            ]
        );

        assert_eq!(
            links.links,
            vec![
                link::Link {
                    from: "id1".into(),
                    to: link::To::Id("id3".into()),
                    location: None,
                    alias: None
                },
                link::Link {
                    from: "id2".into(),
                    to: link::To::Id("id3".into()),
                    location: None,
                    alias: None
                },
            ],
        );

        Ok(())
    }
}
