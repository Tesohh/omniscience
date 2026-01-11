use std::collections::HashMap;

use camino::Utf8Path;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;

use crate::backend::Backend;
use crate::document::Document;
use crate::err_json_rpc_ext::ResultToJsonRpcExt;
use crate::err_log_ext::ErrLogExt;

pub async fn code_action(
    backend: &Backend,
    params: CodeActionParams,
) -> Result<Option<CodeActionResponse>> {
    let uri = params.text_document.uri;
    let document = backend.documents.get(&uri);

    let Some(root) = Backend::find_root_from_uri(&uri, true) else {
        return Ok(None);
    };

    let mut commands = vec![];
    commands.extend(get_template_actions(backend, &uri, &root).await?);

    Ok(Some(commands))
}

async fn get_template_actions(
    backend: &Backend,
    uri: &Uri,
    root: impl AsRef<Utf8Path>,
) -> Result<Vec<CodeActionOrCommand>> {
    if let Some(document) = backend.documents.get(uri)
        && document.content.len_chars() > 1
    {
        return Ok(vec![]);
    }

    let mut templates = tokio::fs::read_dir(root.as_ref().join("resources/templates"))
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

                commands.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Apply template \"{}\" and track", stem.display()),
                    kind: Some(CodeActionKind::SOURCE),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(
                            uri.clone(),
                            vec![TextEdit {
                                range: Range {
                                    start: Position::new(0, 0),
                                    end: Position::new(0, 0),
                                },
                                new_text: "TODO TEMPLAET".to_string(),
                            }],
                        )])),
                        ..Default::default()
                    }),
                    command: Some(Command {
                        title: "Track".to_string(),
                        command: "code_action_track".into(),
                        arguments: Some(vec![target_arg]),
                    }),
                    ..Default::default()
                }));
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }

    Ok(commands)
}
