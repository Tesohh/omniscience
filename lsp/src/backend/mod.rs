mod completion;
mod find_root;
mod language_server;
mod register_project;

use std::sync::Arc;

use camino::Utf8PathBuf;
use dashmap::DashMap;
use tower_lsp_server::Client;
use tower_lsp_server::ls_types::*;

use crate::{document, project};

#[derive(Debug)]
pub struct Backend {
    client: Client,
    documents: Arc<DashMap<Uri, document::Document>>,
    projects: Arc<DashMap<Utf8PathBuf, project::Project>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DashMap::new()),
            projects: Arc::new(DashMap::new()),
        }
    }

    pub fn capabilities() -> ServerCapabilities {
        ServerCapabilities {
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec!["@".into()]),
                ..Default::default()
            }),
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL,
            )),
            ..Default::default()
        }
    }
}
