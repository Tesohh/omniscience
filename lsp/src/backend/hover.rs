use omni::node;
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

    // TODO: switch to resolve_link without the from part
    let maybe_node =
        match project
            .nodes
            .find_from_filepart(&root, &unresolved.file_part, &project.config)
        {
            Ok(node) => Some(node),
            Err(node::Error::NameNotFound(_)) => None,
            Err(_) => return Ok(None),
        };

    // tracing::debug!("resolved: {resolved:?}");

    match maybe_node {
        Some(node) => {
            let content = tokio::fs::read_to_string(&node.path)
                .await
                .log_err_client("unable to read file", &backend.client)
                .await
                .unwrap_or("**Unable to read file**".to_string());

            Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "omni node `{}` at **{}**\n\n```{}\n{}\n```",
                        node.id,
                        node.path.strip_prefix(root).unwrap_or(&node.path),
                        node.path.extension().unwrap_or_default(),
                        content
                    ),
                }),
                range: None, // TODO: when we have heading parts we need to take this into consideration
            }))
        }
        None => Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Ghost".into(),
            }),
            range: None,
        })),
    }
}
