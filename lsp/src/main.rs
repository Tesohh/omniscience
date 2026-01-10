pub mod backend;
pub mod document;
mod err_log_ext;
pub mod links;
pub mod project;
use tower_lsp_server::{LspService, Server};
use tracing_subscriber::{Layer, filter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::backend::Backend;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_filter(filter::LevelFilter::DEBUG);
    tracing_subscriber::registry().with(stderr_layer).init();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
