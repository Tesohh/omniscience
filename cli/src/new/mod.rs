use omni::config::Config;

use crate::args::NewCommand;

#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum Error {
    // #[error("template error")]
    // TemplateError(#[from] tera::Error),
    //
    // #[error("io error: {0}")]
    // IoError(#[from] std::io::Error),
    //
}

// TODO: need preoject root
pub fn new(config: &Config, cmd: NewCommand) -> miette::Result<(), Error> {
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
