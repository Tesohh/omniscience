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
    New(NewCommand),
}

#[derive(clap::Parser, Debug)]
/// Initializes a new project at `./<name>` or at the path provided
pub struct InitCommand {
    pub name: String,
    #[arg(long)]
    pub path: Option<PathBuf>,
}

#[derive(clap::Parser, Debug)]
/// Creates a new file with the template provided and starts tracking it.
pub struct NewCommand {
    /// basename of a template in `resources/templates`
    pub template: String,
    /// location of the new file.
    #[arg(
        long_help = "if it starts with `src`, it is treated as a regular os path, and extension must be added (eg. src/linear-algebra/matrix.typ)
if it doesn't, dir aliases will be considered, and extension is not needed and is inferred from template (eg. linalg/matrix)"
    )]
    pub path: PathBuf,
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
