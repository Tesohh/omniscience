use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[command(
    version,
    about = "cli for omniscience. https://github.com/Tesohh/omniscience",
    propagate_version = true
)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum Subcommand {
    Init(InitCommand),
}

#[derive(clap::Parser, Debug)]
pub struct InitCommand {
    pub name: String,
    #[arg(long)]
    pub path: Option<PathBuf>,
}
