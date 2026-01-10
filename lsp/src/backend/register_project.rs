use std::sync::Arc;

use camino::Utf8PathBuf;
use thiserror::Error;

use crate::backend::Backend;
use crate::err_log_ext::ErrLogExt;
use crate::project;

#[derive(Error, Debug)]
pub enum ProjectRegisterError {
    #[error(transparent)]
    LoadError(#[from] project::LoadError),

    #[error(transparent)]
    NotifyError(#[from] notify::Error),
}

impl Backend {
    pub async fn register_project(
        &self,
        root: &Utf8PathBuf,
    ) -> std::result::Result<(), ProjectRegisterError> {
        if !self.projects.contains_key(root) {
            let project = project::Project::load_project(root).await?;
            self.projects.insert(root.clone(), project);

            // if we inserted a new project then start watching it

            let root_clone = root.clone();
            let projects_clone = Arc::clone(&self.projects);
            let client_clone = self.client.clone();

            tokio::spawn(async move {
                // WARNING: watching files might cause a data race
                // if we have a mutated project,
                // and in the meantime a CLI edits the project or something.

                let _ = project::start_watching_project(root_clone, projects_clone, client_clone)
                    .await
                    .log_err("cannot watch project");
            });
        }

        Ok(())
    }
}
