use tower_lsp_server::LanguageServer;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;

use crate::backend::Backend;
use crate::document;

impl LanguageServer for Backend {
    #[tracing::instrument(skip_all)]
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: Self::capabilities(),
            ..Default::default()
        })
    }

    #[tracing::instrument(skip_all)]
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    #[tracing::instrument(skip_all)]
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        crate::backend::completion::completion(self, params).await
    }

    #[tracing::instrument(skip_all)]
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        crate::backend::hover::hover(self, params).await
    }

    #[tracing::instrument(skip_all)]
    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        crate::backend::goto_definition::goto_definition(self, params).await
    }

    #[tracing::instrument(skip_all)]
    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        crate::backend::code_action::code_action(self, params).await
    }

    #[tracing::instrument(skip_all)]
    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<LSPAny>> {
        crate::backend::execute_command::execute_command(self, params).await
    }

    #[tracing::instrument(skip_all)]
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        tracing::debug!("client did open {}", params.text_document.uri.as_str());

        let maybe_root = Self::find_root_from_uri(&params.text_document.uri, true);
        if let Some(root) = &maybe_root {
            let _ = self
                .register_project(root)
                .await
                .map_err(|err| tracing::error!("err while registering project {}", err));
        };

        tracing::debug!("maybe_root: {:?}", maybe_root);

        self.documents.insert(
            params.text_document.uri.clone(),
            document::Document {
                project_root: maybe_root,
                path: params.text_document.uri.path().into(),
                version: params.text_document.version,
                language_id: params.text_document.language_id,
                content: ropey::Rope::from(params.text_document.text),
            },
        );
    }

    #[tracing::instrument(skip_all)]
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
