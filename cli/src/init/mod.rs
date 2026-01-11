use camino::Utf8PathBuf;
use owo_colors::OwoColorize;

use crate::{args::InitCommand, pretty};

#[derive(thiserror::Error, miette::Diagnostic, Debug)]
pub enum Error {
    #[error("template error")]
    TemplateError(#[from] tera::Error),

    #[error("io error:")]
    IoError(#[from] std::io::Error),

    #[error("directory `{0}` already exists")]
    DirAlreadyExists(Utf8PathBuf),
}

pub fn init(cmd: InitCommand) -> miette::Result<(), Error> {
    let path = match &cmd.path {
        Some(v) => v,
        None => &Utf8PathBuf::from(&cmd.name),
    };

    match std::fs::create_dir(path) {
        Ok(v) => Ok(v),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(Error::DirAlreadyExists(path.clone()))
        }
        Err(e) => Err(e.into()),
    }?;

    let tera = crate::tera::tera();
    let mut context = tera::Context::new();
    context.insert("name", &cmd.name);

    let omni_toml = std::fs::File::create(path.join("omni.toml"))?;
    tera.render_to("omni.toml", &context, omni_toml)?;

    let nodes_toml = std::fs::File::create(path.join("nodes.toml"))?;
    tera.render_to("nodes.toml", &context, nodes_toml)?;

    let gitignore = std::fs::File::create(path.join(".gitignore"))?;
    tera.render_to(".gitignore", &context, gitignore)?;

    std::fs::create_dir(path.join("assets"))?;
    std::fs::create_dir(path.join("build"))?;
    std::fs::create_dir(path.join("src"))?;

    std::fs::create_dir(path.join("resources"))?;
    std::fs::create_dir(path.join("resources/templates"))?;

    let omni_note_template = std::fs::File::create(path.join("resources/templates/note.typ"))?;
    tera.render_to("resources/templates/note.typ", &context, omni_note_template)?;

    std::fs::create_dir(path.join("resources/typst"))?;
    std::fs::create_dir(path.join("resources/typst/lib"))?;

    let omni_typ = std::fs::File::create(path.join("resources/typst/lib/omni.typ"))?;
    tera.render_to("resources/typst/lib/omni.typ", &context, omni_typ)?;

    std::fs::create_dir(path.join("resources/typst/templates"))?;

    let typst_note_template =
        std::fs::File::create(path.join("resources/typst/templates/note.typ"))?;
    tera.render_to(
        "resources/typst/templates/note.typ",
        &context,
        typst_note_template,
    )?;

    std::fs::write(path.join("build/root"), path.as_str())?;

    if let Some(path) = &cmd.path {
        pretty::msg(
            "init",
            format!("{} {}", cmd.name, format!("at {}", path).dimmed()),
        );
    } else {
        pretty::msg("init", &cmd.name);
    }

    /*
    - project
        - nodes.toml
        - omni.toml
        - resources/
            - typst/
                - lib/
                    - omni.typ <typst module>
                    - omniscience_typst.wasm <typst plugin>
                - templates/ <typst templates>
                    - note.typ
            - templates/ <omni templates>
                - note.typ // we also might use tera to pass user arbitrary arguments to templates. like "omni new note --title mytitle"
                - note_md.md
                - blog.md
                - todo.org
        - build/
            - nodes.toml
            - links.toml
            - <all compiled files>
        - src/
            - <all users content>
        - assets/
            - <users static assets>
    */

    pretty::info(
        format!(
            r#"project structure recap:
    {} is your project config
    {} describes where nodes are

    {} contains your static assets (eg. images, fonts, css etc.)

    {} contains your compiled content (should not be touched)

    {} contains your content 

    {} contains typst-specific resources
    {} contains typst-specific libraries and plugins (which shan't be touched)
    {} contains your custom templates
        "#,
            "`omni.toml`".yellow(),
            "`nodes.toml`".yellow(),
            "`assets`".yellow(),
            "`build`".yellow(),
            "`src`".yellow(),
            "`resources/typst`".yellow(),
            "`resources/typst/lib`".yellow(),
            "`resources/templates`".yellow(),
        )
        .trim(),
    );

    Ok(())
}
