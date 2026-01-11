use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;

use crate::backend::Backend;
use crate::err_json_rpc_ext::ResultToJsonRpcExt;
use crate::err_log_ext::ErrLogExt;

pub async fn hover(backend: &Backend, params: HoverParams) -> Result<Option<Hover>> {
    let uri = params.text_document_position_params.text_document.uri;
    let Some(document) = backend.documents.get(&uri) else {
        return Ok(None);
    };
    let Some(root) = &document.project_root else {
        return Ok(None);
    };
    let Some(project) = backend.projects.get(root) else {
        return Ok(None);
    };

    let unresolved = match document.link_under_cursor(params.text_document_position_params.position)
    {
        Some(v) => v,
        None => return Ok(None),
    };

    tracing::debug!("unresolved: {unresolved:?}");

    let resolved = unresolved
        .try_resolve(&project.config, &project.nodes)
        .log_err_client("error while resolving link", &backend.client)
        .await
        .rpc()?;

    match resolved.to {
        omni::link::To::Id(id) => {
            let node = project
                .nodes
                .find_from_id(&id, &project.config)
                .log_err_client("error while fetching node by id", &backend.client)
                .await
                .rpc()?;

            let content = tokio::fs::read_to_string(&node.path)
                .await
                .log_err_client("unable to read file", &backend.client)
                .await
                .unwrap_or("**Unable to read file**".to_string());

            Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "`{}`\n\n```{}\n{}\n```",
                        node.path,
                        node.path.extension().unwrap_or_default(),
                        content
                    ),
                }),
                range: None, // TODO: when we have heading parts we need to take this into consideration
            }))
        }
        omni::link::To::Ghost(_) => Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Ghost".into(),
            }),
            range: None,
        })),
    }
}
