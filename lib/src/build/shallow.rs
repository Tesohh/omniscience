use camino::Utf8Path;
use miette::Diagnostic;
use thiserror::Error;

use crate::{config::Config, link, node};

#[derive(Debug, Error, Diagnostic, PartialEq)]
pub enum ShallowError {
    #[error("cannot shallow build (compile) a file with .{0} format")]
    #[diagnostic(help("file must be .typ, or TBD"))]
    InvalidFormat(String),

    #[error("cannot shallow build (compile) a file with no format")]
    #[diagnostic(help("file must be .typ, or TBD"))]
    NoFormat,
}

pub fn shallow(
    root: impl AsRef<Utf8Path>,
    config: &Config,
    nodes: &mut node::Db,
    links: &mut link::Db,
    file: &node::File,
) -> Result<(), ShallowError> {
    // figure out the file format (for now accept only typst) and reject invalid formats
    let extension = file.path.extension().ok_or(ShallowError::NoFormat)?;

    if extension == "typ" {
    } else {
        return Err(ShallowError::InvalidFormat(extension.to_string()));
    }

    // for typst:
    // - query the file to ask for omni-links
    // - query the file to ask for omni-frontmatter
    // - compile to html and pdf

    // update nodes
    // update links

    Ok(())
}

#[cfg(test)]
mod tests {

    use camino::Utf8PathBuf;
    use tempfile::tempdir;

    use crate::node::Node;

    use super::*;

    #[test]
    fn test_shallow_build_typst() -> Result<(), Box<dyn std::error::Error>> {
        let tempdir = tempdir()?;
        let root = Utf8PathBuf::try_from(tempdir.path().to_path_buf())?;

        let config = Config::default();
        let nodes = node::Db {
            nodes: vec![Node {
                id: "id1".into(),
                path: "vector.typ".into(),
                kind: node::NodeKind::File,
                title: "Vector".into(),
                names: vec!["vector".into()],
                tags: vec![],
            }],
        };

        let links = link::Db { links: vec![] };

        // we want to shallow build a new matrix.typ file

        Ok(())
    }

    #[test]
    fn test_shallow_build_format_fail() -> Result<(), Box<dyn std::error::Error>> {
        let tempdir = tempdir()?;
        let root = Utf8PathBuf::try_from(tempdir.path().to_path_buf())?;

        let config = Config::default();
        let mut nodes = node::Db {
            nodes: vec![Node {
                id: "id1".into(),
                path: "vector.typ".into(),
                kind: node::NodeKind::File,
                title: "Vector".into(),
                names: vec!["vector".into()],
                tags: vec![],
            }],
        };
        let mut links = link::Db { links: vec![] };

        let file = node::File {
            id: "id2".into(),
            path: "matrix".into(),
        };

        assert_eq!(
            shallow(&root, &config, &mut nodes, &mut links, &file)
                .err()
                .unwrap(),
            ShallowError::NoFormat
        );

        let file = node::File {
            id: "id2".into(),
            path: "matrix.CRAZYFORMAT".into(),
        };

        assert_eq!(
            shallow(&root, &config, &mut nodes, &mut links, &file)
                .err()
                .unwrap(),
            ShallowError::InvalidFormat("CRAZYFORMAT".into())
        );

        Ok(())
    }
}
