use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[command(
    author = "tesohh",
    version,
    about = "cli for omniscience. https://github.com/Tesohh/omniscience",
    propagate_version = true,
    name = "omni",
    bin_name = "omni",
    styles = CLAP_STYLING,
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

pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(
        anstyle::Style::new()
            .bold()
            .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Magenta))),
    )
    .usage(
        anstyle::Style::new()
            .bold()
            .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Magenta))),
    )
    .literal(
        anstyle::Style::new()
            .bold()
            .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
    );
