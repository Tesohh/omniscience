use dashmap::DashMap;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer};

use crate::document;

#[derive(Debug)]
pub struct Backend {
    client: Client,
    documents: DashMap<Uri, document::Document>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
        }
    }

    fn capabilities() -> ServerCapabilities {
        ServerCapabilities {
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            completion_provider: Some(CompletionOptions::default()),
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL,
            )),
            ..Default::default()
        }
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: Self::capabilities(),
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .show_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }

    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("You're hovering!".to_string())),
            range: None,
        }))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        log::debug!("client did open {}", params.text_document.uri.as_str());
        self.documents.insert(
            params.text_document.uri,
            document::Document {
                version: params.text_document.version,
                language_id: params.text_document.language_id,
                content: ropey::Rope::from(params.text_document.text),
            },
        );
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        log::debug!("client did change {}", params.text_document.uri.as_str());

        for change in params.content_changes {
            if let Some(range) = change.range {
                // we are using incremental changes
                self.documents
                    .entry(params.text_document.uri.clone())
                    .and_modify(|doc| {
                        doc.version = params.text_document.version;

                        let start_idx = doc.content.line_to_char(range.start.line as usize)
                            + range.start.character as usize;

                        let end_idx = doc.content.line_to_char(range.end.line as usize)
                            + range.end.character as usize;

                        doc.content.remove(start_idx..end_idx);
                        doc.content.insert(start_idx, &change.text);
                        log::debug!("content is now {}", &doc.content);
                    });
            } else {
                // we are using full changes, just replace the whole content
                self.documents
                    .entry(params.text_document.uri.clone())
                    .and_modify(|doc| {
                        doc.version = params.text_document.version;
                        doc.content = ropey::Rope::from(change.text);
                        log::debug!("content is now {}", &doc.content);
                    });
            }
        }
    }
}
