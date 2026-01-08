pub mod backend;
pub mod document;

use std::path::Path;

use ftail::Ftail;
use tower_lsp_server::{LspService, Server};

use crate::backend::Backend;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    Ftail::new()
        .single_file(Path::new(".logs/log.txt"), true, log::LevelFilter::Trace)
        .init()
        .unwrap();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
