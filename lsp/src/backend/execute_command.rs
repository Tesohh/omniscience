use std::str::FromStr;

use camino::Utf8PathBuf;
use thiserror::Error;
use tower_lsp_server::LanguageServer;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;

use crate::backend::Backend;
use crate::document;
use crate::err_json_rpc_ext::ResultToJsonRpcExt;
use crate::err_log_ext::ErrLogExt;

pub async fn execute_command(
    backend: &Backend,
    params: ExecuteCommandParams,
) -> Result<Option<LSPAny>> {
    if params.command == "code_action_track" {
        code_action_track(backend, params.arguments).await
    } else {
        Ok(None)
    }
}

#[derive(Error, Debug)]
enum CodeActionTrackError {
    #[error("invalid args")]
    InvalidArgs,
}

async fn code_action_track(
    backend: &Backend,
    args: Vec<serde_json::Value>,
) -> Result<Option<LSPAny>> {
    let file_uri = args
        .first()
        .ok_or(CodeActionTrackError::InvalidArgs)
        .rpc()?;
    let Some(file_uri) = file_uri.as_str() else {
        return Err(CodeActionTrackError::InvalidArgs).rpc();
    };

    let uri = Uri::from_str(file_uri)
        .log_err("cannot make uri from args")
        .rpc()?;

    let file_path: Utf8PathBuf = uri.path().into();

    let Some(root) = Backend::find_root_from_uri(&uri, true) else {
        return Ok(None);
    };
    let Some(project) = backend.projects.get(&root) else {
        return Ok(None);
    };

    // cli::new::new(
    //     root,
    //     &project.config,
    //     cli::args::NewCommand {
    //         template: template.to_string(),
    //         path: file_path,
    //         raw: true,
    //         overwrite: true,
    //     },
    // )
    // .rpc()?;
    // Ok(None)

    Ok(None)
}

// async fn code_action_new(
//     backend: &Backend,
//     args: Vec<serde_json::Value>,
// ) -> Result<Option<LSPAny>> {
//     let file_uri = args.first().ok_or(CodeActionNewError::InvalidArgs).rpc()?;
//     let template = args.get(1).ok_or(CodeActionNewError::InvalidArgs).rpc()?;
//
//     let Some(file_uri) = file_uri.as_str() else {
//         return Err(CodeActionNewError::InvalidArgs).rpc();
//     };
//     let Some(template) = template.as_str() else {
//         return Err(CodeActionNewError::InvalidArgs).rpc();
//     };
//
//     let uri = Uri::from_str(file_uri)
//         .log_err("cannot make uri from args")
//         .rpc()?;
//
//     let file_path: Utf8PathBuf = uri.path().into();
//
//     let Some(root) = Backend::find_root_from_uri(&uri, true) else {
//         return Ok(None);
//     };
//     let Some(project) = backend.projects.get(&root) else {
//         return Ok(None);
//     };
//
//     cli::new::new(
//         root,
//         &project.config,
//         cli::args::NewCommand {
//             template: template.to_string(),
//             path: file_path,
//             raw: true,
//             overwrite: true,
//         },
//     )
//     .rpc()?;
//     Ok(None)
// }
