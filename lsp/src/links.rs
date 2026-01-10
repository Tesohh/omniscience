use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use omni::{config::Config, node, omni_path::OmniPath};
use thiserror::Error;

use crate::err_log_ext::ErrLogExt;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct LinkEntry {
    pub omni_path: OmniPath,
    pub true_path: Utf8PathBuf,
}

#[derive(Error, Debug)]
pub enum GetPossibleLinksError {
    #[error(transparent)]
    DedupError(#[from] DedupError),
}

pub fn get_possible_links(
    root: impl AsRef<Utf8Path>,
    config: &Config,
    nodes: &node::Db,
) -> Result<Vec<LinkEntry>, GetPossibleLinksError> {
    let mut links: Vec<LinkEntry> = vec![];

    // build linkentries based on the nodes db
    for node in &nodes.nodes {
        for name in &node.names {
            let omni_path = OmniPath::new(vec![], name.to_string()).force_unalias();
            links.push(LinkEntry {
                omni_path,
                true_path: node.path.clone(), // PERF:
            })
        }
    }

    // sort so that chunk_by works
    links.sort_by_key(|link| link.omni_path.clone());

    // grind out dedups
    loop {
        if !dedup(&root, config, &mut links)? {
            break;
        }
    }

    // try applying aliases
    for link in &mut links {
        for (from, to) in &config.dir_aliases {
            let done = link.omni_path.try_realias(from, to);
            if done {
                break;
            }
        }
    }

    Ok(links)
}

#[derive(Debug, Error)]
pub enum DedupError {
    #[error("unable to strip prefix, as the path of the node might be outside the project")]
    NodeOutsideProject,
    #[error("cannot go any further while deduplicating link for node")]
    CannotGoFurther,
}

fn dedup(
    root: impl AsRef<Utf8Path>,
    config: &Config,
    links: &mut [LinkEntry],
) -> Result<bool, DedupError> {
    let groups = links.chunk_by_mut(|a, b| a.omni_path == b.omni_path);

    let mut edited = false;

    for group in groups {
        if group.len() == 1 {
            continue;
        }

        for link in group {
            let mut prefix = root.as_ref().to_path_buf();
            if let Some(notes_prefix) = &config.project.prefix_dir {
                prefix.push(notes_prefix);
            }

            let path = link
                .true_path
                .strip_prefix(&prefix)
                .map_err(|_| DedupError::NodeOutsideProject)?;

            let components = path.components().collect_vec();
            let new_component = components
                .iter()
                .take(components.len() - 1)
                .nth(link.omni_path.path.len())
                .ok_or(DedupError::CannotGoFurther)?;

            link.omni_path.path.push(new_component.to_string());
            edited = true;
        }
    }

    Ok(edited)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_get_possible_links() {
        let nodes = node::Db {
            nodes: vec![
                node::Node {
                    id: "id1".into(),
                    path: "/Users/me/docs/vault/cs/linear-algebra/vector.typ".into(),
                    kind: node::NodeKind::File,
                    title: "Vector".into(),
                    names: vec!["vector".into()],
                    tags: vec![],
                    private: false,
                },
                node::Node {
                    id: "id2".into(),
                    path: "/Users/me/docs/vault/cs/linear-algebra/matrix.typ".into(),
                    kind: node::NodeKind::File,
                    title: "Matrix".into(),
                    names: vec!["matrix".into()],
                    tags: vec![],
                    private: false,
                },
                node::Node {
                    id: "id3".into(),
                    path: "/Users/me/docs/vault/cs/rust/vector.typ".into(),
                    kind: node::NodeKind::File,
                    title: "Vector".into(),
                    names: vec!["vector".into()],
                    tags: vec![],
                    private: false,
                },
            ],
        };

        let root = "/Users/me/docs/vault";

        let config = Config {
            dir_aliases: HashMap::from([("linalg".into(), "cs/linear-algebra".into())]),
            ..Default::default()
        };

        assert_eq!(
            get_possible_links(root, &config, &nodes).unwrap(),
            [
                LinkEntry {
                    omni_path: OmniPath::new(vec![], "matrix".into()).force_unalias(),
                    true_path: "/Users/me/docs/vault/cs/linear-algebra/matrix.typ".into()
                },
                LinkEntry {
                    omni_path: OmniPath::new(vec!["linalg".into()], "vector".into()),
                    true_path: "/Users/me/docs/vault/cs/linear-algebra/vector.typ".into()
                },
                LinkEntry {
                    omni_path: OmniPath::new(vec!["cs".into(), "rust".into()], "vector".into())
                        .force_unalias(),
                    true_path: "/Users/me/docs/vault/cs/rust/vector.typ".into()
                },
            ]
        );
    }
}
