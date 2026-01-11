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
    #[error(transparent)]
    TemplateError(#[from] tera::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    OmniPathError(#[from] omni_path::Error),

    #[error(transparent)]
    TomlDeserializeError(#[from] toml::de::Error),

    #[error(transparent)]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error(transparent)]
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

    #[error(transparent)]
    TemplateFetchError(#[from] omni::get_template::Error),

    #[error("a file at that location already exists")]
    AlreadyExists,

    #[error(transparent)]
    CoreTrackError(#[from] omni::track::Error),
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
    let (template, extension) = omni::get_template::get_template(root, &cmd.template)?;

    target.set_extension(extension);

    pretty::debug(format!("target: {}", target));

    // create the file
    let mut file = if cmd.overwrite {
        std::fs::File::create(&target)?
    } else {
        match std::fs::File::create_new(&target) {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(Error::AlreadyExists);
            }
            Err(err) => return Err(err)?,
        }
    };

    // apply template
    let mut context = tera::Context::new();
    let title = target.file_stem().unwrap_or_default();
    context.insert("title", title);
    context.insert("name", title);

    let new_content = Tera::one_off(&template, &context, false)?;
    file.write_all(new_content.as_bytes())?;

    // track file
    let file_node = omni::track::track(root, target)?;

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

    // SAVEPOINT(nodes, links, root) after a partial build
    let new_nodes_toml = toml::to_string(&nodes)?;
    std::fs::write(root.join("build/nodes.toml"), new_nodes_toml)?;

    let new_links_toml = toml::to_string(&links)?;
    std::fs::write(root.join("build/links.toml"), new_links_toml)?;

    std::fs::write(root.join("build/root"), root.as_str())?;
    Ok(())
}
