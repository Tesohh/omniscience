pub mod args;
pub mod init;
pub mod new;
pub mod pretty;
pub mod tera;

use std::path::Path;

use clap::Parser;
use miette::{Context, IntoDiagnostic};
use omni::config::Config;

use crate::args::Args;

fn main() -> miette::Result<()> {
    let args = Args::parse();
    let config_path = std::env::current_dir().into_diagnostic()?.join("omni.toml");

    match args.subcommand {
        args::Subcommand::Init(cmd) => init::init(cmd)?,
        args::Subcommand::New(cmd) => {
            let config = read_config(&config_path)?;
            new::new(&config, cmd)?
        }
    }
    Ok(())
}

pub fn read_config(config_path: &impl AsRef<Path>) -> miette::Result<Config> {
    let toml_str = std::fs::read_to_string(config_path)
        .into_diagnostic()
        .wrap_err("failed to read omni.toml")?;

    let config: Config = toml::from_str(&toml_str).into_diagnostic()?;
    Ok(config)
}
