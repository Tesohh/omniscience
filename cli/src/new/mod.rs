use std::io::Write;

use camino::{Utf8Path, Utf8PathBuf};
use omni::{
    config::Config,
    link, node,
    omni_path::{self, OmniPath},
};
use tera::Tera;

use crate::{args::NewCommand, pretty, track};

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

    #[error("track error")]
    TrackError(#[from] track::Error),

    #[error(transparent)]
    PartialBuildError(#[from] omni::build::partial::PartialError),

    #[error("path given has no parent")]
    #[diagnostic(help("might be root or empty?"))]
    NoParent,

    #[error("path given is outside project root")]
    OutsideRoot,

    #[error("path given does not exist while in raw mode")]
    DirNotExistsInRawMode,

    #[error("template `{0}` not found")]
    TemplateNotFound(String),

    #[error("a file at that location already exists")]
    AlreadyExists,
}

pub fn new(
    root: impl AsRef<Utf8Path>,
    config: &Config,
    cmd: NewCommand,
) -> miette::Result<(), Error> {
    // get the target:
    // if cmd.raw ==> target = cmd.path
    // else ==> target = OmniPath(cmd.path).unalias().pathize()
    let root = root.as_ref();

    // if cmd.raw and target.parent() is not a subdir of project_root ==> ERROR
    let mut target: Utf8PathBuf = if cmd.raw {
        let parent = cmd.path.parent().ok_or(Error::NoParent)?.canonicalize()?;

        let mut src = root.canonicalize()?;
        if let Some(prefix_dir) = &config.project.prefix_dir {
            src = src.join(prefix_dir)
        }

        if !parent.starts_with(src) {
            return Err(Error::OutsideRoot);
        }

        cmd.path
    } else {
        OmniPath::try_from_path(cmd.path)?
            .unalias(config)?
            .try_into()?
    };

    // if cmd.raw and target.parent() does not exist ==> ERROR (WE SHOULD ALREADY HAVE CAUGHT THIS, but better be safe)
    // else if !cmd.raw mkdirall if needed
    let target_parent = target.parent().ok_or(Error::NoParent)?;
    let parent_exists = std::fs::exists(target_parent)?;

    if cmd.raw && !parent_exists {
        return Err(Error::DirNotExistsInRawMode);
    } else if !cmd.raw && !parent_exists {
        std::fs::create_dir_all(target_parent)?;
        pretty::debug("mkdiralling");
    }

    // get the template
    let mut found_template_path = None;
    for file in std::fs::read_dir(root.join("resources/templates"))? {
        let file = file?;
        let is_it_the_template = file
            .file_name()
            .to_string_lossy()
            .starts_with(&cmd.template);
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
        None => return Err(Error::TemplateNotFound(cmd.template)),
    };
    target.set_extension(extension);

    pretty::debug(format!("target: {}", target));

    // create the file
    let mut file = match std::fs::File::create_new(&target) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(Error::AlreadyExists);
        }
        Err(err) => return Err(err)?,
    };

    // apply template
    let mut context = tera::Context::new();
    let title = target.file_stem().unwrap_or_default();
    context.insert("title", title);
    context.insert("name", title);

    let new_content = Tera::one_off(&template, &context, false)?;
    file.write_all(new_content.as_bytes())?;

    // track file
    let file_node = track::just_track(root, target)?;

    // run a partial build
    // read nodes
    let nodes_file = match std::fs::read(root.join("build/nodes.toml")) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            pretty::debug("created build/nodes.toml");
            std::fs::File::create(root.join("build/nodes.toml"))?;
            vec![]
        }
        Err(err) => return Err(err.into()),
    };
    let mut nodes: node::Db = toml::from_slice(&nodes_file)?;

    // read links
    let links_file = match std::fs::read(root.join("build/links.toml")) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            pretty::debug("created build/links.toml");
            std::fs::File::create(root.join("build/links.toml"))?;
            vec![]
        }
        Err(err) => return Err(err.into()),
    };
    let mut links: link::Db = toml::from_slice(&links_file)?;

    omni::build::partial::partial(root, config, &mut nodes, &mut links, &file_node, true)?;

    // SAVEPOINT(nodes, links) after a partial build
    let new_nodes_toml = toml::to_string(&nodes)?;
    std::fs::write(root.join("build/nodes.toml"), new_nodes_toml)?;

    let new_links_toml = toml::to_string(&links)?;
    std::fs::write(root.join("build/links.toml"), new_links_toml)?;

    Ok(())
}
