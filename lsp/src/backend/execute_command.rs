use std::str::FromStr;

use camino::Utf8PathBuf;
use thiserror::Error;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;

use crate::backend::Backend;
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

async fn code_action_track(_: &Backend, args: Vec<serde_json::Value>) -> Result<Option<LSPAny>> {
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

    omni::track::track(&root, &file_path).rpc()?;

    Ok(None)
}
