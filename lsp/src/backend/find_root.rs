use camino::{Utf8Path, Utf8PathBuf};
use omni::config::find_project_root;
use tower_lsp_server::ls_types::Uri;

use crate::backend::Backend;

impl Backend {
    pub fn find_root_from_uri(uri: &Uri, new_file: bool) -> Option<Utf8PathBuf> {
        let is_file = uri.scheme().as_str() == "file";

        if !is_file {
            tracing::error!("got file with invalid uri: {:#?}", uri);
            return None;
        }

        if let Some(path) = uri.to_file_path() {
            let mut path = Utf8Path::from_path(&path).expect("path should always be valid utf8");
            if new_file {
                path = path.parent().expect("should always have a parent");
            }
            match find_project_root(path) {
                Ok(root) => return Some(root),
                Err(omni::config::Error::NoProjectRoot) => {
                    tracing::warn!("opened file outside a project root");
                    return None;
                }
                Err(err) => {
                    tracing::error!("{}", err);
                    return None;
                }
            }
        }

        None
    }
}
