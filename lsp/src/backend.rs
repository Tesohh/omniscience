use camino::{Utf8Path, Utf8PathBuf};
use dashmap::DashMap;
use omni::config::{self, find_project_root};
use omni::{link, node};
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer};

use crate::{document, project};

#[derive(Debug)]
pub struct Backend {
    client: Client,
    documents: DashMap<Uri, document::Document>,
    projects: DashMap<Utf8PathBuf, project::Project>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            projects: DashMap::new(),
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
            .log_message(MessageType::INFO, "server initialized!")
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

        let maybe_root: Option<Utf8PathBuf> = {
            let is_file = params.text_document.uri.scheme().as_str() == "file";

            if is_file && let Some(path) = params.text_document.uri.to_file_path() {
                let path = Utf8Path::from_path(&path).expect("path should always be valid utf8");

                match find_project_root(path) {
                    Ok(root) => {
                        // load the project (if not already loaded)
                        let mut project_ok = true;
                        if !self.projects.contains_key(&root) {
                            let project = project::Project::load_project(&root)
                                .await
                                .map_err(|err| log::error!("failed to load project: {}", err))
                                .ok();

                            if let Some(project) = project {
                                self.projects.insert(root.clone(), project);
                            } else {
                                project_ok = false
                            }
                        }

                        if project_ok { Some(root) } else { None }
                    }
                    Err(omni::config::Error::NoProjectRoot) => {
                        log::warn!("opened file outside a project root");
                        None
                    }
                    Err(err) => {
                        log::error!("{}", err);
                        None
                    }
                }
            } else {
                log::error!("got file with invalid uri: {:#?}", params.text_document.uri);
                return;
            }
        };

        log::debug!("maybe_root: {:?}", maybe_root);

        self.documents.insert(
            params.text_document.uri,
            document::Document {
                project_root: maybe_root,
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
                    });
            } else {
                // we are using full changes, just replace the whole content
                self.documents
                    .entry(params.text_document.uri.clone())
                    .and_modify(|doc| {
                        doc.version = params.text_document.version;
                        doc.content = ropey::Rope::from(change.text);
                    });
            }
        }
    }
}
