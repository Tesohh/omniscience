use camino::{Utf8Path, Utf8PathBuf};

use crate::node;

#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    TomlDeserializeError(#[from] toml::de::Error),

    #[error(transparent)]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error("path given is outside project root")]
    OutsideRoot,

    #[error("{0} is already tracked")]
    AlreadyTracked(Utf8PathBuf),
}

fn is_already_tracked(db: &node::UserDb, target: impl AsRef<Utf8Path>) -> Result<bool, Error> {
    let canonical_target = match target.as_ref().canonicalize_utf8() {
        Ok(v) => v,
        Err(_) => return Ok(false), // can't be already tracked if it doesnt exist
    };
    Ok(db
        .files
        .iter()
        .filter_map(|f| f.path.canonicalize_utf8().ok())
        .any(|path| path == canonical_target))
}

/// core logic of track, without all input validation.
/// assumes target is a file and exists
/// only check that it does is checking if the file is already tracked
pub fn track(
    root: impl AsRef<Utf8Path>,
    target: impl AsRef<Utf8Path>,
) -> Result<node::File, Error> {
    let target = target.as_ref().to_path_buf();

    let db_path = root.as_ref().join("nodes.toml");
    let db_file = std::fs::read(&db_path)?;

    let mut db: node::UserDb = toml::from_slice(&db_file)?;

    // Check that target it not already tracked
    if is_already_tracked(&db, &target)? {
        return Err(Error::AlreadyTracked(target));
    };

    let file_node = node::File {
        id: node::Id::new(&db),
        path: target.clone(),
    };
    db.files.push(file_node.clone());

    // SAVEPOINT(user_db)

    let new_toml = toml::to_string(&db)?;
    std::fs::write(db_path, new_toml)?;

    Ok(file_node)
}
