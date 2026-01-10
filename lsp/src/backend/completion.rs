use crate::backend::Backend;

use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;

use crate::err_json_rpc_ext::ResultToJsonRpcExt;
use crate::err_log_ext::ErrLogExt;

pub async fn completion(
    backend: &Backend,
    params: CompletionParams,
) -> Result<Option<CompletionResponse>> {
    if let Some(context) = &params.context {
        match context.trigger_kind {
            CompletionTriggerKind::TRIGGER_CHARACTER => {}
            _ => return Ok(None),
        }
    } else {
        return Ok(None);
    }

    let uri = params.text_document_position.text_document.uri;
    let Some(root) = Backend::find_root_from_uri(&uri) else {
        return Ok(None);
    };
    let Some(project) = backend.projects.get(&root) else {
        return Ok(None);
    };
    let Some(document) = backend.documents.get(&uri) else {
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
                .show_err_client("cmp err", &backend.client)
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
