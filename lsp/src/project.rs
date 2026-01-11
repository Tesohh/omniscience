use std::{fmt::Debug, sync::Arc};

use camino::{Utf8Path, Utf8PathBuf};
use dashmap::DashMap;
use notify::{
    Watcher,
    event::{DataChange, ModifyKind},
};
use omni::{link, node};
use serde::Deserialize;
use thiserror::Error;
use tower_lsp_server::Client;

use crate::err_log_ext::ErrLogExt;

#[derive(Debug)]
/// NOTE: the ultimate source of truth for projects is the filesystem.
/// To avoid data races, you should NEVER mutate project
/// with the intent of then saving it to disk.
/// Leave it to the CLI to do everything.
pub struct Project {
    pub config: omni::config::Config,
    pub user_nodes: omni::node::UserDb,
    pub nodes: omni::node::Db,
    pub links: omni::link::Db,
}

#[derive(Error, Debug)]
pub enum LoadError {
    #[error(transparent)]
    IoError(#[from] tokio::io::Error),

    #[error(transparent)]
    TomlDeserializeError(#[from] toml::de::Error),
}

impl Project {
    #[tracing::instrument]
    pub async fn read_and_parse_file<T, F>(file: F) -> Result<T, LoadError>
    where
        T: for<'a> Deserialize<'a>,
        F: AsRef<Utf8Path> + Debug,
    {
        let db_file = tokio::fs::read(file.as_ref()).await?;
        Ok(toml::from_slice(&db_file)?)
    }

    #[tracing::instrument]
    pub async fn load_project(root: &Utf8PathBuf) -> Result<Self, LoadError> {
        Ok(Self {
            config: Self::read_and_parse_file(root.join("omni.toml")).await?,
            user_nodes: match Self::read_and_parse_file(root.join("nodes.toml")).await {
                Ok(v) => v,
                Err(err) => {
                    tracing::error!(
                        "error while loading {}. err: {}",
                        root.join("nodes.toml"),
                        err
                    );
                    node::UserDb { files: vec![] }
                }
            },
            nodes: match Self::read_and_parse_file(root.join("build/nodes.toml")).await {
                Ok(v) => v,
                Err(err) => {
                    tracing::error!(
                        "error while loading {}. err: {}",
                        root.join("build/nodes.toml"),
                        err
                    );
                    node::Db { nodes: vec![] }
                }
            },
            links: match Self::read_and_parse_file(root.join("build/links.toml")).await {
                Ok(v) => v,
                Err(err) => {
                    tracing::error!(
                        "error while loading {}. err: {}",
                        root.join("build/links.toml"),
                        err
                    );
                    link::Db { links: vec![] }
                }
            },
        })
    }
}

#[derive(Error, Debug)]
pub enum WatchError {
    #[error(transparent)]
    NotifyError(#[from] notify::Error),

    #[error(transparent)]
    LoadError(#[from] LoadError),
}

#[tracing::instrument]
pub async fn start_watching_project(
    root: Utf8PathBuf,
    projects: Arc<DashMap<Utf8PathBuf, Project>>,
    client: Client,
) -> Result<(), WatchError> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<notify::Result<notify::Event>>(32);

    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.blocking_send(res).log_err("cannot send notify event");
    })?;

    let config_path = root.join("omni.toml");
    let user_nodes_path = root.join("nodes.toml");
    let nodes_path = root.join("build/nodes.toml");
    let links_path = root.join("build/links.toml");

    let targets = [&config_path, &user_nodes_path, &nodes_path, &links_path];

    for path in &targets {
        watcher.watch(&root.join_os(path), notify::RecursiveMode::NonRecursive)?;
    }

    while let Some(event) = rx.recv().await {
        let event = event?;
        if let notify::EventKind::Modify(ModifyKind::Data(DataChange::Content)) = event.kind {
            for path in event.paths {
                if !targets.map(|t| root.join_os(t)).contains(&path) {
                    continue;
                }

                let mut project = projects
                    .get_mut(&root)
                    .expect("project should always exist");

                tracing::info!("reloading {}", path.display());

                // TODO: cleanup
                if path == root.join_os("omni.toml") {
                    let Ok(config) = Project::read_and_parse_file(root.join("omni.toml"))
                        .await
                        .log_err("cannot read omni.toml")
                    else {
                        continue;
                    };
                    project.config = config;
                } else if path == root.join_os("nodes.toml") {
                    let Ok(user_nodes) = Project::read_and_parse_file(root.join("nodes.toml"))
                        .await
                        .log_err("cannot read nodes.toml")
                    else {
                        continue;
                    };
                    project.user_nodes = user_nodes;
                } else if path == root.join_os("build/nodes.toml") {
                    let Ok(nodes) = Project::read_and_parse_file(root.join("build/nodes.toml"))
                        .await
                        .log_err("cannot read build/nodes.toml")
                    else {
                        continue;
                    };
                    project.nodes = nodes;
                } else if path == root.join_os("build/links.toml") {
                    let Ok(links) = Project::read_and_parse_file(root.join("build/links.toml"))
                        .await
                        .log_err("cannot read build/links.toml")
                    else {
                        continue;
                    };
                    project.links = links;
                };
            }
        }
    }

    Ok(())
}
