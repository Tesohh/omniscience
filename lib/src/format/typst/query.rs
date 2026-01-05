use crate::format::typst::Format;
use std::process::{Command, Stdio};

use camino::Utf8Path;
use miette::Diagnostic;
use serde::{Deserialize, de::DeserializeOwned};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum QueryError {
    #[error("missing typst executable")]
    MissingTypst,

    #[error("`typst query` exited with code {0}")]
    TypstErrorCode(i32),

    #[error("`typst query` error (code {0}): {1}")]
    TypstError(i32, String),

    #[error("io error")]
    IoError(#[from] std::io::Error),

    #[error("deserialization error")]
    DeserializationError(#[from] serde_json::Error),
}

#[derive(Default)]
pub struct QueryParams<'a> {
    pub format: Format,
    pub silent: bool,
    pub one: bool,
    pub field: Option<&'a str>,
}

/// Runs `typst query` on `target` and deserializes json output as `T`.
pub fn query<T>(
    root: impl AsRef<Utf8Path>,
    target: impl AsRef<Utf8Path>,
    selector: &str,
    params: &QueryParams,
) -> Result<T, QueryError>
where
    T: DeserializeOwned,
{
    let root = root.as_ref();
    let target = target.as_ref();

    let mut command = Command::new("typst");
    command
        .current_dir(root)
        .arg("query")
        .arg(target)
        .arg(selector)
        .arg("--root")
        .arg(root)
        .arg("--format")
        .arg("json");

    if !params.silent {
        command.stderr(Stdio::inherit());
    }

    if params.one {
        command.arg("--one");
    }

    if let Some(field) = params.field {
        command.arg("--field").arg(field);
    }

    match params.format {
        Format::Pdf => command.args(["--target", "paged"]),
        Format::Html => command.args(["--target", "html", "--features", "html"]),
    };

    let output = match command.output() {
        Ok(o) => o,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(QueryError::MissingTypst);
        }
        Err(err) => return Err(err.into()),
    };

    if !output.status.success() {
        if params.silent {
            return Err(QueryError::TypstError(
                output.status.code().unwrap_or_default(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        } else {
            return Err(QueryError::TypstErrorCode(
                output.status.code().unwrap_or_default(),
            ));
        }
    }

    let value: T = serde_json::from_slice(output.stdout.as_slice())?;

    Ok(value)
}

#[cfg(test)]
mod tests {
    use camino::Utf8PathBuf;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_typst_query_one() -> Result<(), Box<dyn std::error::Error>> {
        let tempdir = tempdir()?;
        let root = Utf8PathBuf::try_from(tempdir.path().to_path_buf())?;

        let contents = r#"
        #metadata("hello") <frontmatter>
        "#;
        std::fs::write(root.join("note.typ"), contents)?;

        let output = query::<String>(
            &root,
            root.join("note.typ"),
            "<frontmatter>",
            &QueryParams {
                one: true,
                field: Some("value"),
                ..Default::default()
            },
        )?;

        assert_eq!(output, "hello");

        Ok(())
    }

    #[test]
    fn test_typst_query_many() -> Result<(), Box<dyn std::error::Error>> {
        let tempdir = tempdir()?;
        let root = Utf8PathBuf::try_from(tempdir.path().to_path_buf())?;

        let contents = r#"
        #metadata("id1") <omni-link>
        #metadata("id2") <omni-link>
        #metadata("id3") <omni-link>

        = Top
        == Mid
        === Bottom
        "#;
        std::fs::write(root.join("note.typ"), contents)?;

        let output = query::<Vec<String>>(
            &root,
            root.join("note.typ"),
            "<omni-link>",
            &QueryParams {
                format: Format::Pdf,
                silent: false,
                one: false,
                field: Some("value"),
            },
        )?;

        assert_eq!(output, ["id1", "id2", "id3"]);

        let output = query::<Vec<String>>(
            &root,
            root.join("note.typ"),
            "<omni-link-gibberishaiohsdaiohd>",
            &QueryParams {
                format: Format::Pdf,
                silent: false,
                one: false,
                field: Some("value"),
            },
        )?;

        assert_eq!(output, Vec::<String>::new());

        let output = query::<Vec<String>>(
            &root,
            root.join("note.typ"),
            "<omni-link-gibberishaiohsdaiohd>",
            &QueryParams {
                format: Format::Pdf,
                silent: false,
                one: false,
                field: Some("value"),
            },
        )?;

        assert_eq!(output, Vec::<String>::new());

        Ok(())
    }
}
