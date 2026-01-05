use crate::format::typst::Format;
use std::process::{Command, Stdio};

use camino::Utf8Path;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum CompileError {
    #[error("missing typst executable")]
    MissingTypst,

    #[error("`typst compile` exited with code {0}")]
    TypstError(i32),

    #[error("io error")]
    IoError(#[from] std::io::Error),
}

/// Runs `typst compile` on `target`
pub fn compile(
    root: impl AsRef<Utf8Path>,
    target: impl AsRef<Utf8Path>,
    output: impl AsRef<Utf8Path>,
    format: Format,
    silent: bool,
) -> Result<(), CompileError> {
    let root = root.as_ref();
    let target = target.as_ref();
    let output = output.as_ref();

    let mut command = Command::new("typst");
    command
        .current_dir(root)
        .arg("compile")
        .arg(target)
        .arg(output)
        .arg("--root")
        .arg(root);

    if !silent {
        command.stderr(Stdio::inherit());
    }

    match format {
        Format::Pdf => command.args(["--format", "pdf"]),
        Format::Html => command.args(["--format", "html", "--features", "html"]),
    };

    let output = match command.output() {
        Ok(o) => o,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(CompileError::MissingTypst);
        }
        Err(err) => return Err(err.into()),
    };

    if !output.status.success() {
        return Err(CompileError::TypstError(
            output.status.code().unwrap_or_default(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use camino::Utf8PathBuf;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_typst_compile() -> Result<(), Box<dyn std::error::Error>> {
        let tempdir = tempdir()?;
        let root = Utf8PathBuf::try_from(tempdir.path().to_path_buf())?;

        let contents = r#"= Hello world"#;
        std::fs::write(root.join("note.typ"), contents)?;

        compile(
            &root,
            root.join("note.typ"),
            root.join("note.pdf"),
            Format::Pdf,
            false,
        )?;

        assert!(std::fs::exists(root.join("note.pdf"))?);

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_typst_compile_fail() {
        let tempdir = tempdir().unwrap();
        let root = Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap();

        let contents = r#"* bad input"#;
        std::fs::write(root.join("note.typ"), contents).unwrap();

        compile(
            &root,
            root.join("note.typ"),
            root.join("note.pdf"),
            Format::Pdf,
            true,
        )
        .unwrap();
    }
}
