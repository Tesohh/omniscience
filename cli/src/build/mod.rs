use camino::Utf8Path;
use omni::{
    build::{compile::compile, partial::partial},
    config::Config,
    link, node,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{args::BuildCommand, pretty};

#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    TomlDeserializeError(#[from] toml::de::Error),

    #[error(transparent)]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error(transparent)]
    NodeError(#[from] node::Error),

    #[error(transparent)]
    CompileError(#[from] omni::build::compile::CompileError),

    #[error(transparent)]
    PartialBuildError(#[from] omni::build::partial::PartialError),

    #[error(transparent)]
    ShallowBuildError(#[from] omni::build::shallow::ShallowError),
    // #[error("path given has no parent")]
    // #[diagnostic(help("might be root or empty?"))]
    // NoParent,
    //
    // #[error("path given is outside project root")]
    // OutsideRoot,
    //
    // #[error("path given does not exist while in raw mode")]
    // DirNotExistsInRawMode,
    //
    // #[error("template `{0}` not found")]
    // TemplateNotFound(String),
    //
    // #[error("a file at that location already exists")]
    // AlreadyExists,
}

pub fn build(
    root: impl AsRef<Utf8Path>,
    config: &Config,
    cmd: BuildCommand,
) -> miette::Result<(), Error> {
    let user_db: node::UserDb = {
        let db_file = std::fs::read(root.as_ref().join("nodes.toml"))?;
        toml::from_slice(&db_file)?
    };
    let mut nodes: node::Db = {
        let db_file = std::fs::read(root.as_ref().join("build/nodes.toml"))?;
        toml::from_slice(&db_file)?
    };
    let mut links: link::Db = {
        let db_file = std::fs::read(root.as_ref().join("build/links.toml"))?;
        toml::from_slice(&db_file)?
    };

    match cmd.path {
        Some(path) => {
            let path_canonical = path.canonicalize_utf8()?;
            let file = user_db
                .files
                .iter()
                .filter_map(|f| match f.path.canonicalize_utf8() {
                    Ok(p) => Some((f, p)),
                    Err(err) => {
                        pretty::warning(format!(
                            "invalid path found in nodes.toml for id {}. error: {}",
                            f.id, err
                        ));
                        None
                    }
                })
                .find(|file| file.1 == path_canonical)
                .map(|(f, _)| f)
                .ok_or(node::Error::UntrackedNode(path))?;

            partial(&root, config, &mut nodes, &mut links, file, true)?;
        }
        None => {
            for file in &user_db.files {
                pretty::msg("partial", &file.path);
                partial(&root, config, &mut nodes, &mut links, file, false)?
            }

            let root_as_ref = root.as_ref();
            user_db.files.par_iter().try_for_each(|file| {
                pretty::msg("compile", &file.path);
                compile(root_as_ref, &file.path, config)
            })?;
        }
    };

    // SAVEPOINT(nodes, links)
    let new_nodes_toml = toml::to_string(&nodes)?;
    std::fs::write(root.as_ref().join("build/nodes.toml"), new_nodes_toml)?;

    let new_links_toml = toml::to_string(&links)?;
    std::fs::write(root.as_ref().join("build/links.toml"), new_links_toml)?;

    Ok(())
}
