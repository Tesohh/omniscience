use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use dashmap::DashMap;
use notify::Watcher;
use omni::config::find_project_root;
use thiserror::Error;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer};

use crate::err_log_ext::ErrLogExt;
use crate::{document, project};

#[derive(Debug)]
pub struct Backend {
    client: Client,
    documents: Arc<DashMap<Uri, document::Document>>,
    projects: Arc<DashMap<Utf8PathBuf, project::Project>>,
}

#[derive(Error, Debug)]
enum ProjectRegisterError {
    #[error(transparent)]
    LoadError(#[from] project::LoadError),

    #[error(transparent)]
    NotifyError(#[from] notify::Error),
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DashMap::new()),
            projects: Arc::new(DashMap::new()),
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

    async fn register_project(
        &self,
        root: &Utf8PathBuf,
    ) -> std::result::Result<(), ProjectRegisterError> {
        if !self.projects.contains_key(root) {
            let project = project::Project::load_project(root).await?;
            self.projects.insert(root.clone(), project);

            // if we inserted a new project then start watching it

            let root_clone = root.clone();
            let projects_clone = Arc::clone(&self.projects);

            tokio::spawn(async move {
                // WARNING: watching files might cause a data race
                // if we have a mutated project,
                // and in the meantime a CLI edits the project or something.

                let _ = project::start_watching_project(root_clone, projects_clone)
                    .await
                    .log_err("cannot watch project");
            });
        }

        Ok(())
    }

    fn find_root_from_uri(uri: &Uri) -> Option<Utf8PathBuf> {
        let is_file = uri.scheme().as_str() == "file";

        if !is_file {
            tracing::error!("got file with invalid uri: {:#?}", uri);
            return None;
        }

        if let Some(path) = uri.to_file_path() {
            let path = Utf8Path::from_path(&path).expect("path should always be valid utf8");
            match find_project_root(path) {
                Ok(root) => return Some(root),
                Err(omni::config::Error::NoProjectRoot) => {
                    tracing::warn!("opened file outside a project root");
                    return None;
                }
                Err(err) => {
                    tracing::error!("{}", err);
                    return None;
                }
            }
        }

        None
    }
}

impl LanguageServer for Backend {
    #[tracing::instrument]
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: Self::capabilities(),
            ..Default::default()
        })
    }

    #[tracing::instrument]
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    #[tracing::instrument]
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument]
    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }

    #[tracing::instrument]
    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("You're hovering!".to_string())),
            range: None,
        }))
    }

    #[tracing::instrument]
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        tracing::debug!("client did open {}", params.text_document.uri.as_str());

        let maybe_root = Self::find_root_from_uri(&params.text_document.uri);
        if let Some(root) = &maybe_root {
            let _ = self
                .register_project(root)
                .await
                .map_err(|err| tracing::error!("err while registering project {}", err));
        };

        tracing::debug!("maybe_root: {:?}", maybe_root);

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

    #[tracing::instrument]
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        tracing::debug!("client did change {}", params.text_document.uri.as_str());

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
