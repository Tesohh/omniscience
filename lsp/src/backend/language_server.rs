use tower_lsp_server::LanguageServer;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;

use crate::backend::Backend;
use crate::document;
use crate::err_json_rpc_ext::ResultToJsonRpcExt;
use crate::err_log_ext::ErrLogExt;

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
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        if let Some(context) = &params.context {
            match context.trigger_kind {
                CompletionTriggerKind::TRIGGER_CHARACTER => {}
                _ => return Ok(None),
            }
        } else {
            return Ok(None);
        }

        let uri = params.text_document_position.text_document.uri;
        let Some(root) = Self::find_root_from_uri(&uri) else {
            return Ok(None);
        };
        let Some(project) = self.projects.get(&root) else {
            return Ok(None);
        };
        let Some(document) = self.documents.get(&uri) else {
            return Ok(None);
        };

        match document.language_id.as_str() {
            "typst" => {
                // reject anything that doesnt start with @
                if let Some(context) = params.context {
                    if let Some(char) = context.trigger_character
                        && char != "@"
                    {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }

                let links = crate::links::get_possible_links(root, &project.config, &project.nodes)
                    .log_err("error while getting links for completion")
                    .show_err_client("cmp err", &self.client)
                    .await
                    .rpc()?;
                let completions: Vec<CompletionItem> = links
                    .iter()
                    .map(|l| CompletionItem {
                        label: l.omni_path.as_typst_style(),
                        kind: Some(CompletionItemKind::FILE),

                        ..Default::default()
                    })
                    .collect();
                let response = CompletionResponse::Array(completions);
                Ok(Some(response))
            }
            _ => Ok(None),
        }
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
