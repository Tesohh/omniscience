use camino::{Utf8Path, Utf8PathBuf};
use omni::{link, node};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug)]
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
    pub async fn read_and_parse_file<T>(file: impl AsRef<Utf8Path>) -> Result<T, LoadError>
    where
        T: for<'a> Deserialize<'a>,
    {
        let db_file = tokio::fs::read(file.as_ref()).await?;
        Ok(toml::from_slice(&db_file)?)
    }

    pub async fn load_project(root: &Utf8PathBuf) -> Result<Self, LoadError> {
        Ok(Self {
            config: Self::read_and_parse_file(root.join("omni.toml")).await?,
            user_nodes: match Self::read_and_parse_file(root.join("nodes.toml")).await {
                Ok(v) => v,
                Err(err) => {
                    log::error!(
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
                    log::error!(
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
                    log::error!(
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
