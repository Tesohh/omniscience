use omni::node;
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

    match maybe_node {
        Some(node) => {
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
        None => {
            backend
                .client
                .show_message(MessageType::INFO, "ghost")
                .await;
            Ok(None)
        }
    }
}
