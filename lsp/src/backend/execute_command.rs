use std::str::FromStr;

use camino::Utf8PathBuf;
use omni::{link, node};
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
    } else if params.command == "code_action_build" {
        code_action_build(backend, params.arguments).await
    } else {
        Ok(None)
    }
}

#[derive(Error, Debug)]
enum CodeActionBuildError {
    #[error("invalid args")]
    InvalidArgs,
}

async fn code_action_build(
    backend: &Backend,
    args: Vec<serde_json::Value>,
) -> Result<Option<LSPAny>> {
    let token = NumberOrString::String("build".to_string());
    let progress = backend.client.progress(token, "Building").begin().await;

    let file_uri = args
        .first()
        .ok_or(CodeActionBuildError::InvalidArgs)
        .rpc()?;
    let Some(file_uri) = file_uri.as_str() else {
        return Err(CodeActionBuildError::InvalidArgs).rpc();
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

    let path_canonical = file_path.canonicalize_utf8().rpc()?;
    let file = project
        .user_nodes
        .files
        .iter()
        .filter_map(|f| match f.path.canonicalize_utf8() {
            Ok(p) => Some((f, p)),
            Err(err) => {
                tracing::warn!(
                    "invalid path found in nodes.toml for id {}. error: {}",
                    f.id,
                    err,
                );
                None
            }
        })
        .find(|file| file.1 == path_canonical)
        .map(|(f, _)| f)
        .ok_or(node::Error::UntrackedNode(file_path))
        .show_err_client("build err", &backend.client)
        .await
        .rpc()?;

    // we have to get new nodes and links because we cannot mutate project.nodes and links
    let mut nodes: node::Db = {
        let db_file = tokio::fs::read(root.join("build/nodes.toml"))
            .await
            .unwrap_or_else(|_| {
                let _ = std::fs::File::create(root.join("build/nodes.toml"));
                vec![]
            });
        toml::from_slice(&db_file).rpc()?
    };
    let mut links: link::Db = {
        let db_file = tokio::fs::read(root.join("build/links.toml"))
            .await
            .unwrap_or_else(|_| {
                let _ = std::fs::File::create(root.join("build/links.toml"));
                vec![]
            });
        toml::from_slice(&db_file).rpc()?
    };

    omni::build::partial::partial(&root, &project.config, &mut nodes, &mut links, file, true)
        .show_err_client("build err", &backend.client)
        .await
        .rpc()?;

    // SAVEPOINT(nodes, links, root)
    let new_nodes_toml = toml::to_string(&nodes).rpc()?;
    std::fs::write(root.join("build/nodes.toml"), new_nodes_toml).rpc()?;

    let new_links_toml = toml::to_string(&links).rpc()?;
    std::fs::write(root.join("build/links.toml"), new_links_toml).rpc()?;

    std::fs::write(root.join("build/root"), root.as_str()).rpc()?;

    progress.finish_with_message("Done").await;

    Ok(None)
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
