use std::path::Component;

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

    #[error("invalid path")]
    #[diagnostic(help(
        "path must start with `src` (eg. `src/linear-algebra/matrix.typ`) 
or be a omni style path (starts from src, can have aliases, no file extension eg. `linalg/matrix`)"
    ))]
    InvalidPath,
}

pub fn new(config: &Config, cmd: NewCommand) -> miette::Result<(), Error> {
    let mut path_components = cmd.path.components();
    let first = path_components.next().ok_or(Error::PathEmpty)?;

    // TODO: allow for absolute, relative, whatever paths AS LONG as they stay in the project
    // so we'd have
    // /Users/me/docs/vault/src/linear-algebra/matrix.typ (os ABSOLUTE)
    // ./src/linear-algebra/matrix.typ (os relative)
    // src/linear-algebra/matrix.typ (still os relative)
    // linear-algebra/matrix.typ (omni style)
    // linalg/matrix.typ (omni style (aliased))

    let target = match first {
        Component::Normal(first_str) if first_str == "src" => pretty::debug("using full path"),
        Component::Normal(first_str) => pretty::debug("using omni style"),
        _ => return Err(Error::InvalidPath),
    };

    Ok(())
}
