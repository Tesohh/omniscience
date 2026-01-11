use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;

use crate::backend::Backend;
use crate::err_json_rpc_ext::ResultToJsonRpcExt;
use crate::err_log_ext::ErrLogExt;

pub async fn goto_definition(
    backend: &Backend,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>> {
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
        .try_resolve(root, &project.config, &project.nodes)
        .log_err_client("error while resolving link", &backend.client)
        .await
        .rpc()?;
    tracing::debug!("resolved: {resolved:?}");

    match resolved.to {
        omni::link::To::Id(id) => {
            let node = project
                .nodes
                .find_from_id(&id, &project.config)
                .log_err_client("error while fetching node by id", &backend.client)
                .await
                .rpc()?;

            let target_uri = match Uri::from_file_path(node.path.clone()) {
                Some(v) => v,
                None => {
                    backend
                        .client
                        .show_message(MessageType::WARNING, "node exists, but file doesn't")
                        .await;
                    return Ok(None);
                }
            };

            Ok(Some(GotoDefinitionResponse::Scalar(Location {
                uri: target_uri,
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 0,
                    },
                },
            })))
        }
        omni::link::To::Ghost(_) => {
            backend
                .client
                .show_message(MessageType::INFO, "ghost")
                .await;
            Ok(None)
        }
    }
}
