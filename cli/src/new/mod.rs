use std::path::{self, Component, Path, PathBuf};

use omni::config::{Config, OmniPathError};

use crate::{args::NewCommand, pretty};

#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum Error {
    // #[error("template error")]
    // TemplateError(#[from] tera::Error),
    //
    // #[error("io error: {0}")]
    // IoError(#[from] std::io::Error),
    //
    #[error("omni path error:")]
    OmniPathError(#[from] OmniPathError),
}

pub fn new(config: &Config, cmd: NewCommand) -> miette::Result<(), Error> {
    let target = config.parse_omni_path(cmd.path)?;
    Ok(())
}
