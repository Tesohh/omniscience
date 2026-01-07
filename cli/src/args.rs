use camino::Utf8PathBuf;

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
    Track(TrackCommand),
    Build(BuildCommand),
}

#[derive(clap::Parser, Debug)]
/// Initializes a new project at `./<name>` or at the path provided
pub struct InitCommand {
    pub name: String,
    #[arg(long)]
    pub path: Option<Utf8PathBuf>,
}

#[derive(clap::Parser, Debug)]
/// Creates a new file with the template provided and starts tracking it.
pub struct NewCommand {
    /// basename of a template in `resources/templates`
    pub template: String,
    /// omni-style path to the new file, or regular path if `--raw` is set.
    pub path: Utf8PathBuf,
    #[arg(long)]
    pub raw: bool,
}

#[derive(clap::Parser, Debug)]
/// Starts tracking an existing file.
pub struct TrackCommand {
    /// path to the file you want to track.
    pub path: Utf8PathBuf,
}

#[derive(clap::Parser, Debug)]
/// Performs a full build if no arguments provided, partial build if path specified
pub struct BuildCommand {
    pub path: Option<Utf8PathBuf>,
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
