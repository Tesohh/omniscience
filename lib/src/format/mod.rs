pub mod typst;

use camino::{Utf8Path, Utf8PathBuf};

/// Returns None if path is not a sub of root
/// root and path should both be canonical
pub fn src_to_build_path(
    root: impl AsRef<Utf8Path>,
    path: impl AsRef<Utf8Path>,
    extension: &str,
) -> Option<Utf8PathBuf> {
    let target = path.as_ref().strip_prefix(root.as_ref()).ok()?;
    let mut out = root.as_ref().join("build").join(target);
    out.set_extension(extension);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_src_to_build_path() {
        assert_eq!(
            src_to_build_path("/Users/me/vault", "/Users/me/vault/src/note.typ", "pdf"),
            Some("/Users/me/vault/build/src/note.pdf".into())
        );
    }
}
