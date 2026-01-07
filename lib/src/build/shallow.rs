use camino::Utf8Path;
use miette::Diagnostic;
use serde::Deserialize;
use thiserror::Error;

use crate::{
    build::{compile, shallow_typst},
    config::Config,
    format::typst,
    link, node,
};
use shallow_typst::shallow_typst;

#[derive(Debug, Error, Diagnostic)]
pub enum ShallowError {
    #[error("cannot shallow build (compile) a file with .{0} format")]
    #[diagnostic(help("file must be .typ, or TBD"))]
    InvalidFormat(String),

    #[error("cannot shallow build (compile) a file with no format")]
    #[diagnostic(help("file must be .typ, or TBD"))]
    NoFormat,

    #[error("cannot shallow build (compile) a file with no frontmatter")]
    #[diagnostic(help(
        "there's probably something wrong with your /resources/typst/lib/omni.typ, as that should generate a frontmatter"
    ))]
    MissingFrontmatter,

    #[error(transparent)]
    CompileError(#[from] compile::CompileError),

    #[error(transparent)]
    TypstQueryError(#[from] typst::QueryError),

    #[error(transparent)]
    TypstCompileError(#[from] typst::CompileError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Deserialize)]
pub(super) struct Frontmatter {
    pub(super) title: String,
    pub(super) tags: Vec<String>,
    pub(super) names: Vec<String>,
    pub(super) private: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct Link {
    pub(super) content: String,
    pub(super) to: String,
    #[serde(default)]
    pub(super) ghost: bool,
}

/// between shallow builds you should also save nodes.toml and links.toml
pub fn shallow(
    root: impl AsRef<Utf8Path>,
    config: &Config,
    nodes: &mut node::Db,
    links: &mut link::Db,
    file: &node::File,
    compile: bool,
) -> Result<(), ShallowError> {
    // figure out the file format (for now accept only typst) and reject invalid formats
    let my_path_canon = root.as_ref().join(&file.path).canonicalize_utf8()?;
    let extension = file.path.extension().ok_or(ShallowError::NoFormat)?;

    if extension == "typ" {
        shallow_typst(root, &my_path_canon, config, nodes, links, file, compile)?;
    } else {
        return Err(ShallowError::InvalidFormat(extension.to_string()));
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
    fn test_shallow_build_typst() -> Result<(), Box<dyn std::error::Error>> {
        let tempdir = tempdir()?;
        let root = Utf8PathBuf::try_from(tempdir.path().to_path_buf())?.canonicalize_utf8()?;

        let config = Config::default();
        let mut nodes = node::Db {
            nodes: vec![Node {
                id: "id1".into(),
                path: root.join("vector.typ"),
                kind: node::NodeKind::File,
                title: "Vector".into(),
                names: vec!["vector".into()],
                tags: vec![],
                private: false,
            }],
        };

        let mut links = link::Db {
            links: vec![link::Link {
                from: "id2".into(),
                to: link::To::Id("id4555".into()),
                location: None,
                alias: None,
            }],
        };

        // we want to shallow build a new matrix.typ file

        let contents = r#"
        #metadata((
            title: "Matrix",
            tags: ("linalg", "matrix", "linear"),
            names: ("matrix", "matrices"),
            private: false
        )) <omni-frontmatter>
        
        #metadata((
            content: "vector",
            to: "id1",
            ghost: false,
        )) <omni-link>

        #metadata((
            content: "singular matrix",
            to: "singularity",
            ghost: true,
        )) <omni-link>

        = Top
        == Mid
        === Bottom
        "#;
        std::fs::write(root.join("matrix.typ"), contents)?;

        let file = node::File {
            id: "id2".into(),
            path: "matrix.typ".into(),
        };

        shallow(&root, &config, &mut nodes, &mut links, &file, true)?;

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
                    from: "id2".into(),
                    to: link::To::Id("id1".into()),
                    location: None,
                    alias: None
                },
                link::Link {
                    from: "id2".into(),
                    to: link::To::Ghost(link::FilePart::Name("singularity".into())),
                    location: None,
                    alias: None
                }
            ],
        );

        assert!(std::fs::exists(root.join("build/matrix.html"))?);
        assert!(std::fs::exists(root.join("build/matrix.pdf"))?);

        Ok(())
    }

    #[test]
    fn test_shallow_build_format_fail() -> Result<(), Box<dyn std::error::Error>> {
        let tempdir = tempdir()?;
        let root = Utf8PathBuf::try_from(tempdir.path().to_path_buf())?.canonicalize_utf8()?;

        let config = Config::default();
        let mut nodes = node::Db {
            nodes: vec![Node {
                id: "id1".into(),
                path: root.join("vector.typ"),
                kind: node::NodeKind::File,
                title: "Vector".into(),
                names: vec!["vector".into()],
                tags: vec![],
                private: false,
            }],
        };
        let mut links = link::Db { links: vec![] };

        std::fs::write(root.join("matrix"), "")?;
        let file = node::File {
            id: "id2".into(),
            path: "matrix".into(),
        };

        assert_eq!(
            shallow(&root, &config, &mut nodes, &mut links, &file, false)
                .err()
                .unwrap()
                .to_string(),
            ShallowError::NoFormat.to_string()
        );

        std::fs::write(root.join("matrix.CRAZYFORMAT"), "")?;
        let file = node::File {
            id: "id2".into(),
            path: "matrix.CRAZYFORMAT".into(),
        };

        assert_eq!(
            shallow(&root, &config, &mut nodes, &mut links, &file, false)
                .err()
                .unwrap()
                .to_string(),
            ShallowError::InvalidFormat("CRAZYFORMAT".into()).to_string()
        );

        Ok(())
    }
}
