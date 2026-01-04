pub mod args;
pub mod init;
pub mod new;
pub mod pretty;
pub mod tera;

use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use miette::{Context, IntoDiagnostic};
use omni::config::{Config, find_project_root};

use crate::args::Args;

fn main() -> miette::Result<()> {
    let args = Args::parse();
    let cwd = std::env::current_dir().into_diagnostic()?;
    let cwd = Utf8PathBuf::from_path_buf(cwd)
        .map_err(|_| miette::miette!("unable to convert cwd to utf8 path"))?;

    match args.subcommand {
        args::Subcommand::Init(cmd) => init::init(cmd)?,
        args::Subcommand::New(cmd) => {
            let (root, config) = read_config(&cwd)?;
            new::new(&root, &config, cmd)?
        }
    }
    Ok(())
}

pub fn read_config(cwd: &impl AsRef<Utf8Path>) -> miette::Result<(Utf8PathBuf, Config)> {
    let root = find_project_root(cwd)?;

    let toml_str = std::fs::read_to_string(root.join("omni.toml"))
        .into_diagnostic()
        .wrap_err("failed to read omni.toml")?;

    let config: Config = toml::from_str(&toml_str).into_diagnostic()?;
    Ok((root, config))
}
