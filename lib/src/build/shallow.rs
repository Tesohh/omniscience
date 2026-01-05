use camino::Utf8Path;
use miette::Diagnostic;
use serde::Deserialize;
use thiserror::Error;

use crate::{
    config::Config,
    format::typst::{self, QueryParams},
    link, node,
};

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
    TypstQueryError(#[from] typst::QueryError),
}

#[derive(Debug, Deserialize)]
struct Frontmatter {
    title: String,
    tags: Vec<String>,
    names: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Link {
    content: String,
    to: String,
    #[serde(default)]
    ghost: bool,
}

/// between shallow builds you should also save nodes.toml and links.toml
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
        // query the file to ask for omni-frontmatter
        let frontmatter: Frontmatter = typst::query(
            &root,
            &file.path,
            "<omni-frontmatter>",
            &QueryParams {
                format: typst::Format::Html,
                silent: true,
                one: true,
                field: Some("value"),
            },
        )
        .map_err(|err| match err {
            typst::QueryError::TypstError(_, ref message)
                if message == "error: expected exactly one element, found 0\n" =>
            {
                ShallowError::MissingFrontmatter
            }
            _ => err.into(),
        })?;

        // query the file to ask for omni-links
        let links: Vec<Link> = typst::query(
            &root,
            &file.path,
            "<omni-link>",
            &QueryParams {
                format: typst::Format::Html,
                silent: true,
                one: false,
                field: Some("value"),
            },
        )?;

        // compile to html and pdf
    } else {
        return Err(ShallowError::InvalidFormat(extension.to_string()));
    }

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

        // we want to shallow build a new matrix.typ file

        let contents = r#"
        #metadata((
            title: "Matrix",
            tags: ("linalg", "matrix", "linear"),
            names: ("matrix", "matrices")
        )) <omni-frontmatter>
        #metadata(()) <omni-link>
        #metadata("id2") <omni-link>
        #metadata("id3") <omni-link>

        = Top
        == Mid
        === Bottom
        "#;
        std::fs::write(root.join("matrix.typ"), contents)?;

        let file = node::File {
            id: "id2".into(),
            path: "matrix.typ".into(),
        };

        shallow(&root, &config, &mut nodes, &mut links, &file)?;
        panic!();

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
                .unwrap()
                .to_string(),
            ShallowError::NoFormat.to_string()
        );

        let file = node::File {
            id: "id2".into(),
            path: "matrix.CRAZYFORMAT".into(),
        };

        assert_eq!(
            shallow(&root, &config, &mut nodes, &mut links, &file)
                .err()
                .unwrap()
                .to_string(),
            ShallowError::InvalidFormat("CRAZYFORMAT".into()).to_string()
        );

        Ok(())
    }
}
