use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use tera::Tera;
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
                let template_path = entry.path();
                let Some(stem) = template_path.file_stem() else {
                    continue;
                };

                // if the extension does not match just go on
                let my_path: Utf8PathBuf = uri.path().into();
                if let Some(my_ext) = my_path.extension()
                    && let Some(other_ext) = template_path.extension()
                    && my_ext != other_ext
                {
                    continue;
                }

                // get the template
                let template_name = stem.to_os_string().to_string_lossy().to_string();
                let (template, _) =
                    omni::get_template::get_template(&root, &template_name).rpc()?;

                let target_arg = serde_json::Value::String(uri.to_string());

                // prepare the template
                let mut context = tera::Context::new();
                let title = my_path.file_stem().unwrap_or_default();
                context.insert("title", title);
                context.insert("name", title);

                // render the template
                let content = Tera::one_off(&template, &context, false).rpc()?;

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
                                new_text: content,
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
