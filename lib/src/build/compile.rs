use camino::Utf8Path;
use miette::Diagnostic;
use thiserror::Error;

use crate::{
    config::{self, Config},
    format::{src_to_build_path, typst},
};

#[derive(Debug, Error, Diagnostic)]
pub enum CompileError {
    #[error("cannot shallow build (compile) a file with .{0} format")]
    #[diagnostic(help("file must be .typ, or TBD"))]
    InvalidFormat(String),

    #[error("cannot shallow build (compile) a file with no format")]
    #[diagnostic(help("file must be .typ, or TBD"))]
    NoFormat,

    #[error(transparent)]
    TypstCompileError(#[from] typst::CompileError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub fn compile(
    root: impl AsRef<Utf8Path>,
    path: impl AsRef<Utf8Path>,
    config: &Config,
) -> Result<(), CompileError> {
    let my_path_canon = root.as_ref().join(&path).canonicalize_utf8()?;
    let extension = my_path_canon.extension().ok_or(CompileError::NoFormat)?;

    if extension == "typ" {
        compile_typst(root, &my_path_canon, config)
    } else {
        Err(CompileError::InvalidFormat(extension.to_string()))
    }
}

pub fn compile_typst(
    root: impl AsRef<Utf8Path>,
    path: impl AsRef<Utf8Path>,
    config: &Config,
) -> Result<(), CompileError> {
    let out_html = src_to_build_path(&root, &path, "html").expect("both paths should be canonical");

    let mut out_pdf = out_html.clone();
    out_pdf.set_extension("pdf");

    if let Some(parent) = out_html.parent()
        && !std::fs::exists(parent)?
    {
        std::fs::create_dir_all(parent)?;
    }

    match config.typst.output_format {
        config::TypstOutputFormat::Html => {
            typst::compile(&root, &path, out_html, typst::Format::Html, true)?;
        }
        config::TypstOutputFormat::Pdf => {
            typst::compile(&root, &path, out_pdf, typst::Format::Pdf, true)?;
        }
        config::TypstOutputFormat::HtmlAndPdf => {
            typst::compile(&root, &path, out_html, typst::Format::Html, true)?;
            typst::compile(&root, &path, out_pdf, typst::Format::Pdf, true)?;
        }
    };

    Ok(())
}
