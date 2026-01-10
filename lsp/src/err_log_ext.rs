use std::fmt::Display;

use tower_lsp_server::{Client, ls_types::MessageType};

pub trait ErrLogExt {
    fn log_err(self, msg: &str) -> Self;
    async fn log_err_client(self, msg: &str, client: &Client) -> Self;
    async fn show_err_client(self, msg: &str, client: &Client) -> Self;
}

impl<T, E: Display> ErrLogExt for Result<T, E> {
    fn log_err(self, msg: &str) -> Self {
        if let Err(err) = &self {
            tracing::error!("{}: {}", msg, err);
        };
        self
    }

    async fn log_err_client(self, msg: &str, client: &Client) -> Self {
        if let Err(err) = &self {
            client
                .log_message(MessageType::ERROR, format!("{}: {}", msg, err))
                .await;
        };
        self
    }

    async fn show_err_client(self, msg: &str, client: &Client) -> Self {
        if let Err(err) = &self {
            client
                .show_message(MessageType::ERROR, format!("{}: {}", msg, err))
                .await;
        };
        self
    }
}
