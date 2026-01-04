use crate::args::TrackCommand;
use camino::{Utf8Path, Utf8PathBuf};
use omni::{
    config::Config,
    node::{self, UserDb},
    omni_path::{self},
};

#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum Error {
    #[error("template error")]
    TemplateError(#[from] tera::Error),

    #[error("io error")]
    IoError(#[from] std::io::Error),

    #[error("omni path error")]
    OmniPathError(#[from] omni_path::Error),

    #[error("toml deserialization error")]
    TomlDeserializeError(#[from] toml::de::Error),

    #[error("toml serialization error")]
    TomlSerializeError(#[from] toml::ser::Error),
    //
    // #[error("path given has no parent")]
    // #[diagnostic(help("might be root or empty?"))]
    // NoParent,
    //
    #[error("path given is outside project root")]
    OutsideRoot,

    #[error("file at {0} not found")]
    #[diagnostic(help("maybe create it first with `omni new`?"))]
    FileNotFound(Utf8PathBuf),

    #[error("{0} is a directory")]
    IsDirectory(Utf8PathBuf),
}

pub fn track(
    root: impl AsRef<Utf8Path>,
    config: &Config,
    cmd: TrackCommand,
) -> miette::Result<(), Error> {
    let root = root.as_ref();

    // verify that target is in the root/<prefix_dir?>
    let target: Utf8PathBuf = {
        let path_canonical = cmd.path.canonicalize()?;

        let mut src = root.canonicalize()?;
        if let Some(prefix_dir) = &config.project.prefix_dir {
            src = src.join(prefix_dir)
        }

        if !path_canonical.starts_with(src) {
            return Err(Error::OutsideRoot);
        }

        cmd.path
    };

    // check that target exists and is actually a file
    match std::fs::metadata(&target) {
        Ok(metadata) => {
            if metadata.is_dir() {
                return Err(Error::IsDirectory(target));
            }
        }
        // Ok(false) => return ,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(Error::FileNotFound(target));
        }
        Err(err) => return Err(err.into()),
    };

    just_track(root, target)
}

/// core logic of track, without all input validation.
/// assumes target is a file and exists
/// also assumes that target is NOT already tracked.
pub fn just_track(root: impl AsRef<Utf8Path>, target: impl AsRef<Utf8Path>) -> Result<(), Error> {
    let target = target.as_ref().to_path_buf();

    let db_path = root.as_ref().join("nodes.toml");
    let db_file = std::fs::read(&db_path)?;

    let mut db: UserDb = toml::from_slice(&db_file)?;
    let file_node = node::File {
        id: node::Id::new(),
        path: target.clone(),
    };
    db.files.push(file_node.clone());

    let new_toml = toml::to_string(&db)?;
    std::fs::write(db_path, new_toml)?;

    Ok(())
}
