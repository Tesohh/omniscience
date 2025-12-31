use std::{fmt::Display, io::IsTerminal};

use owo_colors::OwoColorize;

pub fn error(err: impl Display) {
    if std::io::stdout().is_terminal() {
        println!("{} {}", "error".red().bold(), err);
    } else {
        println!("error {}", err)
    }
}

pub fn warning(msg: impl Display) {
    if std::io::stdout().is_terminal() {
        println!("{} {}", "warning".yellow().bold(), msg);
    } else {
        println!("warning {}", msg);
    }
}

pub fn info(msg: impl Display) {
    if std::io::stdout().is_terminal() {
        println!("{} {}", "info".blue().bold(), msg);
    } else {
        println!("info {}", msg);
    }
}

pub fn msg(title: impl Display, msg: impl Display) {
    if std::io::stdout().is_terminal() {
        println!("{} {}", title.purple().bold(), msg);
    } else {
        println!("{} {}", title, msg);
    }
}
