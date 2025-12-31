pub mod args;
pub mod init;
pub mod pretty;
pub mod tera;

use clap::Parser;

use crate::args::Args;

fn main() -> miette::Result<()> {
    let args = Args::parse();
    match args.subcommand {
        args::Subcommand::Init(cmd) => init::init(cmd)?,
    }
    Ok(())
}
