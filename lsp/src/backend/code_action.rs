use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;

use crate::backend::Backend;
use crate::err_json_rpc_ext::ResultToJsonRpcExt;
use crate::err_log_ext::ErrLogExt;

pub async fn code_action(
    backend: &Backend,
    params: CodeActionParams,
) -> Result<Option<CodeActionResponse>> {
    let uri = params.text_document.uri;
    if let Some(document) = backend.documents.get(&uri)
        && document.content.len_chars() > 1
    {
        return Ok(None);
    };

    let Some(root) = Backend::find_root_from_uri(&uri, true) else {
        return Ok(None);
    };

    let mut templates = tokio::fs::read_dir(root.join("resources/templates"))
        .await
        .log_err_client("cannot read resources/templates", &backend.client)
        .await
        .rpc()?;

    let mut commands: Vec<CodeActionOrCommand> = vec![];

    loop {
        match templates
            .next_entry()
            .await
            .log_err_client("next_entry error", &backend.client)
            .await
        {
            Ok(Some(entry)) => {
                let path = entry.path();
                let Some(stem) = path.file_stem() else {
                    continue;
                };

                let target_arg = serde_json::Value::String(uri.to_string());
                let template_arg =
                    serde_json::Value::String(stem.to_os_string().to_string_lossy().to_string());

                commands.push(CodeActionOrCommand::Command(Command {
                    title: format!("Create new {}", stem.display()),
                    command: "code_action_new".into(),
                    arguments: Some(vec![target_arg, template_arg]),
                }));
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }

    Ok(Some(commands))
}
