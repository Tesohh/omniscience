use std::path::{Path, PathBuf};

use omni::{
    config::Config,
    omni_path::{self, OmniPath},
};

use crate::args::NewCommand;

#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum Error {
    // #[error("template error")]
    // TemplateError(#[from] tera::Error),
    //
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("omni path error: {0}")]
    OmniPathError(#[from] omni_path::Error),

    #[error("path given has no parent")]
    #[diagnostic(help("might be root or empty?"))]
    NoParent,

    #[error("path given is outside project root")]
    OutsideRoot,
}

// TODO: need preoject root
pub fn new(root: impl AsRef<Path>, config: &Config, cmd: NewCommand) -> miette::Result<(), Error> {
    let root = root.as_ref();

    let target: PathBuf = if cmd.raw {
        let parent = cmd.path.parent().ok_or(Error::NoParent)?.canonicalize()?;
        if !parent.starts_with(root.canonicalize()?) {
            return Err(Error::OutsideRoot);
        }

        cmd.path
    } else {
        OmniPath::try_from_path(cmd.path)?
            .unalias(config)?
            .try_into()?
    };

    dbg!(target);

    // get the target:
    // if cmd.raw ==> target = cmd.path
    // else ==> target = OmniPath(cmd.path).unalias().pathize()

    // if cmd.raw and target.parent() is not a subdir of project_root ==> ERROR

    // if cmd.raw and target.parent() does not exist ==> ERROR
    // else if !cmd.raw mkdirall if needed

    // get the template
    // template not found ==> ERROR

    // create the file

    // apply template

    // track file

    Ok(())
}
