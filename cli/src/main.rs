use clap::{Parser, command};
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum MyLibError {
    #[error(transparent)]
    #[diagnostic(code(my_lib::io_error))]
    IoError(#[from] std::io::Error),

    #[error("Oops it blew up")]
    #[diagnostic(code(my_lib::BAD))]
    #[diagnostic(help("helplpp"))]
    BadThingHappened,

    #[error(transparent)]
    // Use `#[diagnostic(transparent)]` to wrap another [`Diagnostic`]. You won't see labels otherwise
    #[diagnostic(transparent)]
    AnotherError(#[from] AnotherError),
}
#[derive(Error, Diagnostic, Debug)]
#[error("another error")]
pub struct AnotherError {
    #[label("here")]
    pub at: SourceSpan,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: String,
}

fn main() -> miette::Result<()> {
    let args = Args::parse();

    if args.name == "bad" {
        Err(MyLibError::BadThingHappened.into())
    } else {
        println!("Hi {}", args.name);
        Ok(())
    }
}
