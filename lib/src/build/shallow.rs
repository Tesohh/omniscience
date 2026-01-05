use miette::Diagnostic;
use thiserror::Error;

use crate::{config::Config, link, node};

#[derive(Debug, Error, Diagnostic)]
pub enum ShallowError {}

pub fn shallow(
    config: &Config,
    nodes: &mut node::Db,
    links: &mut link::Db,
    file: &node::File,
) -> Result<(), ShallowError> {
    // figure out the file format (for now accept only typst) and reject invalid formats
    // for typst:
    // - query the file to ask for omni-links
    // - query the file to ask for omni-frontmatter
    // - compile to html and pdf

    // update nodes
    // update links

    Ok(())
}
