use camino::Utf8Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("template `{0}` not found")]
    TemplateNotFound(String),
}

pub fn get_template(
    root: impl AsRef<Utf8Path>,
    template_name: &str,
) -> Result<(String, String), Error> {
    // get the template
    let mut found_template_path = None;
    for file in std::fs::read_dir(root.as_ref().join("resources/templates"))? {
        let file = file?;
        let is_it_the_template = file
            .file_name()
            .to_string_lossy()
            .starts_with(template_name);
        if is_it_the_template {
            found_template_path = Some(file.path());
        }
    }

    // let template = std::fs::read(template_path);
    // template not found ==> ERROR
    // else add extension to target
    let (template, extension) = match found_template_path {
        Some(path) => (
            std::fs::read_to_string(&path)?,
            path.extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(), // PERF: remove the to's?
        ),
        None => return Err(Error::TemplateNotFound(template_name.to_string())),
    };

    Ok((template, extension))
}
