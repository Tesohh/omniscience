use std::{
    ffi::OsString,
    path::{Component, Path, PathBuf},
};

use omni::config::Config;

use crate::{args::NewCommand, pretty};

#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum Error {
    // #[error("template error")]
    // TemplateError(#[from] tera::Error),
    //
    // #[error("io error:")]
    // IoError(#[from] std::io::Error),
    #[error("path is empty")]
    PathEmpty,
    //     #[error("invalid path")]
    //     #[diagnostic(help(
    //         "path must start with `src` (eg. `src/linear-algebra/matrix.typ`)
    // or be a omni style path (starts from src, can have aliases, no file extension eg. `linalg/matrix`)"
    //     ))]
    //     InvalidPath,
    #[error("path contains invalid unicode")]
    InvalidUnicode,
}

pub fn new(config: &Config, cmd: NewCommand) -> miette::Result<(), Error> {
    let mut path_components = cmd.path.components();

    let mut target = PathBuf::new();

    // parse potential alias
    if let Component::Normal(first) = path_components.next().ok_or(Error::PathEmpty)? {
        let first = first.to_str().ok_or(Error::InvalidUnicode)?;
        match config.dir_aliases.get(first) {
            Some(first) => target.push(PathBuf::from("src/").join(first)),
            None => target.push(first),
        };
    };

    // add all other components
    for component in path_components {
        target.push(component);
    }

    dbg!(target);

    Ok(())
}
